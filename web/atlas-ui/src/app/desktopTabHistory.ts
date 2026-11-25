import { createMemoryHistory, type RouterHistory } from "@tanstack/react-router";

/** TanStack's memory history runs its registered blockers for pushes and
 * replacements. Browser history also runs them for traversal via popstate;
 * a Windows tab has no browser popstate event. */
export function createDesktopTabHistory(href: string): RouterHistory {
  const history = createMemoryHistory({ initialEntries: [href] });
  const locations = [history.location];
  history.subscribe(({ action, location }) => {
    if (action.type === "PUSH" || action.type === "REPLACE") {
      locations[location.state.__TSR_index] = location;
      locations.length = history.length;
    }
  });

  const { back, forward, go } = history;
  let checking = false;
  async function traverse(
    offset: number,
    action: "BACK" | "FORWARD" | "GO",
    move: () => void,
    ignoreBlocker = false,
  ) {
    const currentLocation = history.location;
    const index = currentLocation.state.__TSR_index;
    const target = Math.min(Math.max(index + offset, 0), history.length - 1);
    if (index === target) return;
    if (ignoreBlocker) {
      move();
      return;
    }
    if (checking) return;

    checking = true;
    try {
      for (const { blockerFn } of history._getBlockers()) {
        if (
          await blockerFn({ currentLocation, nextLocation: locations[target], action })
        )
          return;
        // A pending save can navigate while the decision is outstanding.
        if (history.location !== currentLocation) return;
      }
      move();
    } finally {
      checking = false;
    }
  }
  history.back = (opts) => void traverse(-1, "BACK", back, opts?.ignoreBlocker);
  history.forward = (opts) => void traverse(1, "FORWARD", forward, opts?.ignoreBlocker);
  history.go = (offset, opts) =>
    void traverse(offset, "GO", () => go(offset), opts?.ignoreBlocker);
  return history;
}
