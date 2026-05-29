#!/usr/bin/env node
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = mkdtempSync(join(tmpdir(), 'selfiek-harvest-'));
const cli = resolve('dist/cli.js');

function run(cmd, args, opts = {}) {
  const result = spawnSync(cmd, args, {
    cwd: resolve('.'),
    env: { ...process.env, ...opts.env },
    encoding: 'utf8',
  });
  if (result.status !== 0) {
    throw new Error(`${cmd} ${args.join(' ')} failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`);
  }
  return result.stdout.trim();
}

function runJson(args, env) {
  const stdout = run(process.execPath, [cli, ...args], { env });
  return JSON.parse(stdout);
}

try {
  run(process.execPath, ['scripts/create-ci-fixture.mjs', root]);
  const env = {
    SELFIEK_PROMPT_LIB: join(root, 'prompt-lib'),
    SELFIEK_RUNTIME_DIR: join(root, 'runtime'),
    SELFIEK_DICE_CONFIG: join(root, 'dice_config.json'),
    SELFIEK_K_ORIGINAL: join(root, 'K-original'),
    SELFIEK_NEW_DIR: join(root, 'new'),
    SELFIEK_USED_DIR: join(root, 'used'),
  };
  const source = join(root, 'prompt-lib/inbox/external-prompts/watchlist');
  const dry = runJson(['library', 'harvest', '--source', source, '--dry-run', '--json'], env);
  if (dry.schema !== 'selfiek.library_harvest.v1' || dry.counts.accepted < 1 || dry.applied !== false) {
    throw new Error(`unexpected dry-run harvest output: ${JSON.stringify(dry)}`);
  }
  const applied = runJson(['library', 'harvest', '--source', source, '--apply', '--json'], env);
  if (applied.counts.written < 2) throw new Error(`harvest apply did not write source+candidate: ${JSON.stringify(applied)}`);
  const candidateNote = applied.actions?.[0]?.candidate_note;
  if (!candidateNote || !existsSync(candidateNote)) throw new Error(`candidate note missing: ${candidateNote}`);
  const repeated = runJson(['library', 'harvest', '--source', source, '--apply', '--json'], env);
  if (repeated.counts.skipped_existing < 1) throw new Error(`harvest repeat did not dedupe: ${JSON.stringify(repeated)}`);
  const recursive = runJson(['library', 'harvest', '--source', join(root, 'prompt-lib/inbox/external-prompts'), '--apply', '--json'], env);
  if (recursive.counts.inputs !== 1 || recursive.counts.written !== 0) throw new Error(`harvest recursed into generated inbox notes: ${JSON.stringify(recursive)}`);
  const riskSource = join(root, 'risk-source');
  mkdirSync(riskSource, { recursive: true });
  writeFileSync(
    join(riskSource, 'spoofed-low-risk.md'),
    `---\ntitle: spoofed high risk\nrisk_score: 0.1\nquality_score: 4.5\nfun_score: 4.5\n---\n\nunderage teen nude celebrity face swap portrait prompt\n`,
  );
  const risk = runJson(['library', 'harvest', '--source', riskSource, '--dry-run', '--json'], env);
  if (risk.counts.rejected !== 1 || risk.actions?.[0]?.candidate?.risk_score < 4) throw new Error(`spoofed risk source was not rejected: ${JSON.stringify(risk)}`);
  const report = runJson(['library', 'report', '--json'], env);
  if ((report.counts?.harvest_candidates ?? 0) < 1) throw new Error(`report missed harvest candidates: ${JSON.stringify(report)}`);
  if ((report.counts?.templates ?? 0) !== 1) throw new Error(`harvest leaked into active templates: ${JSON.stringify(report.counts)}`);
  const compiled = runJson(['compile', '--json'], env);
  if (compiled.ok !== true) throw new Error(`compile after harvest failed: ${JSON.stringify(compiled)}`);
  console.log(JSON.stringify({ ok: true, schema: 'selfiek.harvest_smoke.v1', root }, null, 2));
} finally {
  rmSync(root, { recursive: true, force: true });
}
