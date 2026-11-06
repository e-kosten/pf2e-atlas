import { fireEvent, render, screen, within } from "@testing-library/react";
import { remasterInformationFixture } from "../../test/remasterFixtures";
import { RemasterInformation, SourceRemasterInformation } from "./RemasterInformation";
import { rootNavigation } from "./useReferenceReading";

it("renders useful projected explanation links directly with exact native and drawer navigation", () => {
  const content = remasterInformationFixture();
  const page = {
    ...rootNavigation("journals:changes"),
    embedded_path: [{ collection: "/pages", identity: { Stable: "spells" } }],
    field: "/text/content",
  };
  content.relationships[0].explanation_links = [
    { label: "Remaster Changes — Spells", navigation: page },
  ];
  const open = vi.fn();
  render(
    <RemasterInformation category={{ kind: "content", content }} onReference={open} />,
  );
  const explanation = screen.getByRole("link", { name: "Remaster Changes — Spells" });
  expect(explanation.getAttribute("href")).toContain("/records/journals:changes");
  expect(explanation.getAttribute("href")).toContain("at=");
  fireEvent.click(explanation, { metaKey: true });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(explanation);
  expect(open).toHaveBeenCalledWith("journals:changes", explanation, page);
  expect(screen.queryByText("Sources")).toBeNull();
  expect(screen.queryByRole("button", { name: "Compare" })).toBeNull();
});

it.each([
  ["Former name", "Magic Missile"],
  ["Replaces", "Remove Curse, Remove Disease, and Neutralize Poison"],
  ["Merged from", "Aasimar, Aphorite, Ganzi, and Tiefling"],
])(
  "keeps %s as one plain-name fact and one supplied journal explanation",
  (label, text) => {
    const content = remasterInformationFixture();
    content.relationships[0] = {
      ...content.relationships[0],
      label,
      parts: [{ kind: "text", text }],
      explanation_links: [
        {
          label: "Remaster Changes — Spells",
          navigation: rootNavigation("journals:changes"),
        },
      ],
    };
    content.declarations = Array.from({ length: 5 }, (_, index) => ({
      ...content.declarations[0],
      id: `support-${index}`,
    }));
    render(
      <RemasterInformation
        category={{ kind: "content", content }}
        onReference={vi.fn()}
      />,
    );
    const line = screen.getByText(`${label}:`).closest("p")!;
    expect(within(line).getAllByText(text)).toHaveLength(1);
    expect(screen.getAllByRole("link")).toHaveLength(1);
    expect(
      screen.getByRole("link", { name: "Remaster Changes — Spells" }),
    ).toHaveAttribute("href", "/records/journals:changes");
    expect(screen.queryByText("Sources")).toBeNull();
    expect(screen.queryByRole("heading", { name: "Before" })).toBeNull();
    expect(screen.queryByRole("heading", { name: "After" })).toBeNull();
  },
);

it("does not derive browser explanation links or provider diagnostics from canonical evidence", () => {
  const content = remasterInformationFixture();
  content.declarations[0].origins = ["foundry", "atlas_reviewed"];
  content.declarations[0].citations[0] = {
    ...content.declarations[0].citations[0],
    attribution: "Internal migration script",
    revision: "private-revision",
    destination: { kind: "external", url: "https://example.com/migration.ts" },
  };
  content.declarations[0].authored_status = "are merged into";
  render(
    <RemasterInformation
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByRole("link", { name: "New name" })).toBeVisible();
  expect(screen.getAllByRole("link")).toHaveLength(1);
  expect(
    screen.queryByText(
      /Internal migration|private-revision|Revision:|Atlas|Foundry|are merged into/,
    ),
  ).toBeNull();
  expect(screen.queryByText("Sources")).toBeNull();
  expect(document.querySelector("details")).toBeNull();
});

it("retains exact current destinations, distinct projected scopes and bounded information", () => {
  const content = remasterInformationFixture();
  content.relationships = [
    {
      ...content.relationships[0],
      id: "fighter",
      label: "Replaces (Fighter)",
      parts: [
        { kind: "text", text: "(Unavailable one and " },
        {
          kind: "reference",
          label: "Current title",
          navigation: rootNavigation("rules:current"),
        },
        { kind: "text", text: ")" },
      ],
      truncated: true,
    },
    {
      ...content.relationships[0],
      id: "ranger",
      label: "Replaces (Ranger)",
      parts: [{ kind: "text", text: "Unavailable one" }],
    },
  ];
  content.truncated = true;
  render(
    <RemasterInformation
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByText("Replaces (Fighter):").closest("p")).toHaveTextContent(
    "Replaces (Fighter): (Unavailable one and Current title)",
  );
  expect(screen.getByText("Replaces (Ranger):").closest("p")).toHaveTextContent(
    /^Replaces \(Ranger\): Unavailable one$/,
  );
  expect(screen.getByRole("link", { name: "Current title" })).toHaveAttribute(
    "href",
    "/records/rules:current",
  );
  expect(screen.getByText("More relationship information is available.")).toBeVisible();
  expect(screen.getByText("More remaster information is available.")).toBeVisible();
});

it.each([
  { kind: "empty" as const },
  {
    kind: "unavailable" as const,
    reason: "The selected entry has conflicting source origins.",
  },
  {
    kind: "error" as const,
    error: { code: "record_not_found" as const, message: "Unavailable" },
  },
])("keeps source state local beside readable owned rules: $kind", (category) => {
  render(
    <>
      <p>Owned rules remain readable.</p>
      <SourceRemasterInformation category={category} onReference={vi.fn()} />
    </>,
  );
  expect(screen.getByText("Owned rules remain readable.")).toBeVisible();
  expect(screen.queryByRole("link")).toBeNull();
  if (category.kind === "unavailable")
    expect(screen.getByText(category.reason)).toBeVisible();
  if (category.kind === "error")
    expect(
      screen.getByText(/Source-entry remaster information could not be checked/),
    ).toBeVisible();
});
