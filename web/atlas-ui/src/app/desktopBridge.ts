import { useEffect, useRef, useSyncExternalStore } from "react";
import type { UpdateAttachment } from "./DesktopUpdate";

export type SettingsDestination =
  "general" | "updates" | "channel" | "data" | "terminal";
export type ModelSelection = { profiles: string[]; preferred: string | null };
export type ReferenceStatus = {
  revision: string | null;
  target: string;
  targetVersion: string;
  managed: boolean;
  available: boolean;
  modelsAvailable?: boolean;
  canChangeModels?: boolean;
  models?: { id: string; label: string; installed: boolean; usable: boolean }[];
  preferred?: string | null;
};
export type DesktopAttachment = Pick<UpdateAttachment, "channel" | "update"> & {
  token: string;
  settingsWindow?: boolean;
  settingsRequest?: { id: number; destination: SettingsDestination } | null;
  referenceLoading?: boolean;
  reference?: ReferenceStatus | null;
  setup: {
    job: number;
    sequence: number;
    status: "idle" | "available" | "running" | "cancelling" | "failed";
    progress?: { message?: string; current?: number; total?: number; unit?: string };
    message?: string;
    local_build: boolean;
    can_resume: boolean;
    reference_job?: boolean;
    model_job?: boolean;
  };
};
type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: DesktopAttachment;
  __ATLAS_WINDOWS_TABS__?: boolean;
  __TAURI__?: { core: { invoke<T>(command: string, args: object): Promise<T> } };
};

export function windowsTabsEnabled() {
  return (window as DesktopWindow).__ATLAS_WINDOWS_TABS__ === true;
}
export function readDesktopAttachment() {
  return (window as DesktopWindow).__ATLAS_DESKTOP_SETUP__;
}
function subscribe(callback: () => void) {
  window.addEventListener("atlas-desktop-setup", callback);
  return () => window.removeEventListener("atlas-desktop-setup", callback);
}
export function useDesktopAttachment() {
  return useSyncExternalStore(subscribe, readDesktopAttachment);
}
export function invokeDesktop<T>(command: string, args: object): Promise<T> {
  const host = (window as DesktopWindow).__TAURI__;
  if (!host) return Promise.reject(new Error("Open Settings in Atlas Desktop."));
  return host.core.invoke<T>(command, args);
}

export function settingsPath(destination: SettingsDestination, windows: boolean) {
  if (destination === "general" || destination === "updates") {
    return windows ? "/settings" : "/settings/updates";
  }
  return `/settings/${destination}` as const;
}

export function activeSettingsPane(path: string) {
  const page = path.replace(/\/$/, "");
  if (page === "/settings") return "updates";
  return (["updates", "channel", "data", "terminal"] as const).find(
    (pane) => page === settingsPath(pane, false),
  );
}

export function isSettingsPath(path: string) {
  return activeSettingsPane(path) !== undefined;
}

/** Both hosts retain a request until its destination is reached in this document. */
export function useSettingsRequest(
  request: DesktopAttachment["settingsRequest"],
  token: string | undefined,
  open: (destination: SettingsDestination) => void | Promise<void>,
  onError: (error: unknown) => void,
) {
  const handled = useRef<{ token: string; id: number } | null>(null);
  const id = request?.id;
  const destination = request?.destination;
  useEffect(() => {
    if (!token || id === undefined || !destination) return;
    if (handled.current?.token === token && handled.current.id >= id) return;
    handled.current = { token, id };
    void Promise.resolve(open(destination))
      .then(() => invokeDesktop("desktop_settings_ack", { token, id }))
      .catch(onError);
  }, [token, id, destination, open, onError]);
}
