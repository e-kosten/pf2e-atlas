// Anchor the delay and the animation to the same clock in both documents.
function scheduleOpeningStatus(status, startedAt) {
  const revealAt = startedAt + 500;
  const show = () => {
    const elapsed = Math.max(0, Date.now() - revealAt);
    status.style.setProperty("--opening-phase", `-${elapsed}ms`);
    status.hidden = false;
  };
  const remaining = revealAt - Date.now();
  if (remaining > 0) return setTimeout(show, remaining);
  show();
}
