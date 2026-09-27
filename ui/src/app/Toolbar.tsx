// Toolbar: Open, Recent files (FR-001, FR-001b), appearance (FR-010a), diagnostics.
import { useRef } from "react";
import type { Theme } from "../backend/generated/Theme";

interface Props {
  recentFiles: string[];
  theme: Theme;
  showFps: boolean;
  onOpenDialog: () => void;
  onOpenRecent: (path: string) => void;
  onTheme: (theme: Theme) => void;
  onShowFps: (show: boolean) => void;
}

export function Toolbar(props: Props) {
  const recentMenu = useRef<HTMLDetailsElement>(null);
  return (
    <header className="toolbar">
      <button className="primary" onClick={props.onOpenDialog} title="Open a CSV file (Ctrl+O)">
        Open…
      </button>
      <details className="menu" ref={recentMenu}>
        <summary>Recent</summary>
        <ul>
          {props.recentFiles.length === 0 && <li className="empty">No recent files</li>}
          {props.recentFiles.map((path) => (
            <li key={path}>
              <button
                title={path}
                onClick={() => {
                  if (recentMenu.current) recentMenu.current.open = false;
                  props.onOpenRecent(path);
                }}
              >
                {path}
              </button>
            </li>
          ))}
        </ul>
      </details>
      <span className="spacer" />
      <label>
        <input
          type="checkbox"
          checked={props.showFps}
          onChange={(e) => {
            props.onShowFps(e.target.checked);
          }}
        />
        Show FPS
      </label>
      <label>
        Appearance
        <select
          value={props.theme}
          onChange={(e) => {
            props.onTheme(e.target.value as Theme);
          }}
        >
          <option value="system">System</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
      </label>
    </header>
  );
}
