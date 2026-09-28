// Creates the PlotController (renderer, gestures, resize tracking) once per mount.
import { useEffect, useRef, useState } from "react";
import type { BackendError } from "../backend/commands";
import { PlotController, type Overlay } from "./controller";
import { attachGestures, type BoxRect } from "./interaction/gestures";

const EMPTY_OVERLAY: Overlay = { range: null, xTicks: [], yTicks: [], hover: null, fps: 0 };

export function usePlotController(onError: (error: BackendError) => void) {
  const areaRef = useRef<HTMLDivElement>(null);
  const canvasHostRef = useRef<HTMLDivElement>(null);
  const controllerRef = useRef<PlotController | null>(null);
  const onErrorRef = useRef(onError);
  const [overlay, setOverlay] = useState<Overlay>(EMPTY_OVERLAY);
  const [box, setBox] = useState<BoxRect | null>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    onErrorRef.current = onError;
  }, [onError]);

  useEffect(() => {
    const area = areaRef.current;
    const canvasHost = canvasHostRef.current;
    if (!area || !canvasHost) return;
    const controller = new PlotController(setOverlay, (e) => {
      onErrorRef.current(e);
    });
    controllerRef.current = controller;
    const detach = attachGestures(area, controller, setBox);
    const observer = new ResizeObserver(([entry]) => {
      if (entry) controller.resize(entry.contentRect.width, entry.contentRect.height);
    });
    void controller.init(canvasHost).then((ok) => {
      if (!ok) return;
      observer.observe(area);
      setReady(true);
    });
    return () => {
      observer.disconnect();
      detach();
      controller.destroy();
      controllerRef.current = null;
      setReady(false);
    };
  }, []);

  return { areaRef, canvasHostRef, controllerRef, overlay, box, ready };
}
