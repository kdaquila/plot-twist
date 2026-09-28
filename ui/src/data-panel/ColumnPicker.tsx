// X / Y / style selection (FR-006, FR-008) with each column's type and missing count (FR-003).
import type { ColumnInfo } from "../backend/generated/ColumnInfo";
import type { Draft } from "./draft";
import { formatCount } from "./format";

interface Props {
  columns: ColumnInfo[];
  draft: Draft;
  onChange: (draft: Draft) => void;
}

const KIND_LABEL = { numeric: "number", datetime: "date/time", text: "text" } as const;

function yUnavailableReason(column: ColumnInfo, x: string | null): string | null {
  if (column.kind === "text") return "Text columns can't be plotted";
  if (column.kind === "datetime") return "Date/time columns can be used as X only";
  if (column.name === x) return "This is the X column";
  return null;
}

export function ColumnPicker({ columns, draft, onChange }: Props) {
  const toggleY = (name: string, on: boolean) => {
    const y = on ? [...draft.y, name] : draft.y.filter((n) => n !== name);
    onChange({ ...draft, y });
  };

  return (
    <>
      <fieldset>
        <legend>X column</legend>
        <select
          aria-label="X column"
          value={draft.x ?? ""}
          onChange={(e) => {
            const x = e.target.value;
            onChange({ ...draft, x, y: draft.y.filter((n) => n !== x) });
          }}
        >
          {draft.x === null && <option value="">Choose…</option>}
          {columns
            .filter((c) => c.usable_as_x)
            .map((c) => (
              <option key={c.index} value={c.name}>
                {c.name} ({KIND_LABEL[c.kind]})
              </option>
            ))}
        </select>
      </fieldset>

      <fieldset>
        <legend>Y columns</legend>
        {columns.map((c) => {
          const reason = yUnavailableReason(c, draft.x);
          return (
            <label key={c.index} className="column-row" title={reason ?? c.name}>
              <input
                type="checkbox"
                disabled={reason !== null}
                checked={draft.y.includes(c.name)}
                onChange={(e) => {
                  toggleY(c.name, e.target.checked);
                }}
              />
              <span className="name">{c.name}</span>
              <span className="tag">{KIND_LABEL[c.kind]}</span>
              {c.missing_count > 0 && (
                <span className="tag" title="Empty, missing-value, or bad cells">
                  {formatCount(c.missing_count)} missing
                </span>
              )}
            </label>
          );
        })}
      </fieldset>

      <fieldset>
        <legend>Style</legend>
        {(["line", "scatter"] as const).map((style) => (
          <label key={style} className="column-row">
            <input
              type="radio"
              name="plot-style"
              checked={draft.style === style}
              onChange={() => {
                onChange({ ...draft, style });
              }}
            />
            {style === "line" ? "Line" : "Scatter"}
          </label>
        ))}
      </fieldset>
    </>
  );
}
