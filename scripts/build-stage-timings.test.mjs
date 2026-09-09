import assert from 'node:assert/strict';
import { mkdirSync, rmSync, utimesSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import {
  createStageRecorder,
  readStageTimings,
  writeStageTimings,
} from './build-stage-timings.mjs';

function fixture() {
  const root = join(tmpdir(), `openbitfun-stage-timings-${process.pid}-${Date.now()}`);
  mkdirSync(root, { recursive: true });
  return {
    root,
    timingsPath: join(root, 'timings.json'),
    cleanup: () => rmSync(root, { force: true, recursive: true }),
  };
}

test('stage timings are written into a directory that does not exist yet', () => {
  const { root, cleanup } = fixture();
  const timingsPath = join(root, 'gen', 'frontend-timings.json');
  try {
    writeStageTimings(timingsPath, [{ name: 'web', ms: 1200 }], { totalMs: 1500 });
    assert.deepEqual(readStageTimings(timingsPath), [{ name: 'web', ms: 1200 }]);
  } finally {
    cleanup();
  }
});

test('a timings file left over from an earlier run is ignored', () => {
  const { timingsPath, cleanup } = fixture();
  try {
    writeStageTimings(timingsPath, [{ name: 'web', ms: 1200 }], { totalMs: 1500 });
    const stale = new Date(Date.now() - 60_000);
    utimesSync(timingsPath, stale, stale);
    assert.deepEqual(readStageTimings(timingsPath, { notBeforeMs: Date.now() }), []);
    assert.deepEqual(readStageTimings(timingsPath, { notBeforeMs: 0 }), [
      { name: 'web', ms: 1200 },
    ]);
  } finally {
    cleanup();
  }
});

test('unreadable or malformed timings degrade to no stages', () => {
  const { root, timingsPath, cleanup } = fixture();
  try {
    assert.deepEqual(readStageTimings(join(root, 'missing.json')), []);

    writeFileSync(timingsPath, 'not json', 'utf8');
    assert.deepEqual(readStageTimings(timingsPath), []);

    writeFileSync(
      timingsPath,
      JSON.stringify({ stages: [{ name: 'web', ms: 5 }, { name: 'broken' }, 'nope'] }),
      'utf8'
    );
    assert.deepEqual(readStageTimings(timingsPath), [{ name: 'web', ms: 5 }]);
  } finally {
    cleanup();
  }
});

test('a failed timing write does not fail the build that reports it', () => {
  const { root, cleanup } = fixture();
  try {
    // A regular file where the parent directory should be makes the write throw.
    const blocked = join(root, 'blocked');
    writeFileSync(blocked, 'not a directory', 'utf8');
    const target = join(blocked, 'nested', 'timings.json');
    const notices = [];
    const originalWrite = process.stderr.write.bind(process.stderr);
    process.stderr.write = (line) => {
      notices.push(String(line));
      return true;
    };
    try {
      writeStageTimings(target, [{ name: 'web', ms: 1 }]);
    } finally {
      process.stderr.write = originalWrite;
    }
    assert.match(notices.join(''), /stage-timings/);
    assert.deepEqual(readStageTimings(target), []);
  } finally {
    cleanup();
  }
});

test('the recorder measures each task and passes its result through', async () => {
  let clock = 0;
  const recorder = createStageRecorder(() => clock);

  const code = await recorder.time('web', async () => {
    clock += 500;
    return 0;
  });
  assert.equal(code, 0);

  clock = 1200;
  await recorder.time('mobile-web', async () => {
    clock = 1900;
    return 1;
  });

  assert.deepEqual(recorder.stages, [
    { name: 'web', ms: 500 },
    { name: 'mobile-web', ms: 700 },
  ]);
});
