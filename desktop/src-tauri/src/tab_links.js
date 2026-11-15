// Installed only by the desktop host. The host admits commands from attached
// product WebViews on the current service origin.
(() => {
  if (location.protocol !== "http:" || location.hostname !== "127.0.0.1") return;

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
    const dark = appearance === "dark" ||
      (appearance === "system" && matchMedia("(prefers-color-scheme: dark)").matches);
    root.style.backgroundColor = dark ? "#0e0d0c" : "#f6efe3";
  }

  function syncAppearance(event) {
    window.__TAURI_INTERNALS__.invoke("desktop_appearance", event.detail)
      .catch((error) => console.error("Could not update Atlas window appearance", error));
  }

  function watchTheme() {
    const root = document.documentElement;
    if (!root) return false;
    applyStartupTheme(root);
    const observer = new MutationObserver(() => {
      if (root.dataset.theme) root.style.backgroundColor = "var(--bg)";
    });
    window.addEventListener("atlas-appearance-changed", syncAppearance);
    observer.observe(root, { attributes: true, attributeFilter: ["data-theme"] });
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
    const anchor = event.composedPath().find(
      (element) => element instanceof HTMLAnchorElement && element.hasAttribute("href"),
    );
    if (!anchor || anchor.hasAttribute("download") || anchor.classList.contains("skip-link")) return;
    const destination = new URL(anchor.href);
    if (destination.origin !== location.origin) return;
    return destination;
  }

  function request(event, context) {
    const destination = link(event);
    if (!destination) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    window.__TAURI_INTERNALS__.invoke("desktop_tab_link", {
      url: destination.href,
      context,
    }).catch((error) => console.error("Could not open Atlas link", error));
  }

  window.addEventListener("click", (event) => {
    if (event.button === 0 && event.metaKey && !event.ctrlKey &&
        !event.altKey && !event.shiftKey) request(event, false);
  }, true);
  window.addEventListener("onauxclick" in window ? "auxclick" : "mouseup", (event) => {
    if (event.button === 1 && !event.altKey) request(event, false);
  }, true);
  window.addEventListener("contextmenu", (event) => request(event, true), true);
})();
