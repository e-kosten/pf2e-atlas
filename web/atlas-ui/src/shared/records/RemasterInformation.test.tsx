import { fireEvent, render, screen, within } from "@testing-library/react";
import {
  remasterDeclarationFixture,
  remasterInformationFixture,
} from "../../test/remasterFixtures";
import { summaryFixture } from "../../test/fixtures";
import { RemasterInformation, SourceRemasterInformation } from "./RemasterInformation";
import { rootNavigation } from "./useReferenceReading";

it("uses service notices without deriving direction or current names from declaration sides", () => {
  const content = remasterInformationFixture();
  content.notices[0] = {
    ...content.notices[0],
    heading: "These entries were merged",
    attribution: "Atlas-reviewed correspondence",
    links: [
      { label: "Current shared entry", navigation: rootNavigation("rules:shared") },
    ],
  };
  render(
    <RemasterInformation
      compact
      category={{ kind: "content", content }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByText("These entries were merged")).toBeVisible();
  expect(screen.getByText("Atlas-reviewed correspondence")).toBeVisible();
  expect(screen.getByRole("link", { name: "Current shared entry" })).toHaveAttribute(
    "href",
    "/records/rules:shared",
  );
  expect(screen.queryByRole("heading", { name: "Before" })).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
});

it("preserves grouped sides, same-key authored names, uncertainty, all sources and truthful bounds", () => {
  const declaration = remasterDeclarationFixture();
  declaration.kinds = ["renamed", "mechanics_changed", "discussion"];
  declaration.before[0].endpoints = [
    {
      label: "Earlier authored name",
      context: "Earlier book",
      resolution: {
        kind: "current",
        record: summaryFixture("rules:new", "Current name"),
        navigation: rootNavigation("rules:new"),
      },
    },
    {
      label: "Removed explicit entry",
      context: null,
      resolution: {
        kind: "absent",
        record_key: "rules:removed",
        message: "This identified record is absent from this collection.",
      },
    },
    {
      label: "Unbound term",
      context: null,
      resolution: {
        kind: "unresolved",
        reference: null,
        message: "This term could not be resolved.",
      },
    },
    {
      label: "Possible entry",
      context: null,
      resolution: {
        kind: "ambiguous",
        message: "Possible matches; no replacement is verified.",
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
  declaration.after.push({
    text: "A separate discussion group",
    terms: ["Authored term"],
    endpoints: [],
    truncated: true,
  });
  declaration.citations.push({
    ...declaration.citations[0],
    origin: "atlas_reviewed",
    attribution: "Atlas glossary review",
    shortened: true,
    destination: { kind: "external", url: "https://example.com/pinned-source" },
  });
  declaration.citations_truncated = true;
  declaration.truncated = true;
  const open = vi.fn();
  render(
    <RemasterInformation
      category={{
        kind: "content",
        content: { notices: [], declarations: [declaration], truncated: true },
      }}
      onReference={open}
    />,
  );
  expect(
    screen.getAllByRole("heading", { level: 4 }).map((node) => node.textContent),
  ).toEqual(["Before", "After"]);
  expect(screen.getByText("Earlier authored name").closest("a")).toBeNull();
  expect(screen.queryByRole("link", { name: "Earlier authored name" })).toBeNull();
  const old = screen.getByRole("link", { name: "Open current entry: Current name" });
  expect(old).toHaveAttribute("href", "/records/rules:new");
  fireEvent.click(old);
  expect(open).toHaveBeenCalledWith("rules:new", old, rootNavigation("rules:new"));
  expect(screen.getByText(/identified record is absent/)).toBeVisible();
  expect(screen.getByText(/could not be resolved/)).toBeVisible();
  expect(screen.getByText(/no replacement is verified/)).toBeVisible();
  expect(screen.getByText("A separate discussion group")).toBeVisible();
  fireEvent.click(screen.getByText("Sources (2)"));
  expect(screen.getByText("Foundry source")).toBeVisible();
  expect(screen.getByText("Atlas-reviewed correspondence")).toBeVisible();
  expect(screen.getByRole("link", { name: "Atlas glossary review" })).toHaveAttribute(
    "href",
    "https://example.com/pinned-source",
  );
  expect(screen.getByText(/excerpt shortened/)).toBeVisible();
  expect(screen.getByText("More supporting sources exist.")).toBeVisible();
  expect(
    screen.getByText("More remaster information exists in the sources."),
  ).toBeVisible();
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
])(
  "keeps pre-origin source states independent of direct record information: $kind",
  (category) => {
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
  },
);

const unresolvedMessage =
  "The source does not establish an available record for this name or reference";
function showGroups(
  before: import("../../generated/atlas").RemasterSideView[],
  after: import("../../generated/atlas").RemasterSideView[] = [],
) {
  render(
    <RemasterInformation
      category={{
        kind: "content",
        content: {
          notices: [],
          declarations: [
            { ...remasterDeclarationFixture(), summary: null, before, after },
          ],
          truncated: false,
        },
      }}
      onReference={vi.fn()}
    />,
  );
  return within(screen.getByRole("heading", { name: "Before" }).parentElement!);
}
function unresolved(
  label: string,
): import("../../generated/atlas").RemasterEndpointView {
  return {
    label,
    context: null,
    resolution: { kind: "unresolved", reference: null, message: unresolvedMessage },
  };
}
it("shows the live Off-Guard shape once per side, retaining the exact current link", () => {
  const before = showGroups(
    [
      {
        text: "Flat-Footed",
        terms: ["Flat-Footed"],
        endpoints: [unresolved("Flat-Footed")],
        truncated: false,
      },
    ],
    [
      {
        text: "Off-Guard",
        terms: ["Off-Guard"],
        endpoints: [
          {
            label: "Off-Guard",
            context: null,
            resolution: {
              kind: "current",
              record: summaryFixture("conditionitems:AJh5ex99aV6VTggg", "Off-Guard"),
              navigation: rootNavigation("conditionitems:AJh5ex99aV6VTggg"),
            },
          },
        ],
        truncated: false,
      },
    ],
  );
  expect(before.getAllByText("Flat-Footed")).toHaveLength(1);
  expect(screen.getAllByText("Off-Guard")).toHaveLength(1);
  expect(screen.getByRole("link", { name: "Off-Guard" })).toHaveAttribute(
    "href",
    "/records/conditionitems:AJh5ex99aV6VTggg",
  );
  expect(screen.queryByText("Source wording")).toBeNull();
});
it("compresses the live four-predecessor Nephilim shape while preserving exact authored wording and evidence", () => {
  const names = ["Aasimar", "Aphorite", "Ganzi", "Tiefling"];
  const wording = "Aasimar, Aphorite, Ganzi, and Tiefling";
  const before = showGroups(
    [
      {
        text: wording,
        terms: names,
        endpoints: names.map(unresolved),
        truncated: true,
      },
    ],
    [
      {
        text: "Nephilim .",
        terms: ["Nephilim"],
        endpoints: [
          {
            label: "Nephilim",
            context: null,
            resolution: {
              kind: "current",
              record: summaryFixture("heritages:1oLMOmLpurfWTTff", "Nephilim"),
              navigation: rootNavigation("heritages:1oLMOmLpurfWTTff"),
            },
          },
        ],
        truncated: false,
      },
    ],
  );
  for (const name of names) {
    expect(before.getAllByText(name, { exact: true })).toHaveLength(1);
    expect(before.getByText(name, { exact: true })).toBeVisible();
  }
  expect(before.getAllByText(unresolvedMessage)).toHaveLength(1);
  expect(before.getByText("More entries exist in this source group.")).toBeVisible();
  expect(screen.getByRole("link", { name: "Nephilim" })).toHaveAttribute(
    "href",
    "/records/heritages:1oLMOmLpurfWTTff",
  );
  expect(before.queryByText(wording)).toBeNull();
  expect(screen.queryByText("Nephilim .")).toBeNull();
  expect(screen.getByText("Sources (1)")).toBeVisible();
});
it("preserves mixed resolution meanings, distinct terms and authored/current names without merging groups", () => {
  const before = showGroups([
    {
      text: "Different source wording",
      terms: ["Old name", "Old name", "old name", "Distinct term"],
      endpoints: [
        {
          label: "Old name",
          context: "Earlier book",
          resolution: {
            kind: "current",
            record: summaryFixture("rules:new", "Current title"),
            navigation: rootNavigation("rules:new"),
          },
        },
        unresolved("Unresolved name"),
        {
          label: "Explicit absent key",
          context: null,
          resolution: {
            kind: "absent",
            record_key: "rules:absent",
            message: unresolvedMessage,
          },
        },
        {
          label: "Different uncertainty",
          context: null,
          resolution: {
            kind: "unresolved",
            reference: null,
            message: "This other reference requires review.",
          },
        },
      ],
      truncated: false,
    },
    {
      text: "Second group",
      terms: [],
      endpoints: [unresolved("Another name"), unresolved("One more name")],
      truncated: false,
    },
  ]);
  expect(before.getAllByText("Old name", { exact: true })).toHaveLength(1);
  expect(before.getByText("Old name", { exact: true }).closest("a")).toBeNull();
  expect(before.getByText("old name · Distinct term")).toBeVisible();
  expect(
    before.getByRole("link", { name: "Open current entry: Current title" }),
  ).toHaveAttribute("href", "/records/rules:new");
  expect(before.getAllByText(unresolvedMessage)).toHaveLength(3);
  expect(before.getByText("This other reference requires review.")).toBeVisible();
  expect(before.getByText("Earlier book", { exact: false })).toBeVisible();
  expect(before.getByText("Different source wording")).toBeVisible();
  expect(before.getByText("Second group")).toBeVisible();
});

it("keeps meaningful removal context and case-distinct authored text visible", () => {
  const declaration = remasterDeclarationFixture();
  declaration.kinds = ["removed", "mechanics_changed"];
  declaration.summary = "Speedy Rituals";
  declaration.before = [
    {
      text: "Speedy Rituals",
      terms: ["Speedy Rituals"],
      endpoints: [unresolved("Speedy Rituals")],
      truncated: false,
    },
  ];
  declaration.after = [
    {
      text: "Folded into Efficient Rituals",
      terms: ["Efficient Rituals"],
      endpoints: [
        {
          label: "Efficient Rituals",
          context: null,
          resolution: {
            kind: "current",
            record: summaryFixture("feats:efficient", "Efficient Rituals"),
            navigation: rootNavigation("feats:efficient"),
          },
        },
      ],
      truncated: false,
    },
    {
      text: "efficient rituals",
      terms: ["Efficient Rituals"],
      endpoints: [],
      truncated: false,
    },
  ];
  render(
    <RemasterInformation
      category={{
        kind: "content",
        content: { notices: [], declarations: [declaration], truncated: false },
      }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByText("Folded into Efficient Rituals")).toBeVisible();
  expect(screen.getByText("efficient rituals", { exact: true })).toBeVisible();
  expect(screen.getByRole("link", { name: "Efficient Rituals" })).toHaveAttribute(
    "href",
    "/records/feats:efficient",
  );
  const before = within(screen.getByRole("heading", { name: "Before" }).parentElement!);
  expect(before.getAllByText("Speedy Rituals")).toHaveLength(1);
});
