import type { DatasetSummary } from "../../backend/generated/DatasetSummary";
import type { PlotConfig } from "../../backend/generated/PlotConfig";
import { seriesStyle } from "./palette";

interface Props {
  dataset: DatasetSummary;
  plot: PlotConfig;
  slotOf: (column: string) => number;
  hidden: ReadonlySet<string>;
  dark: boolean;
  onToggle: (column: string) => void;
}

/** Click an entry to hide or show that series (FR-009c, FR-009j). */
export function Legend({ dataset, plot, slotOf, hidden, dark, onToggle }: Props) {
  return (
    <div className="legend" aria-label="Legend">
      {plot.y.map((name) => {
        const look = seriesStyle(slotOf(name), dark);
        const isHidden = hidden.has(name);
        const empty = dataset.columns.find((c) => c.name === name)?.min == null;
        return (
          <button
            key={name}
            className={isHidden ? "hidden" : ""}
            title={`${name} — click to ${isHidden ? "show" : "hide"}`}
            aria-pressed={!isHidden}
            onClick={() => {
              onToggle(name);
            }}
          >
            <Swatch
              color={look.color}
              alternate={look.alternate}
              scatter={plot.style === "scatter"}
            />
            <span className="name">{name}</span>
            {isHidden && <span className="note">hidden</span>}
            {empty && <span className="note">no values</span>}
          </button>
        );
      })}
    </div>
  );
}

function Swatch({
  color,
  alternate,
  scatter,
}: {
  color: string;
  alternate: boolean;
  scatter: boolean;
}) {
  return (
    <svg width="22" height="12" aria-hidden="true">
      {scatter ? (
        <circle
          cx="11"
          cy="6"
          r="4"
          fill={alternate ? "none" : color}
          stroke={color}
          strokeWidth="1.5"
        />
      ) : (
        <line
          x1="1"
          y1="6"
          x2="21"
          y2="6"
          stroke={color}
          strokeWidth="2"
          strokeDasharray={alternate ? "5 3" : undefined}
        />
      )}
    </svg>
  );
}
