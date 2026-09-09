#!/usr/bin/env node

/**
 * Runs the independent pre-bundle build pipelines in parallel:
 *   - build:web            (type-check + Vite build + revision manifest + asset verification)
 *   - prepare:mobile-web   (mobile-web install/build with mtime short-circuit)
 *   - prepare:dsh-profile  (the DeepSeek Harness bridge official desktop:build ships)
 *
 * Used as the Tauri beforeBuildCommand so the stage costs max(…) instead of
 * their sum. Any failure fails the whole script with a non-zero exit code.
 *
 * Each pipeline's wall-clock cost is written to `gen/frontend-timings.json` so
 * the packaging wrapper can report where the time went.
 */

import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  createStageRecorder,
  readStageTimings,
  writeStageTimings,
} from './build-stage-timings.mjs';

const ROOT_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const GEN_DIR = path.join(ROOT_DIR, 'src', 'apps', 'desktop', 'gen');
const TIMINGS_PATH = path.join(GEN_DIR, 'frontend-timings.json');
const WEB_TIMINGS_PATH = path.join(GEN_DIR, 'web-timings.json');

/**
 * The `build:web` audits (appearance contract, Monaco assets, WebKit
 * compatibility) only validate a finished `dist/`; they produce nothing and are
 * enforced by CI. `--skip-audits` local packaging drops them.
 */
const skipAudits = ['1', 'true', 'yes'].includes(
  String(process.env.OPENBITFUN_SKIP_AUDITS ?? '').toLowerCase()
);
const webScript = skipAudits ? 'build:web:no-audit' : 'build:web';

function runPrefixed(prefix, command, args) {
  return new Promise((resolve) => {
    const child = spawn(command, args, {
      cwd: ROOT_DIR,
      shell: process.platform === 'win32',
      stdio: ['ignore', 'pipe', 'pipe'],
      env: process.env,
    });

    const forward = (stream, out) => {
      let buffered = '';
      stream.on('data', (chunk) => {
        buffered += chunk.toString();
        const lines = buffered.split(/\r?\n/);
        buffered = lines.pop() ?? '';
        for (const line of lines) {
          out.write(`[${prefix}] ${line}\n`);
        }
      });
      stream.on('end', () => {
        if (buffered.trim() !== '') {
          out.write(`[${prefix}] ${buffered}\n`);
        }
      });
    };

    forward(child.stdout, process.stdout);
    forward(child.stderr, process.stderr);

    child.on('error', (error) => {
      process.stderr.write(`[${prefix}] failed to start: ${error.message}\n`);
      resolve(1);
    });
    child.on('close', (code) => {
      resolve(code ?? 1);
    });
  });
}

const startedAtMs = Date.now();
if (skipAudits) {
  process.stdout.write(`[frontend-build-all] OPENBITFUN_SKIP_AUDITS=1: running ${webScript}\n`);
}
const recorder = createStageRecorder();
const codes = await Promise.all([
  recorder.time('web', () => runPrefixed('web', 'pnpm', ['run', webScript])),
  recorder.time('mobile-web', () => runPrefixed('mobile-web', 'pnpm', ['run', 'prepare:mobile-web'])),
  // The DeepSeek Harness bridge Tauri ships as a resource. On a cold tree this
  // installs its own pinned toolchain (~30s), which still fits inside the two
  // above; it is independent of them, and of OpenBitFun's pnpm store.
  recorder.time('dsh-profile', () => runPrefixed('dsh-profile', 'pnpm', ['run', 'prepare:dsh-profile'])),
]);

// `build-web-parallel.mjs` records its own steps; inline them under `web/`.
const webStages = readStageTimings(WEB_TIMINGS_PATH, { notBeforeMs: startedAtMs - 1_000 }).map(
  (stage) => ({ name: `web/${stage.name}`, ms: stage.ms })
);
const stages = [];
for (const stage of recorder.stages) {
  stages.push(stage);
  if (stage.name === 'web') {
    stages.push(...webStages);
  }
}
writeStageTimings(TIMINGS_PATH, stages, { totalMs: Date.now() - startedAtMs });

const failed = codes.some((code) => code !== 0);
if (failed) {
  process.stderr.write('[frontend-build-all] frontend build failed (see output above)\n');
}
// Set the code instead of calling process.exit(): stdout is a pipe under CI and
// process.exit() would drop whatever is still queued on it.
process.exitCode = failed ? 1 : 0;
