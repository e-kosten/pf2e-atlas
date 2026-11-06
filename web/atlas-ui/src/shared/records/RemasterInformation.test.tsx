import { fireEvent, render, screen, within } from "@testing-library/react";
import {
  remasterDeclarationFixture,
  remasterInformationFixture,
} from "../../test/remasterFixtures";
import { summaryFixture } from "../../test/fixtures";
import type { RemasterRelationshipsView } from "../../generated/atlas";
import { RemasterInformation, SourceRemasterInformation } from "./RemasterInformation";
import { rootNavigation } from "./useReferenceReading";

it("renders one compact inline current destination with native/drawer behavior and no source attribution", () => {
  const content = remasterInformationFixture();
  content.relationships[0] = {
    ...content.relationships[0],
    label: "Renamed to",
    parts: [
      {
        kind: "reference",
        label: "Reactive Strike",
        navigation: rootNavigation("glossary:reactive-strike"),
      },
    ],
  };
  content.declarations[0].citations[0].origin = "atlas_reviewed";
  content.declarations[0].citations[0].attribution = "Atlas mapping";
  const open = vi.fn();
  render(
    <RemasterInformation
      compact
      category={{ kind: "content", content }}
      onReference={open}
    />,
  );
  const line = screen.getByText("Renamed to:").closest("p")!;
  expect(line).toHaveTextContent(/^Renamed to: Reactive Strike$/);
  const link = within(line).getByRole("link", { name: "Reactive Strike" });
  expect(link).toHaveAttribute("href", "/records/glossary:reactive-strike");
  fireEvent.click(link, { metaKey: true });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith(
    "glossary:reactive-strike",
    link,
    rootNavigation("glossary:reactive-strike"),
  );
  expect(screen.queryByText("Atlas mapping")).toBeNull();
  expect(screen.queryByText("Relationship added by Atlas")).toBeNull();
  expect(screen.queryByText("Sources")).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.queryByRole("button", { name: "Compare" })).toBeNull();
});

it("prints the server's single Cleanse relationship with three plain absent names and retains all five supports in Sources", async () => {
  const declarations = Array.from({ length: 5 }, (_, index) => ({
    ...remasterDeclarationFixture(),
    id: `support-${index}`,
    citations: [
      {
        ...remasterDeclarationFixture().citations[0],
        statement: `Supporting source statement ${index + 1}`,
      },
    ],
  }));
  const content: RemasterRelationshipsView = {
    relationships: [
      {
        id: "cleanse",
        label: "Replaces",
        parts: [
          { kind: "text", text: "Remove Curse, Remove Disease, and Neutralize Poison" },
        ],
        declaration_ids: declarations.map((item) => item.id),
        truncated: false,
      },
    ],
    declarations,
    truncated: false,
  };
  render(
    <RemasterInformation
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  const line = screen.getByText("Replaces:").closest("p")!;
  expect(line).toHaveTextContent(
    /^Replaces: Remove Curse, Remove Disease, and Neutralize Poison$/,
  );
  expect(within(line).queryByRole("link")).toBeNull();
  expect(screen.getAllByText("Replaces:")).toHaveLength(1);
  expect(screen.queryByText("Supporting source statement 1")).toBeNull();
  fireEvent.click(screen.getByText("Sources"));
  for (let index = 1; index <= 5; index++)
    expect(
      await screen.findByText(`Supporting source statement ${index}`),
    ).toBeVisible();
});

it("shows Nephilim once as a collective merger and keeps raw source fragments out of the primary view", () => {
  const content = remasterInformationFixture();
  content.relationships[0] = {
    ...content.relationships[0],
    label: "Merged from",
    parts: [{ kind: "text", text: "Aasimar, Aphorite, Ganzi, and Tiefling" }],
  };
  content.declarations[0].authored_status = "are merged into";
  content.declarations[0].context = ["Remaster Changes", "rules"];
  render(
    <RemasterInformation
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByText("Merged from:").closest("p")).toHaveTextContent(
    /^Merged from: Aasimar, Aphorite, Ganzi, and Tiefling$/,
  );
  expect(screen.queryByText("are merged into")).toBeNull();
  expect(screen.queryByText("Remaster Changes · rules")).toBeNull();
  expect(screen.queryByRole("heading", { name: "Before" })).toBeNull();
  expect(screen.queryByRole("heading", { name: "After" })).toBeNull();
});

it("keeps distinct projected scopes, grouped operands, mixed plain/available names and truthful truncation", () => {
  const content = remasterInformationFixture();
  content.relationships = [
    {
      ...content.relationships[0],
      id: "scope-1",
      label: "Replaces (Fighter)",
      parts: [
        { kind: "text", text: "(Unavailable one and " },
        {
          kind: "reference",
          label: "Available current name",
          navigation: rootNavigation("rules:current"),
        },
        { kind: "text", text: ")" },
      ],
      truncated: true,
    },
    {
      ...content.relationships[0],
      id: "scope-2",
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
    "Replaces (Fighter): (Unavailable one and Available current name)",
  );
  expect(screen.getByText("Replaces (Ranger):").closest("p")).toHaveTextContent(
    /^Replaces \(Ranger\): Unavailable one$/,
  );
  expect(screen.getByRole("link", { name: "Available current name" })).toHaveAttribute(
    "href",
    "/records/rules:current",
  );
  expect(
    screen.getByText("More relationship information exists in the sources."),
  ).toBeVisible();
  expect(
    screen.getByText("More remaster information exists in the sources."),
  ).toBeVisible();
});

it("discloses exact source origins, source availability, authored/current distinctions and bounded evidence", async () => {
  const content = remasterInformationFixture();
  const declaration = content.declarations[0];
  declaration.before[0].endpoints = [
    {
      label: "Earlier authored name",
      context: "Earlier book",
      resolution: {
        kind: "current",
        record: summaryFixture("rules:current", "Current title"),
        navigation: rootNavigation("rules:current"),
      },
    },
    {
      label: "Absent explicit key",
      context: null,
      resolution: {
        kind: "absent",
        record_key: "rules:absent",
        message: "This identified record is absent.",
      },
    },
    {
      label: "Unresolved term",
      context: null,
      resolution: {
        kind: "unresolved",
        reference: null,
        message: "No record has been established for this term.",
      },
    },
    {
      label: "Ambiguous term",
      context: null,
      resolution: {
        kind: "ambiguous",
        message: "Possible matches; no verified replacement.",
        candidates: [
          {
            record: summaryFixture("rules:possible", "Possibility"),
            navigation: rootNavigation("rules:possible"),
          },
        ],
        truncated: true,
      },
    },
  ];
  declaration.citations.push({
    ...declaration.citations[0],
    origin: "atlas_reviewed",
    attribution: "Atlas mapping",
    shortened: true,
    destination: { kind: "external", url: "https://example.com/pinned-source" },
  });
  declaration.citations_truncated = true;
  declaration.truncated = true;
  render(
    <RemasterInformation
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByText("Sources"));
  expect(await screen.findByText("Relationship added by Atlas")).toBeVisible();
  expect(screen.getByRole("link", { name: "Atlas mapping" })).toHaveAttribute(
    "href",
    "https://example.com/pinned-source",
  );
  expect(screen.getByText("Earlier authored name").closest("a")).toBeNull();
  expect(
    screen.getByRole("link", { name: "Open current entry: Current title" }),
  ).toHaveAttribute("href", "/records/rules:current");
  expect(screen.getByText("This identified record is absent.")).toBeVisible();
  expect(
    screen.getByText("No record has been established for this term."),
  ).toBeVisible();
  expect(screen.getByText("Possible matches; no verified replacement.")).toBeVisible();
  expect(screen.getByText("More supporting sources exist.")).toBeVisible();
  expect(
    screen.getByText("Additional details exist in this source statement."),
  ).toBeVisible();
  expect(screen.getByText(/excerpt shortened/)).toBeVisible();
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
])("keeps source states independent of readable owned content: $kind", (category) => {
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

it("keeps declaration origins visible in Sources when citation budget is exhausted", async () => {
  const content = remasterInformationFixture();
  content.declarations[0].origins = ["foundry", "atlas_reviewed"];
  content.declarations[0].citations = [];
  content.declarations[0].citations_truncated = true;
  render(
    <RemasterInformation
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.queryByText("Relationship added by Atlas")).toBeNull();
  fireEvent.click(screen.getByText("Sources"));
  expect(await screen.findByText("Relationship added by Atlas")).toBeVisible();
  expect(screen.getByText("Foundry source")).toBeVisible();
  expect(screen.getByText("More supporting sources exist.")).toBeVisible();
});
it("renders a projected complete fact without a trailing colon when there are no operands", () => {
  const content = remasterInformationFixture();
  content.relationships[0] = {
    ...content.relationships[0],
    label: "Removed in the remaster",
    parts: [],
  };
  render(
    <RemasterInformation
      compact
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByText("Removed in the remaster").closest("p")).toHaveTextContent(
    /^Removed in the remaster$/,
  );
});
