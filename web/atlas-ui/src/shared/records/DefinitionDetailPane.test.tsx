import { fireEvent, render, screen, within } from "@testing-library/react";
import { ConfigProvider } from "antd";
import type { DefinitionDetailView } from "../../generated/atlas";
import { DefinitionAttribution, DefinitionDetailPane } from "./DefinitionDetailPane";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import { rootNavigation } from "./useReferenceReading";
import { detailFixture } from "../../test/fixtures";

function definition(key = "trait:fire"): DefinitionDetailView {
  return {
    artifact_generation: "generation-a",
    definition_key: key,
    label: "Fire",
    help: {
      definition_key: key,

      destination: { kind: "definition", key },
    },
    content: {
      body: { kind: "plain", text: "Fire effects deal fire damage." },
      references: [],
    },

    notices: [
      {
        id: "source-notice",
        text: "Internal complete legal notice",
        source_url: "https://example.com/notice",
      },
    ],

    related_filters: [
      {
        label: "With this trait",

        predicate: {
          kind: "in",
          clause_id: "trait",
          field: "traits",
          values: ["fire"],
        },
      },
    ],
    publications: [{ title: "Player Core", page: 456, license: "ORC" }],
  };
}

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
);

it("renders trait Drawer content with its natural header and no related actions or technical sources", () => {
  const detail = definition();
  render(
    <RecordReferenceDrawer
      destination={{ kind: "definition", key: detail.definition_key }}
      definition={detail}
      help={detail.help ?? undefined}
      anchor={null}
      detail={undefined}
      loading={false}
      onClose={vi.fn()}
      onReference={vi.fn()}
    />,
    { wrapper },
  );
  const drawer = screen.getByRole("dialog", { name: "Fire" });
  const header = within(
    screen.getByRole("heading", { name: "Fire" }).closest("header")!,
  );
  expect(header.getByText("Trait", { exact: true })).toBeVisible();
  expect(header.getByText("Source: Player Core, p. 456")).toBeVisible();
  expect(within(drawer).getByText("Fire effects deal fire damage.")).toBeVisible();
  expect(within(drawer).queryByRole("link", { name: "With this trait" })).toBeNull();
  expect(within(drawer).queryByRole("button", { name: /Explore/ })).toBeNull();
  expect(within(drawer).queryByText("Source and notices")).toBeNull();
  expect(
    within(drawer).queryByText(
      /internal-|scripts\/config|Internal complete legal notice/,
    ),
  ).toBeNull();
  expect(within(drawer).queryByRole("heading", { name: "References" })).toBeNull();
  expect(
    within(drawer).getByRole("link", { name: "Open reference full page" }),
  ).toHaveAttribute("href", "/traits/trait:fire");
});

it("renders prepared nested reference markers through their existing exact ordinals", () => {
  const detail = definition("language:chthonian");
  detail.content = {
    body: {
      kind: "html",
      html: '<p>Language context with <span data-atlas-reference="7">Darkvision</span>.</p>',
      inclusions: [],
      controls: [],
    },
    references: [
      {
        ordinal: 7,
        status: "resolved_record",
        target: rootNavigation("abilities:darkvision"),
        url: null,
      },
    ],
  };
  const open = vi.fn();
  render(<DefinitionDetailPane detail={detail} loading={false} onReference={open} />, {
    wrapper,
  });
  const link = screen.getByRole("link", { name: "Darkvision" });
  expect(link).toHaveAttribute("href", "/records/abilities:darkvision");
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith(
    "abilities:darkvision",
    link,
    rootNavigation("abilities:darkvision"),
  );
});

it("keeps absent publication attribution and unavailable bodies quiet", () => {
  const detail = definition("language:custom");
  detail.publications = [];
  detail.help = null;
  detail.content = null;
  const { container } = render(
    <>
      <DefinitionAttribution detail={detail} />
      <DefinitionDetailPane
        detail={detail}
        identityInHost
        loading={false}
        onReference={vi.fn()}
      />
    </>,
    { wrapper },
  );
  expect(container.textContent).toBe("");
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.queryByText("Source: pf2e")).toBeNull();
});

it("preserves ordinary record actions while keeping general-definition reading free of property controls", () => {
  const detail = detailFixture("abilities:darkvision", "Darkvision");
  const help = definition().help;
  render(
    <RecordReferenceDrawer
      destination={{
        kind: "record",
        navigation: detail.selected,
      }}
      help={help ?? undefined}
      anchor={null}
      detail={detail}
      loading={false}
      actions={<button>Ordinary record action</button>}
      onClose={vi.fn()}
      onReference={vi.fn()}
    />,
    { wrapper },
  );
  expect(screen.getByRole("button", { name: "Ordinary record action" })).toBeVisible();
  expect(screen.queryByRole("link", { name: "With this trait" })).toBeNull();
  expect(screen.queryByRole("button", { name: /Explore/ })).toBeNull();
});
