import { render, screen, fireEvent } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BackendCompatibilityGate } from "./BackendCompatibilityGate";
import { AtlasCompatibilityError } from "../api/compatibility";
import { verifyBackendCompatibility } from "../api/atlasApi";
vi.mock("../api/atlasApi", () => ({ verifyBackendCompatibility: vi.fn() }));

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
});
