// Errors and notices (FR-011): what went wrong (the message names the line and column),
// the offending value, and the suggested next step. Errors stay until dismissed.
import type { Message } from "./messages";

interface Props {
  messages: Message[];
  onDismiss: (id: number) => void;
}

export function ErrorDisplay({ messages, onDismiss }: Props) {
  if (messages.length === 0) return null;
  return (
    <div className="messages" aria-live="polite">
      {messages.map((m) => (
        <div
          key={m.id}
          className={`message ${m.kind}`}
          role={m.kind === "error" ? "alert" : "status"}
        >
          <div className="body">
            {m.kind === "error" ? (
              <>
                <div>{m.report.message}</div>
                {m.report.value !== null && !m.report.message.includes(m.report.value) && (
                  <div>
                    Value: <code>{m.report.value}</code>
                  </div>
                )}
                <div className="hint">{m.report.hint}</div>
              </>
            ) : (
              <>
                <div>{m.text}</div>
                {m.hint !== undefined && <div className="hint">{m.hint}</div>}
              </>
            )}
          </div>
          {m.kind === "error" && m.action && (
            <button
              onClick={() => {
                m.action?.run();
                onDismiss(m.id);
              }}
            >
              {m.action.label}
            </button>
          )}
          <button
            aria-label="Dismiss"
            title="Dismiss"
            onClick={() => {
              onDismiss(m.id);
            }}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
