const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const {
  mkdtempSync,
  readFileSync,
  rmSync,
} = require('node:fs');
const { tmpdir } = require('node:os');
const path = require('node:path');
const { after, describe, test } = require('node:test');

const projectRoot = path.resolve(__dirname, '..');
const controllerPath = path.join(
  projectRoot,
  'src',
  'engine',
  'DragController.ts',
);
const controllerSource = readFileSync(controllerPath, 'utf8');
const compiledRoot = mkdtempSync(
  path.join(tmpdir(), 'ducky-drag-controller-'),
);

const loadDragController = () => {
  execFileSync(
    path.join(projectRoot, 'node_modules', '.bin', 'tsc'),
    [
      '--ignoreConfig',
      controllerPath,
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

  return require(path.join(compiledRoot, 'engine', 'DragController.js'))
    .DragController;
};

const DragController = loadDragController();

after(() => {
  rmSync(compiledRoot, { recursive: true, force: true });
});

class FakeEventTarget {
  constructor() {
    this.listeners = new Map();
  }

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) ?? new Set();
    listeners.add(listener);
    this.listeners.set(type, listeners);
  }

  removeEventListener(type, listener) {
    this.listeners.get(type)?.delete(listener);
  }

  dispatch(type, event) {
    for (const listener of this.listeners.get(type) ?? []) {
      listener(event);
    }
  }
}

class FakeSurface extends FakeEventTarget {
  constructor(ownerWindow) {
    super();
    this.ownerDocument = { defaultView: ownerWindow };
    this.capturedPointer = null;
  }

  setPointerCapture(pointerId) {
    this.capturedPointer = pointerId;
  }

  hasPointerCapture(pointerId) {
    return this.capturedPointer === pointerId;
  }

  releasePointerCapture(pointerId) {
    if (this.capturedPointer === pointerId) {
      this.capturedPointer = null;
    }
  }
}

const pointerEvent = ({
  pointerId = 1,
  clientX,
  clientY,
  screenX,
  screenY,
  buttons = 1,
}) => ({
  pointerId,
  clientX,
  clientY,
  screenX,
  screenY,
  buttons,
  button: 0,
  isPrimary: true,
  preventDefault() {},
});

const flushPromises = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

describe('DragController native drag anchor', () => {
  test('preserves the exact grab point when WKWebView window screen coordinates are wrong', async () => {
    assert.doesNotMatch(controllerSource, /\bwindow\.screen[XY]\b/);

    global.window = {
      screenX: -80_000,
      screenY: 75_000,
    };

    try {
      const ownerWindow = new FakeEventTarget();
      const surface = new FakeSurface(ownerWindow);
      const movedPositions = [];
      const getWindowPositionCalls = [];
      const controller = new DragController({
        surface,
        getWindowPosition: async () => {
          getWindowPositionCalls.push(true);
          return { x: 420, y: 260 };
        },
        moveWindow: (position) => movedPositions.push(position),
      });

      controller.start();
      surface.dispatch(
        'pointerdown',
        pointerEvent({
          clientX: 35,
          clientY: 70,
          screenX: 455,
          screenY: 330,
        }),
      );
      await flushPromises();

      ownerWindow.dispatch(
        'pointermove',
        pointerEvent({
          clientX: 35,
          clientY: 70,
          screenX: 515,
          screenY: 395,
        }),
      );

      assert.equal(getWindowPositionCalls.length, 1);
      assert.deepEqual(movedPositions, [{ x: 480, y: 325 }]);
      assert.deepEqual(
        {
          x: movedPositions[0].x + 35,
          y: movedPositions[0].y + 70,
        },
        { x: 515, y: 395 },
      );
    } finally {
      delete global.window;
    }
  });

  test('does not accumulate anchor error across repeated drags', async () => {
    const ownerWindow = new FakeEventTarget();
    const surface = new FakeSurface(ownerWindow);
    const nativePositions = [
      { x: 100, y: 200 },
      { x: 180, y: 270 },
    ];
    const movedPositions = [];
    let nativePositionIndex = 0;
    const controller = new DragController({
      surface,
      getWindowPosition: async () =>
        nativePositions[nativePositionIndex++],
      moveWindow: (position) => movedPositions.push(position),
    });

    controller.start();
    surface.dispatch(
      'pointerdown',
      pointerEvent({
        pointerId: 1,
        clientX: 20,
        clientY: 30,
        screenX: 120,
        screenY: 230,
      }),
    );
    await flushPromises();
    ownerWindow.dispatch(
      'pointermove',
      pointerEvent({
        pointerId: 1,
        clientX: 20,
        clientY: 30,
        screenX: 200,
        screenY: 300,
      }),
    );
    ownerWindow.dispatch(
      'pointerup',
      pointerEvent({
        pointerId: 1,
        clientX: 20,
        clientY: 30,
        screenX: 200,
        screenY: 300,
        buttons: 0,
      }),
    );

    surface.dispatch(
      'pointerdown',
      pointerEvent({
        pointerId: 2,
        clientX: 45,
        clientY: 55,
        screenX: 225,
        screenY: 325,
      }),
    );
    await flushPromises();
    ownerWindow.dispatch(
      'pointermove',
      pointerEvent({
        pointerId: 2,
        clientX: 45,
        clientY: 55,
        screenX: 250,
        screenY: 350,
      }),
    );

    assert.equal(nativePositionIndex, 2);
    assert.deepEqual(movedPositions, [
      { x: 180, y: 270 },
      { x: 205, y: 295 },
    ]);
    assert.deepEqual(
      {
        x: movedPositions[1].x + 45,
        y: movedPositions[1].y + 55,
      },
      { x: 250, y: 350 },
    );
  });

  test('retains movement that occurs while the native position is loading', async () => {
    const ownerWindow = new FakeEventTarget();
    const surface = new FakeSurface(ownerWindow);
    const movedPositions = [];
    let resolveNativePosition;
    const nativePosition = new Promise((resolve) => {
      resolveNativePosition = resolve;
    });
    const controller = new DragController({
      surface,
      getWindowPosition: () => nativePosition,
      moveWindow: (position) => movedPositions.push(position),
    });

    controller.start();
    surface.dispatch(
      'pointerdown',
      pointerEvent({
        clientX: 25,
        clientY: 40,
        screenX: 325,
        screenY: 440,
      }),
    );
    ownerWindow.dispatch(
      'pointermove',
      pointerEvent({
        clientX: 25,
        clientY: 40,
        screenX: 355,
        screenY: 465,
      }),
    );

    assert.deepEqual(movedPositions, []);

    resolveNativePosition({ x: 300, y: 400 });
    await flushPromises();

    assert.deepEqual(movedPositions, [{ x: 330, y: 425 }]);
  });
});
