// Installed only by the desktop host. The host admits commands from attached
// product WebViews on the current service origin.
(() => {
  if (location.protocol !== "http:" || location.hostname !== "127.0.0.1")
    return;

  // Paint the document before the application bundle runs. The React theme
  // takes over when it sets data-theme on the document element.
  function themePreference() {
    try {
      const value = localStorage.getItem("atlas-ui-color-scheme");
      if (value === "dark" || value === "light") return value;
    } catch {
      // Use the system appearance if preferences are unavailable.
    }
    return "system";
  }

  function applyStartupTheme(root) {
    const appearance = themePreference();
    const dark =
      appearance === "dark" ||
      (appearance === "system" &&
        matchMedia("(prefers-color-scheme: dark)").matches);
    root.style.setProperty("--bg", dark ? "#0e0d0c" : "#f6efe3");
    root.style.backgroundColor = "var(--bg)";
    root.style.colorScheme = dark ? "dark" : "light";
  }

  function syncAppearance(event) {
    window.__TAURI_INTERNALS__
      .invoke("desktop_appearance", event.detail)
      .catch((error) =>
        console.error("Could not update Atlas window appearance", error),
      );
  }

  function watchTheme() {
    const root = document.documentElement;
    if (!root) return false;
    applyStartupTheme(root);
    window.addEventListener("atlas-appearance-changed", syncAppearance);
    return true;
  }

  if (!watchTheme()) {
    const observer = new MutationObserver(() => {
      if (watchTheme()) observer.disconnect();
    });
    observer.observe(document, { childList: true });
  }

  function link(event) {
    if (!event.isTrusted || event.defaultPrevented) return;
    const anchor = event
      .composedPath()
      .find(
        (element) =>
          element instanceof HTMLAnchorElement && element.hasAttribute("href"),
      );
    if (
      !anchor ||
      anchor.hasAttribute("download") ||
      anchor.classList.contains("skip-link")
    )
      return;
    if (anchor.origin === location.origin) return anchor;
  }

  function request(event, context, foreground = false) {
    const anchor = link(event);
    if (!anchor || (foreground && anchor.target.toLowerCase() !== "_blank")) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    window.__TAURI_INTERNALS__
      .invoke("desktop_tab_link", {
        url: anchor.href,
        context,
        foreground,
      })
      .catch((error) => console.error("Could not open Atlas link", error));
  }

  const windows = navigator.platform === "Win32";
  window.addEventListener(
    "click",
    (event) => {
      if (event.button !== 0 || event.altKey || event.shiftKey) return;
      if (windows ? event.ctrlKey && !event.metaKey : event.metaKey && !event.ctrlKey)
        request(event, false);
      else if (!event.ctrlKey && !event.metaKey)
        request(event, false, true);
    },
    true,
  );
  window.addEventListener(
    "onauxclick" in window ? "auxclick" : "mouseup",
    (event) => {
      if (event.button === 1 && !event.altKey) request(event, false);
    },
    true,
  );
  window.addEventListener("contextmenu", (event) => request(event, true), true);
})();
