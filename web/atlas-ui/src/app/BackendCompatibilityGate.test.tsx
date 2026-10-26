import { render, screen, fireEvent } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BackendCompatibilityGate } from "./BackendCompatibilityGate";
import { AtlasCompatibilityError } from "../api/compatibility";
import { verifyBackendCompatibility } from "../api/atlasApi";
import { reloadFrontend } from "../shared/browser/reloadFrontend";
vi.mock("../shared/browser/reloadFrontend", () => ({ reloadFrontend: vi.fn() }));
vi.mock("../api/atlasApi", () => ({ verifyBackendCompatibility: vi.fn() }));

beforeEach(() => {
  window.sessionStorage.clear();
  vi.clearAllMocks();
});
afterEach(() => vi.unstubAllEnvs());

it("holds feature hooks until a successful check and retries a mismatch without navigation", async () => {
  let allow: (value: true) => void = () => {};
  vi.mocked(verifyBackendCompatibility)
    .mockRejectedValueOnce(new AtlasCompatibilityError(null))
    .mockImplementationOnce(
      () =>
        new Promise<true>((resolve) => {
          allow = resolve;
        }),
    );
  const mount = vi.fn();
  function Feature() {
    mount();
    return <div>Feature ready</div>;
  }
  render(
    <QueryClientProvider client={new QueryClient()}>
      <BackendCompatibilityGate>
        <Feature />
      </BackendCompatibilityGate>
    </QueryClientProvider>,
  );
  await screen.findByText("Atlas versions don’t match");
  expect(mount).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(mount).not.toHaveBeenCalled();
  allow(true);
  await screen.findByText("Feature ready");
  expect(reloadFrontend).not.toHaveBeenCalled();
});

it("automatically reloads once in production, then offers a manual reload", async () => {
  vi.stubEnv("PROD", true);
  vi.stubEnv("DEV", false);
  vi.mocked(verifyBackendCompatibility).mockRejectedValue(
    new AtlasCompatibilityError("other"),
  );
  const renderGate = () =>
    render(
      <QueryClientProvider client={new QueryClient()}>
        <BackendCompatibilityGate>
          <div>Feature ready</div>
        </BackendCompatibilityGate>
      </QueryClientProvider>,
    );
  const first = renderGate();
  await screen.findByText("Atlas versions don’t match");
  expect(reloadFrontend).toHaveBeenCalledTimes(1);
  first.unmount();
  renderGate();
  await screen.findByText("Atlas versions don’t match");
  expect(reloadFrontend).toHaveBeenCalledTimes(1);
  expect(screen.queryByText("Feature ready")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Reload page" }));
  expect(reloadFrontend).toHaveBeenCalledTimes(2);
});
