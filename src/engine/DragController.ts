import type { ScreenPoint } from '../shared/types';

export interface DragControllerOptions {
  readonly surface: HTMLElement;
  readonly getWindowPosition: () => Promise<ScreenPoint>;
  readonly moveWindow: (position: ScreenPoint) => void;
  readonly onDraggingChange?: (dragging: boolean) => void;
}

export class DragController {
  private readonly surface: HTMLElement;
  private readonly getWindowPosition: () => Promise<ScreenPoint>;
  private readonly moveWindow: (position: ScreenPoint) => void;
  private readonly onDraggingChange: ((dragging: boolean) => void) | undefined;
  private dragAnchor: ScreenPoint | null = null;
  private dragWindowPosition: ScreenPoint | null = null;
  private pendingPointerPosition: ScreenPoint | null = null;
  private lastWindowPosition: ScreenPoint | null = null;
  private activePointerId: number | null = null;
  private dragGeneration = 0;
  private attached = false;
  private dragging = false;

  public constructor(options: DragControllerOptions) {
    this.surface = options.surface;
    this.getWindowPosition = options.getWindowPosition;
    this.moveWindow = options.moveWindow;
    this.onDraggingChange = options.onDraggingChange;
  }

  public get isDragging(): boolean {
    return this.dragging;
  }

  public start(): void {
    if (this.attached) {
      return;
    }

    this.surface.addEventListener('pointerdown', this.handlePointerDown);
    this.surface.addEventListener('lostpointercapture', this.handleLostPointerCapture);
    this.attached = true;
  }

  public stop(): void {
    if (!this.attached) {
      return;
    }

    this.endDrag();
    this.surface.removeEventListener('pointerdown', this.handlePointerDown);
    this.surface.removeEventListener(
      'lostpointercapture',
      this.handleLostPointerCapture,
    );
    this.attached = false;
  }

  private readonly handlePointerDown = (event: PointerEvent): void => {
    if (this.dragging || event.button !== 0 || !event.isPrimary) {
      return;
    }

    event.preventDefault();

    this.activePointerId = event.pointerId;
    this.dragging = true;
    this.dragAnchor = null;
    this.dragWindowPosition = null;
    this.pendingPointerPosition = null;
    this.lastWindowPosition = null;
    const generation = ++this.dragGeneration;

    this.surface.setPointerCapture(event.pointerId);
    this.addActiveListeners();
    this.onDraggingChange?.(true);

    void this.initializeDragAnchor(
      generation,
      event.clientX,
      event.clientY,
    );
  };

  private readonly handlePointerMove = (event: PointerEvent): void => {
    if (!this.isActivePointer(event)) {
      return;
    }

    if ((event.buttons & 1) === 0) {
      this.endDrag();
      return;
    }

    const pointerPosition = {
      x: event.screenX,
      y: event.screenY,
    };
    this.pendingPointerPosition = pointerPosition;

    this.moveToPointerPosition(pointerPosition);
  };

  private moveToPointerPosition(pointerPosition: ScreenPoint): void {
    if (
      this.dragAnchor === null ||
      this.dragWindowPosition === null
    ) {
      return;
    }

    const grabOffset = {
      x: this.dragAnchor.x - this.dragWindowPosition.x,
      y: this.dragAnchor.y - this.dragWindowPosition.y,
    };
    const nextWindowPosition = {
      x: pointerPosition.x - grabOffset.x,
      y: pointerPosition.y - grabOffset.y,
    };

    if (
      nextWindowPosition.x === this.lastWindowPosition?.x &&
      nextWindowPosition.y === this.lastWindowPosition.y
    ) {
      return;
    }

    this.lastWindowPosition = nextWindowPosition;
    this.moveWindow(nextWindowPosition);
  }

  private async initializeDragAnchor(
    generation: number,
    clientX: number,
    clientY: number,
  ): Promise<void> {
    try {
      const windowPosition = await this.getWindowPosition();

      if (!this.dragging || generation !== this.dragGeneration) {
        return;
      }

      this.dragWindowPosition = windowPosition;
      this.dragAnchor = {
        x: windowPosition.x + clientX,
        y: windowPosition.y + clientY,
      };
      this.lastWindowPosition = windowPosition;

      if (this.pendingPointerPosition !== null) {
        this.moveToPointerPosition(this.pendingPointerPosition);
      }
    } catch (error) {
      if (this.dragging && generation === this.dragGeneration) {
        console.error(
          '[drag] Unable to read the native companion window position.',
          error,
        );
        this.endDrag();
      }
    }
  }

  private readonly handlePointerUp = (event: PointerEvent): void => {
    if (this.isActivePointer(event)) {
      this.endDrag();
    }
  };

  private readonly handleLostPointerCapture = (event: PointerEvent): void => {
    if (this.isActivePointer(event)) {
      this.endDrag();
    }
  };

  private readonly handleWindowBlur = (): void => {
    this.endDrag();
  };

  private isActivePointer(event: PointerEvent): boolean {
    return this.dragging && event.pointerId === this.activePointerId;
  }

  private addActiveListeners(): void {
    const ownerWindow = this.surface.ownerDocument.defaultView;

    ownerWindow?.addEventListener('pointermove', this.handlePointerMove);
    ownerWindow?.addEventListener('pointerup', this.handlePointerUp);
    ownerWindow?.addEventListener('pointercancel', this.handlePointerUp);
    ownerWindow?.addEventListener('blur', this.handleWindowBlur);
  }

  private removeActiveListeners(): void {
    const ownerWindow = this.surface.ownerDocument.defaultView;

    ownerWindow?.removeEventListener('pointermove', this.handlePointerMove);
    ownerWindow?.removeEventListener('pointerup', this.handlePointerUp);
    ownerWindow?.removeEventListener('pointercancel', this.handlePointerUp);
    ownerWindow?.removeEventListener('blur', this.handleWindowBlur);
  }

  private endDrag(): void {
    if (!this.dragging) {
      return;
    }

    const pointerId = this.activePointerId;
    this.dragging = false;
    this.activePointerId = null;
    this.dragGeneration += 1;
    this.dragAnchor = null;
    this.dragWindowPosition = null;
    this.pendingPointerPosition = null;
    this.lastWindowPosition = null;
    this.removeActiveListeners();

    if (pointerId !== null && this.surface.hasPointerCapture(pointerId)) {
      this.surface.releasePointerCapture(pointerId);
    }

    this.onDraggingChange?.(false);
  }
}
