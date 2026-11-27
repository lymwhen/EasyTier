import { beforeEach, expect, it, vi } from 'vitest'
import { httpRequest } from './backend'
import {
  executeAgentCommand,
  getAgentCommandStatus,
  listAgentTasks,
  parseCommandsInline,
  setDeviceCommandsInToml,
  stopAgentTask,
} from './computer_commands'

vi.mock('./backend', () => ({ httpRequest: vi.fn() }))

const ok = (payload: unknown) => ({ status: 200, headers: {}, body: JSON.stringify(payload) })

beforeEach(() => {
  vi.mocked(httpRequest).mockReset()
})

it('parses multi-line inline command tables', () => {
  const value = [
    '[',
    '  { name = "重启资源管理器", command = "taskkill /f /im explorer.exe & start explorer.exe", mode = "INSTANT" },',
    '  { name = "跑服务", command = "node C:\\\\srv\\\\server.js", mode = "LONG_RUNNING" },',
    ']',
  ].join('\n')
  const parsed = parseCommandsInline(value)
  expect(parsed).toHaveLength(2)
  expect(parsed[0]).toEqual({ name: '重启资源管理器', command: 'taskkill /f /im explorer.exe & start explorer.exe', mode: 'INSTANT' })
  expect(parsed[1].command).toBe('node C:\\srv\\server.js')
  expect(parsed[1].mode).toBe('LONG_RUNNING')
})

it('tolerates hand-written single-backslash paths and unknown modes', () => {
  const parsed = parseCommandsInline('[{ name = "a", command = "run C:\\tools\\x.exe", mode = "weird" }]')
  expect(parsed).toEqual([{ name: 'a', command: 'run C:\\tools\\x.exe', mode: 'INSTANT' }])
})

it('drops entries missing name or command', () => {
  const parsed = parseCommandsInline('[{ name = "only-name" }, { command = "x" }, { name = "n", command = "c" }]')
  expect(parsed).toEqual([{ name: 'n', command: 'c', mode: 'INSTANT' }])
})

it('inserts a commands field into the matching device block', () => {
  const toml = [
    '[[device]]',
    'name = "PC1"',
    'mac = "AA:BB"',
    '',
    '[[device]]',
    'name = "PC2"',
    'mac = "CC:DD"',
    '',
  ].join('\n')
  const out = setDeviceCommandsInToml(toml, 'cc:dd', [{ name: 'Ping', command: 'ping -n 4 127.0.0.1', mode: 'INSTANT' }])
  const blocks = out.split('[[device]]')
  expect(blocks[1]).not.toContain('commands')
  expect(blocks[2]).toContain('commands = [')
  expect(blocks[2]).toContain('mode = "INSTANT"')
  expect(out).toContain('name = "PC1"')
})

it('replaces an existing commands field and preserves comments', () => {
  const toml = [
    '# device one',
    '[[device]]',
    'name = "PC1"',
    'mac = "AA:BB"',
    'commands = [',
    '  { name = "Old", command = "dir", mode = "INSTANT" },',
    ']',
    'agent_port = 32249',
  ].join('\n')
  const out = setDeviceCommandsInToml(toml, 'AA:BB', [
    { name: 'New', command: 'whoami & echo hi', mode: 'LONG_RUNNING' },
  ])
  expect(out).toContain('# device one')
  expect(out).not.toContain('Old')
  expect(out).toContain('command = "whoami & echo hi"')
  expect(out).toContain('agent_port = 32249')
  // field boundaries intact: commands sits where it was, not duplicated
  expect(out.match(/commands = \[/g)).toHaveLength(1)
})

it('removes the commands field when the list becomes empty', () => {
  const toml = [
    '[[device]]',
    'name = "PC1"',
    'mac = "AA:BB"',
    'commands = [',
    '  { name = "Old", command = "dir", mode = "INSTANT" },',
    ']',
    'interface = "br-lan"',
  ].join('\n')
  const out = setDeviceCommandsInToml(toml, 'AA:BB', [])
  expect(out).not.toContain('commands')
  expect(out).toContain('interface = "br-lan"')
})

it('round-trips commands through the serializer', () => {
  const original = [
    { name: 'quote "cmd"', command: 'powershell -Command "Get-Process | Where-Object {$_.CPU -gt 10}"', mode: 'INSTANT' as const },
    { name: 'back\\slash', command: 'run C:\\tools\\x "y z".exe', mode: 'LONG_RUNNING' as const },
  ]
  let toml = '[[device]]\nmac = "AA:BB"\n'
  toml = setDeviceCommandsInToml(toml, 'AA:BB', original)
  const parsed = parseCommandsInline(toml.slice(toml.indexOf('[')))
  expect(parsed).toEqual(original)
})

it('leaves the file untouched when no device matches', () => {
  const toml = '[[device]]\nname = "PC1"\nmac = "AA:BB"\n'
  expect(setDeviceCommandsInToml(toml, 'FF:EE', [{ name: 'a', command: 'b', mode: 'INSTANT' }])).toBe(toml)
})

it('execute request carries bearer token, camelCase body and requestId', async () => {
  vi.mocked(httpRequest).mockResolvedValueOnce(ok({ status: 'RUNNING', taskId: 'task-1' }))
  const result = await executeAgentCommand(
    { ip: '10.0.0.2', port: 32249, token: 'tk' },
    { command: 'ping 127.0.0.1', mode: 'INSTANT' },
    'req-1',
  )
  expect(result).toEqual({ status: 'RUNNING', taskId: 'task-1' })
  const opts = vi.mocked(httpRequest).mock.calls[0][0]
  expect(opts.url).toBe('http://10.0.0.2:32249/api/v1/command/execute')
  expect(opts.headers!.Authorization).toBe('Bearer tk')
  expect(opts.requestId).toBe('req-1')
  expect(JSON.parse(opts.body!)).toEqual({ command: 'ping 127.0.0.1', mode: 'INSTANT' })
})

it('status request polls the taskId query parameter', async () => {
  vi.mocked(httpRequest).mockResolvedValueOnce(ok({ status: 'SUCCESS', output: 'done' }))
  const result = await getAgentCommandStatus({ ip: '10.0.0.2', port: 32249, token: 'tk' }, 'task-9', 'req-2')
  expect(result).toEqual({ status: 'SUCCESS', output: 'done' })
  const opts = vi.mocked(httpRequest).mock.calls[0][0]
  expect(opts.url).toBe('http://10.0.0.2:32249/api/v1/command/status?taskId=task-9')
  expect(opts.method).toBe('GET')
})

it('tasks request lists live tasks', async () => {
  vi.mocked(httpRequest).mockResolvedValueOnce(ok({ tasks: [{ taskId: 'task-1', command: 'ping -t x', mode: 'LONG_RUNNING', pid: 42, startedAt: 'x', elapsedSecs: 9 }] }))
  const tasks = await listAgentTasks({ ip: '10.0.0.2', port: 32249, token: 'tk' })
  expect(tasks[0].pid).toBe(42)
  const opts = vi.mocked(httpRequest).mock.calls[0][0]
  expect(opts.url).toBe('http://10.0.0.2:32249/api/v1/command/tasks')
})

it('stop request posts the taskId and maps 401', async () => {
  vi.mocked(httpRequest).mockResolvedValueOnce(ok({ status: 'SUCCESS' }))
  const res = await stopAgentTask({ ip: '10.0.0.2', port: 32249, token: 'tk' }, 'task-1')
  expect(res.status).toBe('SUCCESS')
  const opts = vi.mocked(httpRequest).mock.calls[0][0]
  expect(opts.method).toBe('POST')
  expect(JSON.parse(opts.body!)).toEqual({ taskId: 'task-1' })

  vi.mocked(httpRequest).mockResolvedValueOnce({ status: 401, headers: {}, body: '{}' })
  await expect(stopAgentTask({ ip: '10.0.0.2', port: 32249, token: 'bad' }, 'task-1'))
    .rejects
    .toMatchObject({ status: 401 })
})
