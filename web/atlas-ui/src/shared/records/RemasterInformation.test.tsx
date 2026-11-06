import { fireEvent, render, screen } from "@testing-library/react";
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
