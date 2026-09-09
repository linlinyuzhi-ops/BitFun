#!/usr/bin/env node

/**
 * Shared stage timing recorder for the packaging pipelines.
 *
 * `tauri build` hides the frontend pipeline behind one opaque stage, so the
 * nested costs are written under `src/apps/desktop/gen/` (git-ignored) and the
 * packaging wrapper merges them back into the summary it prints at the end.
 * Timings are diagnostics: failing to read or write them never fails a build.
 */
import { mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

export function createStageRecorder(now = () => Date.now()) {
  const stages = [];
  return {
    stages,
    /** Awaits `task`, records its wall-clock cost under `name`, returns its result. */
    async time(name, task) {
      const startedAt = now();
      const result = await task();
      stages.push({ name, ms: Math.max(0, now() - startedAt) });
      return result;
    },
  };
}

export function writeStageTimings(timingsPath, stages, { totalMs } = {}) {
  try {
    mkdirSync(dirname(timingsPath), { recursive: true });
    writeFileSync(
      timingsPath,
      `${JSON.stringify({ generatedAt: new Date().toISOString(), totalMs, stages }, null, 2)}\n`,
      'utf8'
    );
  } catch (error) {
    process.stderr.write(`[stage-timings] could not write ${timingsPath}: ${error.message}\n`);
  }
}

/**
 * Reads stages written by an earlier step. `notBeforeMs` rejects a file left over
 * from a previous run, which would otherwise report stages this build never ran.
 */
export function readStageTimings(
  timingsPath,
  { notBeforeMs, stat = statSync, read = readFileSync } = {}
) {
  try {
    if (notBeforeMs !== undefined && stat(timingsPath).mtimeMs < notBeforeMs) {
      return [];
    }
    const parsed = JSON.parse(read(timingsPath, 'utf8'));
    if (!Array.isArray(parsed?.stages)) {
      return [];
    }
    return parsed.stages.filter(
      (stage) => typeof stage?.name === 'string' && Number.isFinite(stage?.ms)
    );
  } catch {
    return [];
  }
}
