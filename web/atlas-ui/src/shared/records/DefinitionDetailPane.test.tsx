import { fireEvent, render, screen } from "@testing-library/react";
import { ConfigProvider } from "antd";
import type { DefinitionDetailView } from "../../generated/atlas";
import { DefinitionAttribution, DefinitionDetailPane } from "./DefinitionDetailPane";
import { rootNavigation } from "./useReferenceReading";

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

    notices: [],
    related_filters: [],
    publications: [{ title: "Player Core", page: 456, license: "ORC" }],
  };
}

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
);

it("formats publication title and page for reading", () => {
  render(<DefinitionAttribution detail={definition()} />, { wrapper });
  expect(screen.getByText("Source: Player Core, p. 456")).toBeVisible();
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
