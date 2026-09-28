// DOM tick labels and title for one axis; gridlines are drawn by the renderer.
import type { PositionedTick } from "../controller";

interface Props {
  side: "x" | "y";
  ticks: PositionedTick[];
  title: string | null;
}

export function Axis({ side, ticks, title }: Props) {
  return (
    <div className={`plot-${side}-axis`} aria-hidden="true">
      {ticks.map((t) => (
        <span key={t.value} className="tick" style={side === "x" ? { left: t.px } : { top: t.px }}>
          {t.label}
        </span>
      ))}
      {title !== null && <span className="title">{title}</span>}
    </div>
  );
}
