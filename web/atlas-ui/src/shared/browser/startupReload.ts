import { API_CONTRACT } from "../../api/compatibility";
import { reloadFrontend } from "./reloadFrontend";

const RELOAD_KEY = "atlas-startup-reload";

export function reloadMismatchedFrontendOnce() {
  try {
    // One automatic attempt per tab until a successful handshake. Record it
    // before reloading; unavailable storage falls back to the manual button.
    if (window.sessionStorage.getItem(RELOAD_KEY)) return;
    window.sessionStorage.setItem(RELOAD_KEY, API_CONTRACT);
  } catch {
    return;
  }
  reloadFrontend();
}

export function clearStartupReloadAttempt() {
  try {
    window.sessionStorage.removeItem(RELOAD_KEY);
  } catch {
    // The manual Reload page action works without browser storage.
  }
}
