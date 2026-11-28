import { createDesktopTabHistory } from "./desktopTabHistory";

it("runs the registered route blocker for Back and Forward with the destination entry's state", async () => {
  const history = createDesktopTabHistory("/parties");
  history.push("/characters/ada", { atlasReadingDismissal: "draft origin" });
  const guard = vi.fn().mockResolvedValue(true);
  history.block({ blockerFn: guard });
  history.back();
  await vi.waitFor(() => expect(guard).toHaveBeenCalledOnce());
  expect(history.location.href).toBe("/characters/ada");
  expect(guard.mock.lastCall![0]).toMatchObject({
    currentLocation: { href: "/characters/ada" },
    nextLocation: { href: "/parties" },
    action: "BACK",
  });

  guard.mockResolvedValue(false);
  history.back();
  await vi.waitFor(() => expect(history.location.href).toBe("/parties"));
  history.forward();
  await vi.waitFor(() => expect(history.location.href).toBe("/characters/ada"));
  expect(guard.mock.lastCall![0]).toMatchObject({
    nextLocation: { state: { atlasReadingDismissal: "draft origin" } },
    action: "FORWARD",
  });
});

it("checks one traversal at a time and abandons it if a pending save navigates", async () => {
  const history = createDesktopTabHistory("/characters");
  history.push("/characters/ada");
  let finishSave!: (blocked: boolean) => void;
  const save = new Promise<boolean>((resolve) => {
    finishSave = resolve;
  });
  const guard = vi.fn(() => save);
  history.block({ blockerFn: guard });
  history.back();
  history.back();
  expect(guard).toHaveBeenCalledTimes(1);
  expect(history.location.href).toBe("/characters/ada");
  history.replace("/parties", undefined, { ignoreBlocker: true });
  finishSave(false);
  await save;
  expect(history.location.href).toBe("/parties");
});

it("abandons a traversal when its tab is reactivated while the guard is pending", async () => {
  let activation = 1;
  const history = createDesktopTabHistory("/characters", () => activation);
  history.push("/characters/ada");
  let finish!: (blocked: boolean) => void;
  const pending = new Promise<boolean>((resolve) => {
    finish = resolve;
  });
  const guard = vi.fn(() => pending);
  history.block({ blockerFn: guard });
  history.back();
  expect(guard).toHaveBeenCalledOnce();
  activation = 3;
  finish(false);
  await pending;
  expect(history.location.href).toBe("/characters/ada");
  history.back();
  await vi.waitFor(() => expect(history.location.href).toBe("/characters"));
});
