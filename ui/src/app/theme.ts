// Appearance (FR-010a): System follows the Windows app mode live; the choice is stored in
// settings by the backend.
import { useEffect, useState } from "react";
import type { Theme } from "../backend/generated/Theme";

const DARK_QUERY = "(prefers-color-scheme: dark)";

/** Applies `theme` to the document and returns whether the dark appearance is active. */
export function useAppliedTheme(theme: Theme): boolean {
  const [systemDark, setSystemDark] = useState(() => matchMedia(DARK_QUERY).matches);

  useEffect(() => {
    const query = matchMedia(DARK_QUERY);
    const onChange = () => {
      setSystemDark(query.matches);
    };
    query.addEventListener("change", onChange);
    return () => {
      query.removeEventListener("change", onChange);
    };
  }, []);

  const dark = theme === "dark" || (theme === "system" && systemDark);
  useEffect(() => {
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }, [dark]);
  return dark;
}
