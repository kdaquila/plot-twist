// Hover readout (FR-009b): series name and exact X and Y of the nearest point.
import { formatDateTime, formatExact } from "./axes/format";
import type { HoverHit } from "./hover";

interface Props {
  hit: HoverHit;
  xName: string;
  xIsTime: boolean;
}

export function HoverTooltip({ hit, xName, xIsTime }: Props) {
  return (
    <>
      <div className="plot-marker" style={{ left: hit.px, top: hit.py }} />
      <div className="plot-tooltip" style={{ left: hit.px, top: hit.py }}>
        <strong>{hit.series}</strong>
        <div>
          {xName}: {xIsTime ? formatDateTime(hit.x) : formatExact(hit.x)}
        </div>
        <div>
          {hit.series}: {formatExact(hit.y)}
        </div>
      </div>
    </>
  );
}
