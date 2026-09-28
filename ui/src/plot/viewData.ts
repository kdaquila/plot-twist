// Schedules backend view requests: at most one in flight, at most one every 100 ms during
// continuous motion, and a final request 50 ms after the view settles (research R4).
import { getView, toBackendError, type BackendError } from "../backend/commands";
import type { ViewRequest } from "../backend/generated/ViewRequest";
import { decodeViewPayload, type SeriesPayload } from "../backend/viewPayload";

const SETTLE_MS = 50;
const THROTTLE_MS = 100;

export class ViewFetcher {
  private inFlight = false;
  private queued = false;
  private lastSent = 0;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private stopped = false;

  constructor(
    private readonly build: () => ViewRequest | null,
    private readonly onData: (request: ViewRequest, series: SeriesPayload[]) => void,
    private readonly onError: (error: BackendError) => void,
  ) {}

  /** Call whenever the view or the plotted data changes. */
  request() {
    if (this.stopped) return;
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.flush();
    }, SETTLE_MS);
    if (performance.now() - this.lastSent >= THROTTLE_MS) this.flush();
  }

  stop() {
    this.stopped = true;
    clearTimeout(this.timer);
  }

  private flush() {
    if (this.stopped) return;
    if (this.inFlight) {
      this.queued = true;
      return;
    }
    const request = this.build();
    if (!request) return;
    this.inFlight = true;
    this.lastSent = performance.now();
    getView(request)
      .then((buffer) => {
        if (!this.stopped) this.onData(request, decodeViewPayload(buffer));
      })
      .catch((error: unknown) => {
        if (!this.stopped) this.onError(toBackendError(error));
      })
      .finally(() => {
        this.inFlight = false;
        if (this.queued) {
          this.queued = false;
          this.flush();
        }
      });
  }
}
