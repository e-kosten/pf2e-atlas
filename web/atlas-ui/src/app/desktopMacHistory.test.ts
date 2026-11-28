import { createBrowserHistory } from "@tanstack/react-router";
import { waitFor } from "@testing-library/react";
import { connectMacHistory } from "./desktopMacHistory";

it("settles a rejected native move after the compensating pop, even when a reply is lost", async () => {
  window.history.replaceState(null, "", "/search");
  const history = createBrowserHistory();
  const guard = vi.fn().mockResolvedValue(true);
  let failedStatus = false;
  let finishes = 0;
  const invoke = vi.fn(async (_name, { request }) => {
    if (
      !failedStatus &&
      request.action === "status" &&
      location.pathname === "/search" &&
      history.location.pathname === "/lists"
    ) {
      failedStatus = true;
      throw new Error("status reply lost");
    }
    if (request.action === "finish" && ++finishes === 1) {
      throw new Error("finish reply lost");
    }
    return { back: false, forward: false };
  });
  vi.spyOn(console, "error").mockImplementation(() => {});
  vi.stubGlobal("__ATLAS_MAC_HISTORY__", true);
  vi.stubGlobal("__ATLAS_DESKTOP_SETUP__", { token: "test-workspace" });
  vi.stubGlobal("__TAURI__", { core: { invoke } });
  try {
    connectMacHistory(history);
    history.push("/lists");
    history.flush();
    history.block({ blockerFn: guard });
    // Simulate the raw WebKit entry movement before the Router receives popstate.
    window.history.back();
    await waitFor(() => expect(guard).toHaveBeenCalledOnce());
    await waitFor(() => expect(finishes).toBe(2), { timeout: 2500 });
    expect(failedStatus).toBe(true);
    expect(location.pathname).toBe("/lists");
    expect(history.location.pathname).toBe("/lists");
    const finishRequests = invoke.mock.calls
      .map(([, args]) => args.request)
      .filter((request) => request.action === "finish");
    expect(finishRequests).toEqual([
      { action: "finish", sequence: 2 },
      { action: "finish", sequence: 2 },
    ]);
  } finally {
    history.destroy();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  }
});
