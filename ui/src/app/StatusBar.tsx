// Status bar (FR-005, FR-005c): load progress, last load time, and the local API address.
import type { ApiStatus } from "../backend/generated/ApiStatus";
import type { LoadingInfo } from "../backend/generated/LoadingInfo";

interface Props {
  loading: LoadingInfo | null;
  lastLoadMs: number | null;
  apiStatus: ApiStatus | null;
}

const formatDuration = (ms: number) =>
  ms < 1000 ? `${String(Math.round(ms))} ms` : `${(ms / 1000).toFixed(2)} s`;

const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;

export function StatusBar({ loading, lastLoadMs, apiStatus }: Props) {
  return (
    <footer className="status-bar">
      {loading ? (
        <span role="status" className="loading">
          <progress aria-label="Loading" /> Loading {fileName(loading.path)}…
        </span>
      ) : (
        lastLoadMs !== null && <span>Loaded in {formatDuration(lastLoadMs)}</span>
      )}
      <span className="spacer" />
      {apiStatus === null ? (
        <span>Local API starting…</span>
      ) : "base_url" in apiStatus ? (
        <span title="Scripts can control this window through the local API">
          Local API: {apiStatus.base_url}
        </span>
      ) : (
        <span className="api-error" title={apiStatus.error.hint}>
          Local API unavailable: {apiStatus.error.message}
        </span>
      )}
    </footer>
  );
}
