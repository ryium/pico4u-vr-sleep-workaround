import { existsSync } from 'node:fs'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

// Tauri runs hooks through a shell, whose profile can restore a broken global
// pnpm shim. Resolve the same local native binary used by scripts/pnpm.ps1.
const root = fileURLToPath(new URL('../', import.meta.url))
const local = path.join(root, '.tools/node_modules/@pnpm/exe.win32-x64/pnpm.exe')
const command = process.argv[2]
if (command !== 'dev' && command !== 'build') throw new Error('Expected dev or build')
const manager = process.platform === 'win32' && existsSync(local)
  ? local : process.env.npm_execpath
const isScript = manager && /\.[cm]?js$/.test(manager)
const child = spawn(isScript ? process.execPath : manager || 'pnpm',
  [...(isScript ? [manager] : []), '-C', 'frontend', command],
  { cwd: root, stdio: 'inherit', shell: !manager && process.platform === 'win32' })
child.on('error', error => { console.error(error.message); process.exitCode = 1 })
child.on('exit', code => { process.exitCode = code ?? 1 })
