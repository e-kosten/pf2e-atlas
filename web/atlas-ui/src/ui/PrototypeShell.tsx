import { Moon, RefreshCw, Sun } from "lucide-react";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";
import type { LibraryPrototype } from "../state/searchState";

type PrototypeShellProps = {
  activeLibrary: LibraryPrototype;
  darkMode: boolean;
  onDarkModeChange: (enabled: boolean) => void;
  onLibraryChange: (library: LibraryPrototype) => void;
  workspace: AtlasWorkspaceState;
  children: React.ReactNode;
};

export function PrototypeShell({
  activeLibrary,
  darkMode,
  onDarkModeChange,
  onLibraryChange,
  workspace,
  children,
}: PrototypeShellProps) {
  const readiness = workspace.readiness.data;
  const status = readiness?.status ?? "blocked";

  return (
    <div className="atlas-app" data-theme={darkMode ? "dark" : "light"}>
      <header className="topbar">
        <div>
          <h1>PF2e Atlas</h1>
          <p>{readiness?.message ?? "Waiting for the local Atlas service."}</p>
        </div>
        <div className="topbar__actions">
          <span className={`status-pill status-pill--${status}`}>{status}</span>
          <div className="library-switch" role="tablist" aria-label="Component library">
            <button
              aria-selected={activeLibrary === "ant"}
              onClick={() => onLibraryChange("ant")}
              type="button"
            >
              AntD
            </button>
            <button
              aria-selected={activeLibrary === "mantine"}
              onClick={() => onLibraryChange("mantine")}
              type="button"
            >
              Mantine
            </button>
          </div>
          <button
            className="icon-button"
            onClick={workspace.refresh}
            title="Refresh"
            type="button"
          >
            <RefreshCw size={18} />
          </button>
          <button
            className="icon-button"
            onClick={() => onDarkModeChange(!darkMode)}
            title={darkMode ? "Light mode" : "Dark mode"}
            type="button"
          >
            {darkMode ? <Sun size={18} /> : <Moon size={18} />}
          </button>
        </div>
      </header>
      {workspace.errorMessage && (
        <div className="error-banner">{workspace.errorMessage}</div>
      )}
      {children}
    </div>
  );
}
