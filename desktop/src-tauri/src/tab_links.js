// Installed only by the desktop host. The host admits commands from attached
// product WebViews on the current service origin.
(() => {
  if (location.protocol !== "http:" || location.hostname !== "127.0.0.1") return;

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
