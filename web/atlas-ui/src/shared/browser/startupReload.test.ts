import {
  clearStartupReloadAttempt,
  reloadMismatchedFrontendOnce,
} from "./startupReload";
import { reloadFrontend } from "./reloadFrontend";
vi.mock("./reloadFrontend", () => ({ reloadFrontend: vi.fn() }));

beforeEach(() => {
  window.sessionStorage.clear();
  vi.clearAllMocks();
});
afterEach(() => vi.restoreAllMocks());

it("limits automatic recovery across page loads and rearms only after success", () => {
  reloadMismatchedFrontendOnce();
  reloadMismatchedFrontendOnce();
  expect(reloadFrontend).toHaveBeenCalledTimes(1);
  clearStartupReloadAttempt();
  reloadMismatchedFrontendOnce();
  expect(reloadFrontend).toHaveBeenCalledTimes(2);
});

it("keeps recovery manual when storage cannot enforce the reload limit", () => {
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("storage unavailable");
  });
  reloadMismatchedFrontendOnce();
  expect(reloadFrontend).not.toHaveBeenCalled();
});
