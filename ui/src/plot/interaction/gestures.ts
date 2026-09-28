// Mouse and keyboard mapping (FR-009, FR-009a, FR-009e, FR-009k):
// drag = box zoom; Shift+drag or middle-drag = pan; wheel = zoom at pointer (1.2× per notch);
// double-click = reset; keyboard (when focused): arrows pan, +/- zoom, 0 resets.
import type { PlotController } from "../controller";
import { WHEEL_FACTOR } from "../viewState";

export interface BoxRect {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

const CLICK_PX = 5;
const KEY_PAN_FRACTION = 0.1;

export function attachGestures(
  element: HTMLElement,
  controller: PlotController,
  onBox: (box: BoxRect | null) => void,
): () => void {
  let drag: { mode: "box" | "pan"; x: number; y: number; lastX: number; lastY: number } | null =
    null;

  const local = (e: MouseEvent) => {
    const rect = element.getBoundingClientRect();
    return [e.clientX - rect.left, e.clientY - rect.top] as const;
  };

  const onPointerDown = (e: PointerEvent) => {
    if (e.button !== 0 && e.button !== 1) return;
    const [x, y] = local(e);
    const mode = e.button === 1 || e.shiftKey ? "pan" : "box";
    drag = { mode, x, y, lastX: x, lastY: y };
    element.setPointerCapture(e.pointerId);
    element.focus();
    e.preventDefault();
  };

  const onPointerMove = (e: PointerEvent) => {
    const [x, y] = local(e);
    if (!drag) {
      controller.hover(x, y);
      return;
    }
    if (drag.mode === "pan") {
      controller.panPx(x - drag.lastX, y - drag.lastY);
    } else if (Math.abs(x - drag.x) >= CLICK_PX || Math.abs(y - drag.y) >= CLICK_PX) {
      onBox({ x0: drag.x, y0: drag.y, x1: x, y1: y });
    }
    drag.lastX = x;
    drag.lastY = y;
  };

  const onPointerUp = (e: PointerEvent) => {
    if (!drag) return;
    const [x, y] = local(e);
    if (
      drag.mode === "box" &&
      (Math.abs(x - drag.x) >= CLICK_PX || Math.abs(y - drag.y) >= CLICK_PX)
    ) {
      controller.zoomRectPx(drag.x, drag.y, x, y);
    }
    onBox(null);
    drag = null;
    element.releasePointerCapture(e.pointerId);
  };

  const onWheel = (e: WheelEvent) => {
    e.preventDefault();
    const [x, y] = local(e);
    const notches = Math.max(-5, Math.min(5, e.deltaY / 100)) || Math.sign(e.deltaY);
    controller.zoomAtPx(x, y, WHEEL_FACTOR ** notches);
  };

  const onDoubleClick = () => {
    controller.reset();
  };

  const onLeave = () => {
    if (!drag) controller.clearHover();
  };

  const onKeyDown = (e: KeyboardEvent) => {
    const rect = element.getBoundingClientRect();
    const step = { x: rect.width * KEY_PAN_FRACTION, y: rect.height * KEY_PAN_FRACTION };
    const actions: Record<string, () => void> = {
      ArrowLeft: () => {
        controller.panPx(step.x, 0);
      },
      ArrowRight: () => {
        controller.panPx(-step.x, 0);
      },
      ArrowUp: () => {
        controller.panPx(0, step.y);
      },
      ArrowDown: () => {
        controller.panPx(0, -step.y);
      },
      "+": () => {
        controller.zoomAtPx(rect.width / 2, rect.height / 2, 1 / WHEEL_FACTOR);
      },
      "=": () => {
        controller.zoomAtPx(rect.width / 2, rect.height / 2, 1 / WHEEL_FACTOR);
      },
      "-": () => {
        controller.zoomAtPx(rect.width / 2, rect.height / 2, WHEEL_FACTOR);
      },
      "0": () => {
        controller.reset();
      },
    };
    const action = actions[e.key];
    if (action) {
      e.preventDefault();
      action();
    }
  };

  element.addEventListener("pointerdown", onPointerDown);
  element.addEventListener("pointermove", onPointerMove);
  element.addEventListener("pointerup", onPointerUp);
  element.addEventListener("pointercancel", onPointerUp);
  element.addEventListener("wheel", onWheel, { passive: false });
  element.addEventListener("dblclick", onDoubleClick);
  element.addEventListener("pointerleave", onLeave);
  element.addEventListener("keydown", onKeyDown);
  return () => {
    element.removeEventListener("pointerdown", onPointerDown);
    element.removeEventListener("pointermove", onPointerMove);
    element.removeEventListener("pointerup", onPointerUp);
    element.removeEventListener("pointercancel", onPointerUp);
    element.removeEventListener("wheel", onWheel);
    element.removeEventListener("dblclick", onDoubleClick);
    element.removeEventListener("pointerleave", onLeave);
    element.removeEventListener("keydown", onKeyDown);
  };
}
