const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const {
  mkdtempSync,
  readFileSync,
  rmSync,
} = require('node:fs');
const { tmpdir } = require('node:os');
const path = require('node:path');
const { after, beforeEach, describe, test } = require('node:test');

const projectRoot = path.resolve(__dirname, '..');
const trackerPath = path.join(projectRoot, 'src', 'engine', 'EyeTracker.ts');
const psyDuckPath = path.join(
  projectRoot,
  'src',
  'renderer',
  'components',
  'PsyDuck.tsx',
);
const compiledRoot = mkdtempSync(path.join(tmpdir(), 'ducky-eye-tracker-'));

execFileSync(
  path.join(projectRoot, 'node_modules', '.bin', 'tsc'),
  [
    '--ignoreConfig',
    trackerPath,
    '--target',
    'ES2022',
    '--module',
    'Node16',
    '--moduleResolution',
    'Node16',
    '--lib',
    'ES2022,DOM',
    '--skipLibCheck',
    '--outDir',
    compiledRoot,
  ],
  { cwd: projectRoot, stdio: 'pipe' },
);

const { EyeTracker } = require(
  path.join(compiledRoot, 'engine', 'EyeTracker.js'),
);
const psyDuckSource = readFileSync(psyDuckPath, 'utf8');

after(() => {
  rmSync(compiledRoot, { recursive: true, force: true });
});

let animationFrameCallbacks;

beforeEach(() => {
  animationFrameCallbacks = [];
  global.requestAnimationFrame = (callback) => {
    animationFrameCallbacks.push(callback);
    return animationFrameCallbacks.length;
  };
  global.cancelAnimationFrame = () => {};
});

const flushPromises = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

const runAnimationFrames = () => {
  let timestamp = 0;

  while (animationFrameCallbacks.length > 0) {
    const callback = animationFrameCallbacks.shift();
    timestamp += 1_000 / 60;
    callback(timestamp);
  }
};

const createCursorSource = (initialPosition) => {
  const listeners = new Set();

  return {
    source: {
      getCurrentPosition: async () => initialPosition,
      subscribe: (listener) => {
        listeners.add(listener);
        return () => listeners.delete(listener);
      },
    },
    emit(position) {
      for (const listener of listeners) {
        listener(position);
      }
    },
    get listenerCount() {
      return listeners.size;
    },
  };
};

describe('EyeTracker native window origin', () => {
  test('does not use unreliable WKWebView screen coordinates', () => {
    assert.doesNotMatch(psyDuckSource, /\bwindow\.screen[XY]\b/);
    assert.match(
      psyDuckSource,
      /companionWindowBridge\.getWindowPosition\(\)/,
    );
  });

  test('waits for the native logical eye origin before tracking the cursor', async () => {
    let resolveEyeOrigin;
    const eyeOrigin = new Promise((resolve) => {
      resolveEyeOrigin = resolve;
    });
    const cursor = createCursorSource({ x: 540, y: 340 });
    const offsets = [];
    const tracker = new EyeTracker({
      cursorSource: cursor.source,
      getEyeOrigin: () => eyeOrigin,
      onOffsetChange: (offset) => offsets.push(offset),
      normalizationDistance: 100,
      smoothing: 1,
    });

    tracker.start();
    cursor.emit({ x: 640, y: 340 });

    assert.equal(cursor.listenerCount, 0);

    resolveEyeOrigin({ x: 540, y: 340 });
    await flushPromises();
    runAnimationFrames();

    assert.equal(cursor.listenerCount, 1);
    assert.deepEqual(offsets.at(-1), { x: 0, y: 0 });

    cursor.emit({ x: 640, y: 340 });
    runAnimationFrames();

    assert.deepEqual(offsets.at(-1), { x: 1, y: 0 });
  });

  test('refreshes the native origin after a drag without accepting a stale read', async () => {
    const pendingOrigins = [];
    const cursor = createCursorSource({ x: 0, y: 0 });
    const offsets = [];
    const tracker = new EyeTracker({
      cursorSource: cursor.source,
      getEyeOrigin: () =>
        new Promise((resolve) => pendingOrigins.push(resolve)),
      onOffsetChange: (offset) => offsets.push(offset),
      normalizationDistance: 100,
      smoothing: 1,
    });

    tracker.start();
    tracker.stop();
    tracker.start();

    pendingOrigins[0]({ x: -500, y: -500 });
    pendingOrigins[1]({ x: 300, y: 200 });
    await flushPromises();

    cursor.emit({ x: 350, y: 200 });
    runAnimationFrames();

    assert.equal(cursor.listenerCount, 1);
    assert.deepEqual(offsets.at(-1), { x: 0.5, y: 0 });
  });
});
