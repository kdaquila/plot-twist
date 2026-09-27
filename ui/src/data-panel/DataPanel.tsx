// Left panel: loaded file summary (FR-003), bad-cell warning, and column selection.
import type { BackendError } from "../backend/commands";
import type { DatasetSummary } from "../backend/generated/DatasetSummary";
import { BadCells } from "./BadCells";
import { ColumnPicker } from "./ColumnPicker";
import type { Draft } from "./draft";

interface Props {
  dataset: DatasetSummary | null;
  draft: Draft;
  onDraft: (draft: Draft) => void;
  onError: (error: BackendError) => void;
}

const DELIMITER = { comma: "comma", semicolon: "semicolon", tab: "tab" } as const;

export function DataPanel({ dataset, draft, onDraft, onError }: Props) {
  if (!dataset) {
    return (
      <aside className="data-panel">
        <p className="empty">No file loaded. Open a CSV file or drop one onto the window.</p>
      </aside>
    );
  }
  return (
    <aside className="data-panel" aria-label="Data">
      <h2 title={dataset.path}>{dataset.file_name}</h2>
      <div className="meta">
        {dataset.row_count.toLocaleString("en-US")} rows · {dataset.columns.length} columns ·{" "}
        {DELIMITER[dataset.delimiter]}-separated{dataset.has_header ? "" : " · no header row"}
      </div>
      <BadCells key={dataset.id} dataset={dataset} onError={onError} />
      <ColumnPicker columns={dataset.columns} draft={draft} onChange={onDraft} />
    </aside>
  );
}
