import { Alert, Button, Tooltip } from "antd";
import { Link, useRouterState } from "@tanstack/react-router";
import {
  Activity,
  ArrowDownToLine,
  ArrowLeft,
  BookOpen,
  ChevronRight,
  Settings,
  Terminal,
} from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { getReadiness } from "../api/atlasApi";
import { SavedWritesPaused } from "../shared/feedback/SavedWritesPaused";
import { LoadingStatus } from "../shared/ui/LoadingStatus";
import { useDesktopTab } from "./DesktopTabContext";
import { usePageTitle } from "../shared/browser/usePageTitle";
import { LinkNotFound } from "./InvalidUrl";
import { useAtlasRouter } from "./router";
import { DesktopUpdateSection, DesktopUpdateShortcut } from "./DesktopUpdate";
import { DesktopSearchData, DesktopTerminal } from "./DesktopMaintenance";
import {
  activeSettingsPane,
  invokeDesktop,
  readDesktopAttachment,
  settingsPath,
  useDesktopAttachment,
  useSettingsRequest,
  windowsTabsEnabled,
  type SettingsDestination,
} from "./desktopBridge";
import "./desktopSettings.css";

const paneNames = {
  updates: "Updates",
  channel: "Release channel",
  data: "Search & data",
  terminal: "Terminal",
};
const windowsTitles = {
  ...paneNames,
  data: "Search and reference data",
  terminal: "Terminal access",
  updates: "Settings",
};
const macPanes = [
  { key: "updates", icon: ArrowDownToLine },
  { key: "data", icon: BookOpen },
  { key: "terminal", icon: Terminal },
] as const;
type Pane = keyof typeof paneNames;
function actionError(error: unknown) {
  return typeof error === "string" ? error : "Atlas couldn’t open Settings.";
}

export function DesktopSettingsControls() {
  const attachment = useDesktopAttachment();
  const [error, setError] = useState<string | null>(null);
  if (!attachment?.setup) return null;
  const {
    token,
    setup: { status },
  } = attachment;
  const running = status === "running" || status === "cancelling";
  function open(destination: SettingsDestination) {
    setError(null);
    void invokeDesktop("desktop_settings_open", {
      token,
      destination,
    }).catch((failure) => setError(actionError(failure)));
  }
  return (
    <>
      {windowsTabsEnabled() && (
        <Tooltip title="Settings" trigger={["hover", "focus"]}>
          <Button
            type="text"
            aria-label="Settings"
            icon={<Settings size={18} />}
            onClick={() => open("general")}
          />
        </Tooltip>
      )}
      {status !== "idle" && (
        <Button size="small" icon={<Activity size={15} />} onClick={() => open("data")}>
          {running
            ? "Setup in progress"
            : status === "failed"
              ? "Review setup"
              : "Resume setup"}
        </Button>
      )}
      <DesktopUpdateShortcut onOpen={open} />
      {error && <Alert type="error" title={error} />}
    </>
  );
}

function SearchDataPane() {
  const active = useDesktopTab()?.active ?? true;
  const refreshed = useRef(false);
  const [refreshError, setRefreshError] = useState<string | null>(null);
  const readiness = useQuery({
    queryKey: ["readiness"],
    queryFn: getReadiness,
    enabled: active,
    refetchInterval: (query) => (query.state.data?.saved_writes_paused ? 1500 : false),
  });
  useEffect(() => {
    if (!active) {
      refreshed.current = false;
      return;
    }
    if (refreshed.current) return;
    refreshed.current = true;
    const attachment = readDesktopAttachment();
    if (
      !attachment ||
      attachment.referenceLoading ||
      attachment.setup.status === "running" ||
      attachment.setup.status === "cancelling"
    )
      return;
    void invokeDesktop("desktop_setup_action", {
      action: "read_reference_status",
      token: attachment.token,
      job: attachment.setup.job,
      sequence: attachment.setup.sequence,
    })
      .then(() => setRefreshError(null))
      .catch((error) =>
        setRefreshError(
          typeof error === "string" ? error : "Atlas couldn’t check reference data.",
        ),
      );
  }, [active]);
  const refetch = readiness.refetch;
  useEffect(() => {
    if (!active) return;
    const refresh = () => void refetch();
    window.addEventListener("atlas-reference-updated", refresh);
    return () => window.removeEventListener("atlas-reference-updated", refresh);
  }, [active, refetch]);
  return (
    <SavedWritesPaused value={readiness.data?.saved_writes_paused === true}>
      {readiness.data?.saved_writes_paused && (
        <Alert
          type="info"
          title="Editing is temporarily paused while Atlas updates the workspace."
        />
      )}
      {refreshError && <Alert type="error" title={refreshError} />}
      <DesktopSearchData />
    </SavedWritesPaused>
  );
}

function SettingsRow({
  to,
  label,
  value,
}: {
  to: "/settings/channel" | "/settings/data" | "/settings/terminal";
  label: string;
  value?: string;
}) {
  return (
    <Link
      className="desktop-settings__row"
      to="/settings/$pane"
      params={{ pane: to.slice("/settings/".length) }}
      data-settings-target={to}
    >
      <span>{label}</span>
      <span className="desktop-settings__row-end">
        {value} <ChevronRight size={16} aria-hidden="true" />
      </span>
    </Link>
  );
}

export function DesktopSettings() {
  const attachment = useDesktopAttachment();
  const path = useRouterState({ select: (state) => state.location.pathname });
  const windows = windowsTabsEnabled();
  const pane = activeSettingsPane(path);
  if (
    !pane ||
    (!attachment && !("__TAURI__" in window)) ||
    (attachment && !windows && !attachment.settingsWindow)
  )
    return <LinkNotFound />;
  if (!attachment) return <LoadingStatus label="Opening Settings…" />;
  return <SettingsPage key={attachment.token} pane={pane} windows={windows} />;
}
function SettingsPage({ pane, windows }: { pane: Pane; windows: boolean }) {
  const router = useAtlasRouter();
  const attachment = useDesktopAttachment()!;
  const main = useRef<HTMLElement>(null);
  const previousPane = useRef<Pane>(pane);
  const tab = useDesktopTab();
  const active = tab?.active ?? true;
  const [error, setError] = useState<string | null>(null);
  usePageTitle(
    windows ? windowsTitles[pane] : paneNames[pane === "channel" ? "updates" : pane],
    "Settings",
  );
  useSettingsRequest(
    windows ? undefined : attachment.settingsRequest,
    attachment.token,
    (destination) =>
      destination === "general"
        ? undefined
        : router.navigate({ href: settingsPath(destination, false), replace: true }),
    (failure) => setError(actionError(failure)),
  );
  useEffect(() => {
    if (windows) return;
    void invokeDesktop("desktop_settings_pane", {
      token: attachment.token,
      destination: pane,
    }).catch((failure) => setError(actionError(failure)));
  }, [attachment.token, pane, windows]);
  useLayoutEffect(() => {
    const previous = previousPane.current;
    previousPane.current = pane;
    if (previous === pane || !active || !windows) return;
    const target =
      pane === "updates"
        ? main.current?.querySelector<HTMLElement>(
            `[data-settings-target="${settingsPath(previous, true)}"]`,
          )
        : null;
    (target ?? main.current?.querySelector<HTMLElement>("h1"))?.focus();
  }, [pane, active, windows]);

  const channel = attachment.channel;
  const channelLabel = channel?.installed.tag.startsWith("nightly/")
    ? "Nightly"
    : "Stable";
  return (
    <main
      ref={main}
      id={active ? "atlas-main" : undefined}
      tabIndex={-1}
      className={`desktop-settings desktop-settings--${windows ? "windows" : "mac"}`}
    >
      {!windows && (
        <nav className="desktop-settings__panes" aria-label="Settings panes">
          {macPanes.map(({ key, icon: Icon }) => (
            <Link
              to="/settings/$pane"
              params={{ pane: key }}
              key={key}
              replace
              aria-current={
                (pane === "channel" ? "updates" : pane) === key ? "page" : undefined
              }
            >
              <Icon size={20} aria-hidden="true" />
              {paneNames[key]}
            </Link>
          ))}
        </nav>
      )}
      <div className="desktop-settings__body">
        {windows && pane !== "updates" && (
          <Link
            to="/settings"
            className="desktop-settings__back"
            aria-label="Back to Settings"
          >
            <ArrowLeft size={16} /> Settings
          </Link>
        )}
        <h1 className={windows ? undefined : "sr-only"} tabIndex={-1}>
          {windows
            ? windowsTitles[pane]
            : paneNames[pane === "channel" ? "updates" : pane]}
        </h1>
        {error && <Alert type="error" title={error} />}
        {(pane === "updates" || (!windows && pane === "channel")) && (
          <>
            <section className="desktop-settings__group" aria-label="App updates">
              {windows && <h2>App updates</h2>}
              <div className="desktop-settings__updates">
                <DesktopUpdateSection
                  channelDisclosure={
                    windows
                      ? undefined
                      : {
                          open: pane === "channel",
                          onToggle: () =>
                            void router.navigate({
                              to: "/settings/$pane",
                              params: {
                                pane: pane === "channel" ? "updates" : "channel",
                              },
                              replace: true,
                            }),
                        }
                  }
                />
                {windows && channel && (
                  <SettingsRow
                    to="/settings/channel"
                    label="Release channel"
                    value={channelLabel}
                  />
                )}
              </div>
            </section>
            {windows && (
              <>
                <section className="desktop-settings__group">
                  <h2>Workspace</h2>
                  <SettingsRow to="/settings/data" label={windowsTitles.data} />
                </section>
                <section className="desktop-settings__group">
                  <h2>Tools</h2>
                  <SettingsRow to="/settings/terminal" label={windowsTitles.terminal} />
                </section>
              </>
            )}
          </>
        )}
        {windows && pane === "channel" && <DesktopUpdateSection view="channel" />}
        {pane === "data" && <SearchDataPane />}
        {pane === "terminal" && <DesktopTerminal />}
      </div>
    </main>
  );
}
