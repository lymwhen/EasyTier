import { httpRequest } from './backend'

export type CommandRisk = 'normal' | 'high'
export type CommandRunStatus =
  | 'queued'
  | 'running'
  | 'succeeded'
  | 'failed'
  | 'timed_out'
  | 'canceled'
  | 'interrupted'
  | 'unknown'

export interface AppCommand {
  id: string
  name: string
  program: string
  args: string[]
  timeoutSecs: number
  cancelable: boolean
  risk: CommandRisk
}

export interface CommandRun {
  run_id: string
  request_id: string
  name: string
  program: string
  args: string[]
  risk: CommandRisk
  status: CommandRunStatus
  queued_at: string
  started_at?: string
  finished_at?: string
  duration_ms: number
  exit_code?: number
  error?: string
  cancelable: boolean
  cancel_requested: boolean
  truncated: boolean
  next_seq: number
  eof: boolean
}

export interface CommandOutputChunk {
  seq: number
  stream: 'stdout' | 'stderr'
  text: string
}

export interface CommandOutput {
  next_seq: number
  eof: boolean
  truncated: boolean
  chunks: CommandOutputChunk[]
}

export interface CommandPolicy {
  enabled: boolean
  version: string
  allowed_program_dirs: string[]
  blocked_programs: string[]
  max_concurrent_runs: number
  default_timeout_secs: number
  max_timeout_secs: number
  max_output_bytes: number
  max_requests_per_minute: number
  shell_execution: boolean
  arguments_supported: boolean
  cancel_supported: boolean
  output_cursor_support: boolean
}

interface AgentRequest {
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
  requireToken = true,
): Promise<T> {
  const response = await httpRequest({
    method,
    url: agentUrl(request, path),
    proxy: request.proxy,
    timeoutSecs: 10,
    headers: requireToken ? agentHeaders(request.token) : { 'Content-Type': 'application/json' },
    body: body == null ? undefined : JSON.stringify(body),
  })

  let payload: any = {}
  try {
    payload = response.body ? JSON.parse(response.body) : {}
  }
  catch {
    payload = {}
  }

  if (response.status < 200 || response.status >= 300) {
    const message = payload?.error?.message || payload?.message || `HTTP ${response.status}`
    const error = new Error(message) as Error & { status?: number, code?: string, activeRunId?: string }
    error.status = response.status
    error.code = payload?.error?.code
    error.activeRunId = payload?.active_run_id
    throw error
  }
  return payload as T
}

export async function getCommandPolicy(request: AgentRequest): Promise<CommandPolicy> {
  return await agentJson<CommandPolicy>(
    request,
    'GET',
    '/api/v1/command-policy',
    undefined,
    false,
  )
}

export async function listActiveCommandRuns(request: AgentRequest): Promise<CommandRun[]> {
  const response = await agentJson<{ runs: CommandRun[] }>(
    request,
    'GET',
    '/api/v1/command-runs?active=true',
  )
  return response.runs || []
}

export async function getCommandRun(request: AgentRequest, runId: string): Promise<CommandRun> {
  return await agentJson<CommandRun>(
    request,
    'GET',
    `/api/v1/command-runs/${encodeURIComponent(runId)}`,
  )
}

export async function getCommandOutput(
  request: AgentRequest,
  runId: string,
  afterSeq: number,
): Promise<CommandOutput> {
  return await agentJson<CommandOutput>(
    request,
    'GET',
    `/api/v1/command-runs/${encodeURIComponent(runId)}/output?after_seq=${afterSeq}`,
  )
}

export async function createCommandRun(
  request: AgentRequest,
  command: AppCommand,
): Promise<CommandRun> {
  return await agentJson<CommandRun>(
    request,
    'POST',
    '/api/v1/command-runs',
    {
      request_id: createUuid(),
      command: {
        name: command.name,
        program: command.program,
        args: command.args,
        timeout_secs: command.timeoutSecs,
        cancelable: command.cancelable,
        risk: command.risk,
      },
    },
  )
}

export async function cancelCommandRun(request: AgentRequest, runId: string): Promise<CommandRun> {
  return await agentJson<CommandRun>(
    request,
    'POST',
    `/api/v1/command-runs/${encodeURIComponent(runId)}/cancel`,
    {},
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

export function createCommandId(): string {
  return `cmd-${createUuid()}`
}

function quoteArg(value: string) {
  if (value !== '' && !/[\s"]/.test(value))
    return value
  return `"${value.replace(/(\\*)"/g, '$1$1\\"').replace(/(\\+)$/, '$1$1')}"`
}

export function formatDisplayCommand(command: Pick<AppCommand, 'program' | 'args'>): string {
  return [command.program, ...command.args].map(quoteArg).join(' ')
}

const COMMANDS_STORAGE_KEY = 'computerCustomCommandsV1'
const TOKENS_STORAGE_KEY = 'wolAgentCommandTokensV1'

interface StoredCommandFile {
  version: 1
  devices: Record<string, AppCommand[]>
}

interface StoredTokenFile {
  version: 1
  devices: Record<string, string>
}

function readStorage<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key)
    if (!raw)
      return fallback
    return JSON.parse(raw) as T
  }
  catch {
    return fallback
  }
}

export function loadDeviceCommands(deviceId: string): AppCommand[] {
  const file = readStorage<StoredCommandFile>(COMMANDS_STORAGE_KEY, { version: 1, devices: {} })
  return Array.isArray(file.devices?.[deviceId]) ? file.devices[deviceId] : []
}

export function saveDeviceCommands(deviceId: string, commands: AppCommand[]): void {
  const file = readStorage<StoredCommandFile>(COMMANDS_STORAGE_KEY, { version: 1, devices: {} })
  file.version = 1
  file.devices = file.devices || {}
  file.devices[deviceId] = commands
  localStorage.setItem(COMMANDS_STORAGE_KEY, JSON.stringify(file))
}

export function loadDeviceCommandToken(deviceId: string): string {
  const file = readStorage<StoredTokenFile>(TOKENS_STORAGE_KEY, { version: 1, devices: {} })
  return file.devices?.[deviceId] || ''
}

export function saveDeviceCommandToken(deviceId: string, token: string): void {
  const file = readStorage<StoredTokenFile>(TOKENS_STORAGE_KEY, { version: 1, devices: {} })
  file.version = 1
  file.devices = file.devices || {}
  if (token)
    file.devices[deviceId] = token
  else delete file.devices[deviceId]
  localStorage.setItem(TOKENS_STORAGE_KEY, JSON.stringify(file))
}
