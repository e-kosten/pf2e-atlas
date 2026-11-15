// Native windows run this before the initial product document paints.
(() => {
  if (location.protocol !== "http:" || location.hostname !== "127.0.0.1")
    return;
  if (performance.getEntriesByType("navigation")[0]?.type === "reload") return;
  const storageKey = `atlas-opened:${window.__atlasOpeningStarted}`;
  try {
    if (sessionStorage.getItem(storageKey)) return;
  } catch {
    // A disabled storage profile can still show the opening view.
  }
  const statusMarkup = /* status markup */ null;
  const openingStyles = /* opening styles */ null;
  const markStyles = /* mark styles */ null;
  const lightColors = /* light colors */ null;
  const darkColors = /* dark colors */ null;

  const style = document.createElement("style");
  const css = `${markStyles}
    ${openingStyles}
    #atlas-opening-cover {
      display: grid;
      position: fixed;
      inset: 0;
      place-items: center;
    }
    #atlas-opening-cover [hidden] { display: none; }
    html:has(#atlas-opening-cover) #root {
      visibility: hidden;
    }
  `;
  const cover = document.createElement("div");
  cover.id = "atlas-opening-cover";
  cover.className = "opening";
  cover.innerHTML = statusMarkup;
  const timer = scheduleOpeningStatus(
    cover.firstElementChild,
    window.__atlasOpeningStarted,
  );

  function mount() {
    if (!document.documentElement) return false;
    // The desktop document prepaint resolves the saved product preference.
    const scheme =
      document.documentElement.style.colorScheme ||
      (window.__atlasOpeningDark ? "dark" : "light");
    cover.style.colorScheme = scheme;
    style.textContent = `${scheme === "dark" ? darkColors : lightColors} ${css}`;
    document.documentElement.append(style, cover);
    return true;
  }
  function rememberOpening() {
    try {
      sessionStorage.setItem(storageKey, "true");
    } catch {
      // The cover still releases as soon as the app renders.
    }
  }
  const unmanagedTimeout = window.__atlasPrimaryOpening
    ? undefined
    : setTimeout(release, 30_000);
  function release() {
    observer.disconnect();
    clearTimeout(timer);
    clearTimeout(unmanagedTimeout);
    cover.remove();
    style.remove();
  }
  function releaseWhenMounted() {
    const main = document.getElementById("atlas-main");
    const productReady =
      main?.getClientRects().length &&
      (!main.closest(".atlas-app--search") ||
        main.querySelector(".search-workspace"));
    // The primary window routes startup failures back to the bundled Retry page.
    // Other windows let the product display its own error and Retry action.
    const productError =
      !window.__atlasPrimaryOpening &&
      document.querySelector(".backend-status [role=alert]")?.getClientRects()
        .length;
    if (!productReady && !productError) return;
    release();
    if (!window.__atlasPrimaryOpening && productReady) rememberOpening();
  }
  const observer = new MutationObserver(() => {
    if (!cover.isConnected && !mount()) return;
    releaseWhenMounted();
  });
  observer.observe(document, {
    subtree: true,
    childList: true,
    attributes: true,
  });
  mount();
  window.addEventListener("atlas-opening-committed", rememberOpening, {
    once: true,
  });
})();
