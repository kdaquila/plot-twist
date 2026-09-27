// Keeps a rendering fault in one area from blanking the whole window (FR-013).
import { Component, type ReactNode } from "react";

interface Props {
  area: string;
  children: ReactNode;
}

export class ErrorBoundary extends Component<Props, { failed: boolean }> {
  override state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  override componentDidCatch(error: unknown) {
    console.error(`${this.props.area} failed`, error);
  }

  override render() {
    if (!this.state.failed) return this.props.children;
    return (
      <div className="message error" role="alert">
        <div className="body">
          <div>The {this.props.area} stopped working.</div>
          <div className="hint">Try again; if it keeps happening, please report this issue.</div>
        </div>
        <button
          onClick={() => {
            this.setState({ failed: false });
          }}
        >
          Try again
        </button>
      </div>
    );
  }
}
