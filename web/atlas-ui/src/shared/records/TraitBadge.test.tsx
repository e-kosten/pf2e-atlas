import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { RouterTestContext } from "../../test/router";
import { getDefinitionDetail } from "../../api/atlasApi";
import { ReferenceHelpHost } from "./ReferenceHelpHost";
import { ReferenceHelpProvider } from "./ReferenceHelp";
import type { DefinitionDetailView, ReferenceHelpView } from "../../generated/atlas";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { TraitBadge } from "./TraitBadge";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionDetail: vi.fn(),
}));

function readableTraitHelp(key: string): ReferenceHelpView {
  return {
    definition_key: key,

    destination: { kind: "definition", key },
  };
}
const wrapper = ({ children }: { children: React.ReactNode }) => (
  <RouterTestContext>
    <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
  </RouterTestContext>
);

it("opens exact variant definitions in reader badges", async () => {
  history.replaceState(null, "", "/search");
  const onHelp = vi.fn();
  const help = readableTraitHelp("trait:deadly-d12");
  render(
    <ReferenceHelpProvider onHelp={onHelp}>
      <TraitBadge
        trait={{
          kind: "trait",
          value: "deadly-d12",
          label: "Deadly d12",
          trait_key: "trait:deadly-d12",
          help,
        }}
      />
    </ReferenceHelpProvider>,
    { wrapper },
  );
  const link = screen.getByRole("link", { name: "Deadly d12" });
  expect(link.getAttribute("href")).toBe("/traits/trait:deadly-d12");
  expect(link.parentElement?.closest("a")).toBeNull();
  fireEvent.click(link);
  expect(onHelp).toHaveBeenCalledWith(help, link, undefined);
  expect(location.pathname).toBe("/search");
});

it("retains plain labels for namespaces without a definition destination", () => {
  render(
    <TraitBadge trait={{ kind: "other_tag", value: "custom", label: "Custom" }} />,
  );
  expect(screen.getByText("Custom")).toBeVisible();
  expect(screen.queryByRole("link")).toBeNull();
});

it("preserves native modified clicks", () => {
  history.replaceState(null, "", "/search");
  render(
    <TraitBadge
      trait={{
        kind: "trait",
        value: "agile",
        label: "Agile",
        trait_key: "trait:agile",
        help: readableTraitHelp("trait:agile"),
      }}
    />,
    { wrapper },
  );
  fireEvent.click(screen.getByRole("link"), { ctrlKey: true });
  expect(location.pathname).toBe("/search");
});

it("keeps a trait without reading help as plain text", () => {
  render(
    <TraitBadge
      trait={{
        kind: "trait",
        value: "custom",
        label: "Custom",
        trait_key: "trait:custom",
      }}
    />,
  );
  expect(screen.getByText("Custom")).toBeVisible();
  expect(screen.queryByRole("link")).toBeNull();
});

it("opens supplied prepared trait help in the shared clean Drawer with its native trait URL", async () => {
  const help = readableTraitHelp("trait:agile");
  const detail: DefinitionDetailView = {
    artifact_generation: "generation-a",
    definition_key: "trait:agile",
    label: "Agile",
    help,
    content: {
      body: { kind: "plain", text: "Multiple attack penalty is reduced." },
      references: [],
    },

    notices: [],

    related_filters: [],
    publications: [],
  };
  vi.mocked(getDefinitionDetail).mockResolvedValue(detail);
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <ReferenceHelpHost generation="generation-a">
        <TraitBadge
          trait={{
            kind: "trait",
            value: "agile",
            label: "Agile",
            trait_key: "trait:agile",
            help,
          }}
        />
      </ReferenceHelpHost>
    </QueryClientProvider>,
    { wrapper },
  );
  const opener = screen.getByRole("link", { name: "Agile" });
  expect(opener).toHaveAttribute("href", "/traits/trait:agile");
  opener.focus();
  fireEvent.click(opener);
  const drawer = await screen.findByRole("dialog", { name: "Agile" });
  expect(within(drawer).getByText("Trait", { exact: true })).toBeVisible();
  expect(within(drawer).getByText("Multiple attack penalty is reduced.")).toBeVisible();
  expect(within(drawer).queryByText("Source and notices")).toBeNull();
  expect(within(drawer).queryByRole("button", { name: /Explore/ })).toBeNull();
  expect(
    within(drawer).getByRole("link", { name: "Open reference full page" }),
  ).toHaveAttribute("href", "/traits/trait:agile");
  fireEvent.click(
    within(drawer).getByRole("button", { name: "Close reference preview" }),
  );
  await waitFor(() => expect(opener).toHaveFocus());
});
