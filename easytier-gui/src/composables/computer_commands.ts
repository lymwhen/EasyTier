import { httpRequest } from './backend'

export type CommandMode = 'INSTANT' | 'LONG_RUNNING'

export interface AppCommand {
  id: string
  name: string
  command: string
  mode: CommandMode
}

export type CommandStatus = 'RUNNING' | 'SUCCESS' | 'FAILED'

export interface CommandResult {
  status: CommandStatus
  taskId?: string
  output?: string
  error?: string
}

export interface AgentTask {
  taskId: string
  command: string
  mode: CommandMode
  pid: number
  startedAt: string
  elapsedSecs: number
}

export interface AgentRequest {
  ip: string
  port: number
  proxy?: string
  token: string
}

function agentUrl(request: AgentRequest, path: string) {
  return `http://${request.ip}:${request.port}${path}`
}

function agentHeaders(token: string) {
  return {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  }
}

async function agentJson<T>(
  request: AgentRequest,
  method: string,
  path: string,
  body?: unknown,
  requestId?: string,
): Promise<T> {
  const response = await httpRequest({
    method,
    url: agentUrl(request, path),
    proxy: request.proxy,
    timeoutSecs: 4,
    headers: agentHeaders(request.token),
    body: body == null ? undefined : JSON.stringify(body),
    requestId,
  })

  let payload: any = {}
  try {
    payload = response.body ? JSON.parse(response.body) : {}
  }
  catch {
    payload = {}
  }

  if (response.status === 401) {
    const error = new Error('unauthorized') as Error & { status?: number }
    error.status = 401
    throw error
  }
  if (response.status < 200 || response.status >= 300) {
    const message = payload?.message || payload?.error?.message || `HTTP ${response.status}`
    const error = new Error(message) as Error & { status?: number }
    error.status = response.status
    throw error
  }
  return payload as T
}

// POST /api/v1/command/execute — starts a remote command on the agent.
export async function executeAgentCommand(
  request: AgentRequest,
  command: Pick<AppCommand, 'command' | 'mode'>,
  requestId?: string,
): Promise<CommandResult> {
  return await agentJson<CommandResult>(
    request,
    'POST',
    '/api/v1/command/execute',
    { command: command.command, mode: command.mode },
    requestId,
  )
}

// GET /api/v1/command/status — polls a previously started task.
export async function getAgentCommandStatus(
  request: AgentRequest,
  taskId: string,
  requestId?: string,
): Promise<CommandResult> {
  return await agentJson<CommandResult>(
    request,
    'GET',
    `/api/v1/command/status?taskId=${encodeURIComponent(taskId)}`,
    undefined,
    requestId,
  )
}

// GET /api/v1/command/tasks — ledger of tasks whose process is still alive.
export async function listAgentTasks(
  request: AgentRequest,
  requestId?: string,
): Promise<AgentTask[]> {
  const response = await agentJson<{ tasks?: AgentTask[] }>(
    request,
    'GET',
    '/api/v1/command/tasks',
    undefined,
    requestId,
  )
  return Array.isArray(response.tasks) ? response.tasks : []
}

// POST /api/v1/command/stop — kills the process tree of a live task.
export async function stopAgentTask(
  request: AgentRequest,
  taskId: string,
  requestId?: string,
): Promise<CommandResult> {
  return await agentJson<CommandResult>(
    request,
    'POST',
    '/api/v1/command/stop',
    { taskId },
    requestId,
  )
}

export function createUuid(): string {
  const cryptoApi = globalThis.crypto as Crypto | undefined
  if (cryptoApi?.randomUUID)
    return cryptoApi.randomUUID()
  const bytes = new Uint8Array(16)
  cryptoApi?.getRandomValues?.(bytes)
  if (!bytes.some(byte => byte !== 0)) {
    for (let index = 0; index < bytes.length; index++) {
      bytes[index] = Math.floor(Math.random() * 256)
    }
  }
  bytes[6] = (bytes[6] & 0x0F) | 0x40
  bytes[8] = (bytes[8] & 0x3F) | 0x80
  const hex = Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

// ==================== devices.toml command persistence ====================
// Commands are stored inside each [[device]] block as a TOML array of inline
// tables:
//   commands = [
//     { name = "重启资源管理器", command = "taskkill /f /im explorer.exe", mode = "INSTANT" },
//   ]

export interface RawTomlCommand {
  name: string
  command: string
  mode: CommandMode
}

function normalizeMode(value: unknown): CommandMode {
  return String(value ?? '').trim().toUpperCase() === 'LONG_RUNNING' ? 'LONG_RUNNING' : 'INSTANT'
}

// Escape a JS string into a TOML basic string body (between the quotes).
export function escapeTomlString(value: string): string {
  return value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')
}

// Decode a TOML basic/literal string token leniently. Unknown escapes such as
// the `\s` inside "C:\srv\x.js" (typed by hand) keep the backslash.
export function unescapeTomlString(token: string): string {
  const trimmed = token.trim()
  if (trimmed.startsWith('\'') && trimmed.endsWith('\'') && trimmed.length >= 2)
    return trimmed.slice(1, -1)
  if (trimmed.startsWith('"') && trimmed.endsWith('"') && trimmed.length >= 2) {
    const raw = trimmed.slice(1, -1)
    let out = ''
    for (let i = 0; i < raw.length; i++) {
      const ch = raw[i]
      if (ch === '\\' && i + 1 < raw.length) {
        const next = raw[i + 1]
        // Lenient: only \\ and \" are treated as escapes so that hand-typed
        // Windows paths like C:\tools keep their backslashes.
        if (next === '\\' || next === '"') {
          out += next
          i++
          continue
        }
        out += ch + next
        i++
        continue
      }
      out += ch
    }
    return out
  }
  return trimmed
}

// Split an inline table body on top-level commas (quote aware).
function splitTopLevel(text: string): string[] {
  const parts: string[] = []
  let current = ''
  let quote: string | null = null
  for (let i = 0; i < text.length; i++) {
    const ch = text[i]
    if (quote) {
      current += ch
      if (ch === '\\' && quote === '"') {
        if (i + 1 < text.length) {
          current += text[i + 1]
          i++
        }
      }
      else if (ch === quote) {
        quote = null
      }
      continue
    }
    if (ch === '"' || ch === '\'') {
      quote = ch
      current += ch
      continue
    }
    if (ch === ',') {
      parts.push(current)
      current = ''
      continue
    }
    current += ch
  }
  if (current.trim())
    parts.push(current)
  return parts
}

function parseInlineTable(body: string): RawTomlCommand | null {
  const fields: Record<string, string> = {}
  for (const part of splitTopLevel(body)) {
    const eq = part.indexOf('=')
    if (eq <= 0)
      continue
    const key = part.slice(0, eq).trim().toLowerCase()
    fields[key] = unescapeTomlString(part.slice(eq + 1))
  }
  const name = (fields.name ?? '').trim()
  const command = (fields.command ?? '').trim()
  if (!name || !command)
    return null
  return { name, command, mode: normalizeMode(fields.mode) }
}

// Extract all `{ ... }` inline tables from a `commands = [ ... ]` value.
export function parseCommandsInline(value: string): RawTomlCommand[] {
  const out: RawTomlCommand[] = []
  let i = 0
  while (i < value.length) {
    if (value[i] === '{') {
      let depth = 1
      let j = i + 1
      let quote: string | null = null
      while (j < value.length && depth > 0) {
        const ch = value[j]
        if (quote) {
          if (quote === '"' && ch === '\\') { j += 2; continue }
          if (ch === quote)
            quote = null
        }
        else if (ch === '"' || ch === '\'') {
          quote = ch
        }
        else if (ch === '{') {
          depth++
        }
        else if (ch === '}') {
          depth--
          if (depth === 0)
            break
        }
        j++
      }
      const entry = parseInlineTable(value.slice(i + 1, j))
      if (entry)
        out.push(entry)
      i = j + 1
      continue
    }
    i++
  }
  return out
}

// Find the index of the line closing the bracketed array that starts inside
// the given line. Quote aware. Returns -1 when unbalanced.
function findArrayEnd(lines: string[], startIdx: number): number {
  let depth = 0
  let quote: string | null = null
  for (let i = startIdx; i < lines.length; i++) {
    const line = lines[i]
    for (let j = 0; j < line.length; j++) {
      const ch = line[j]
      if (quote) {
        if (quote === '"' && ch === '\\') { j++; continue }
        if (ch === quote)
          quote = null
        continue
      }
      if (ch === '"' || ch === '\'') {
        quote = ch
        continue
      }
      if (ch === '[') {
        depth++
      }
      else if (ch === ']') {
        depth--
        if (depth === 0)
          return i
      }
    }
  }
  return -1
}

function serializeCommandsField(commands: RawTomlCommand[]): string[] {
  if (!commands.length)
    return []
  const lines = ['commands = [']
  for (const item of commands) {
    lines.push(
      `  { name = "${escapeTomlString(item.name)}", command = "${escapeTomlString(item.command)}", mode = "${item.mode}" },`,
    )
  }
  lines.push(']')
  return lines
}

// Replace (or insert, or drop) the `commands = [...]` field of one [[device]]
// block identified by its MAC. Everything else in the file — including
// comments and field order — is preserved verbatim.
export function setDeviceCommandsInToml(
  toml: string,
  deviceMac: string,
  commands: RawTomlCommand[],
): string {
  const eol = toml.includes('\r\n') ? '\r\n' : '\n'
  const lines = toml.split(/\r?\n/)
  const normMac = (value: string) => value.replace(/["'\s]/g, '').toLowerCase()
  const target = normMac(deviceMac)
  if (!target)
    return toml

  const headerIdx: number[] = []
  for (let i = 0; i < lines.length; i++) {
    const trimmed = lines[i].trim()
    if (/^\[.+\]$/.test(trimmed))
      headerIdx.push(i)
  }

  let blockStart = -1
  let blockEnd = lines.length
  for (let b = 0; b < headerIdx.length; b++) {
    if (lines[headerIdx[b]].trim() !== '[[device]]')
      continue
    const next = b + 1 < headerIdx.length ? headerIdx[b + 1] : lines.length
    for (let i = headerIdx[b] + 1; i < next; i++) {
      const m = lines[i].match(/^\s*mac\s*=\s*(.+)$/)
      if (m && normMac(m[1]) === target) {
        blockStart = headerIdx[b] + 1
        blockEnd = next
        break
      }
    }
    if (blockStart !== -1)
      break
  }
  if (blockStart === -1)
    return toml

  let fieldStart = -1
  let fieldEnd = -1
  for (let i = blockStart; i < blockEnd; i++) {
    if (/^\s*commands\s*=/.test(lines[i])) {
      fieldStart = i
      fieldEnd = findArrayEnd(lines, i)
      if (fieldEnd === -1)
        fieldEnd = i
      break
    }
  }

  const replacement = serializeCommandsField(commands)
  const before = lines.slice(0, fieldStart === -1 ? blockStart : fieldStart)
  const after = lines.slice(fieldStart === -1 ? blockStart : fieldEnd + 1)

  if (fieldStart === -1) {
    if (!replacement.length)
      return lines.join(eol)
    // Insert at the end of the device block, after the last non-empty line.
    let insertAt = blockEnd
    while (insertAt > blockStart && lines[insertAt - 1].trim() === '')
      insertAt--
    const merged = [
      ...lines.slice(0, insertAt),
      ...replacement,
      ...lines.slice(insertAt),
    ]
    return merged.join(eol)
  }

  const merged = [...before, ...replacement, ...after]
  return merged.join(eol)
}

export function createCommandId(): string {
  return `cmd-${createUuid()}`
}

// Stable UI keys for TOML-backed commands (TOML itself stores no ids).
export function commandKeyFor(deviceId: string, index: number): string {
  return `cmd-${deviceId}-${index}`
}
