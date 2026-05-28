#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, '..');

function bundledBinary(): string | null {
  const platform = process.platform;
  const arch = process.arch;
  if (platform === 'linux' && arch === 'x64') {
    return resolve(pkgRoot, 'bin', 'selfiek-core-linux-x64');
  }
  return null;
}

function resolveCore(): string {
  if (process.env.SELFIEK_CORE_BIN && existsSync(process.env.SELFIEK_CORE_BIN)) return process.env.SELFIEK_CORE_BIN;
  const b = bundledBinary();
  if (b && existsSync(b)) return b;
  const repoCore = resolve(pkgRoot, '..', '..', 'target', 'release', process.platform === 'win32' ? 'selfiek-core.exe' : 'selfiek-core');
  if (existsSync(repoCore)) return repoCore;
  throw new Error('selfiek-core binary not found. Reinstall selfiek or set SELFIEK_CORE_BIN=/path/to/selfiek-core.');
}

function main(): void {
  const core = resolveCore();
  const result = spawnSync(core, process.argv.slice(2), { stdio: 'inherit', env: process.env });
  if (result.error) {
    console.error(`[selfiek] failed to run core: ${result.error.message}`);
    process.exit(1);
  }
  process.exit(result.status ?? 1);
}

main();
