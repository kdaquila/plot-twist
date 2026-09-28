// Bad-cell warning (FR-005b, FR-011a): summary with per-column counts that opens a paged
// list (100 per page) fetched from the backend.
import { useRef, useState } from "react";
import { getBadCells, toBackendError, type BackendError } from "../backend/commands";
import type { BadCellPage } from "../backend/generated/BadCellPage";
import type { DatasetSummary } from "../backend/generated/DatasetSummary";
import { formatCount as count } from "./format";

const PAGE = 100;

interface Props {
  dataset: DatasetSummary;
  onError: (error: BackendError) => void;
}

export function BadCells({ dataset, onError }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [page, setPage] = useState<BadCellPage | null>(null);
  const total = dataset.bad_cell_total;
  if (total === 0) return null;

  const show = (offset: number) => {
    getBadCells(dataset.id, offset, PAGE).then(
      (p) => {
        setPage(p);
        if (!dialog.current?.open) dialog.current?.showModal();
      },
      (e: unknown) => {
        onError(toBackendError(e));
      },
    );
  };

  const perColumn = dataset.columns.filter((c) => c.bad_cell_count > 0);
  const offset = page?.offset ?? 0;
  return (
    <div className="warning" role="status">
      <button
        className="link"
        onClick={() => {
          show(0);
        }}
      >
        {count(total)} bad {total === 1 ? "cell" : "cells"}
      </button>{" "}
      stored as missing values:{" "}
      {perColumn.map((c) => `${c.name} (${count(c.bad_cell_count)})`).join(", ")}
      <dialog ref={dialog} aria-label="Bad cells">
        <h3>
          {count(total)} bad {total === 1 ? "cell" : "cells"} in {dataset.file_name}
        </h3>
        <p className="hint">
          Values that don&apos;t match their column&apos;s type are plotted as missing. Fix or
          remove them in the file if they should be numbers or dates.
        </p>
        <table>
          <thead>
            <tr>
              <th>Line</th>
              <th>Column</th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            {page?.items.map((cell) => (
              <tr key={`${String(cell.line)}:${String(cell.column)}`}>
                <td>{count(cell.line)}</td>
                <td>{cell.column_name}</td>
                <td>
                  <code>{cell.text}</code>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="pager">
          <span>
            {count(offset + 1)}–{count(Math.min(offset + PAGE, total))} of {count(total)}
          </span>
          <button
            disabled={offset === 0}
            onClick={() => {
              show(Math.max(0, offset - PAGE));
            }}
          >
            Previous
          </button>
          <button
            disabled={offset + PAGE >= total}
            onClick={() => {
              show(offset + PAGE);
            }}
          >
            Next
          </button>
          <button
            onClick={() => {
              dialog.current?.close();
            }}
          >
            Close
          </button>
        </div>
      </dialog>
    </div>
  );
}
