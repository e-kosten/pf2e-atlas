import { ConfigProvider } from "antd";
import { TableRollControl } from "./TableRollControl";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SpellReference } from "./SpellReference";
import { RecordPresentation, ContentFields } from "./RecordPresentation";
import { ActorActivity } from "./ActorReference";
import {
  summaryFixture,
  activityFixture,
  knownFact,
  numberFact,
  spellFixture,
  unavailableFact,
} from "../../test/fixtures";

it("keeps ritual checks and zero casters visible and counteraction quiet only when false", () => {
  const spell = {
    ...spellFixture(),
    ritual: knownFact({
      primary_check: knownFact("Religion (expert) or Occultism (master)"),
      secondary_casters: numberFact(0),
      secondary_checks: knownFact("Medicine; Society"),
    }),
    counteraction: knownFact(true),
  };
  const { rerender } = render(<SpellReference spell={spell} />);
  expect(screen.getByText("Primary check").parentElement).toHaveTextContent(
    "Religion (expert) or Occultism (master)",
  );
  expect(screen.getByText("Secondary casters").parentElement).toHaveTextContent("0");
  expect(screen.getByText("Secondary checks").parentElement).toHaveTextContent(
    "Medicine; Society",
  );
  expect(screen.getByText("Counteract").parentElement).toHaveTextContent("Yes");
  rerender(<SpellReference spell={{ ...spell, counteraction: knownFact(false) }} />);
  expect(screen.queryByText("Counteract")).toBeNull();
  rerender(
    <SpellReference spell={{ ...spell, counteraction: unavailableFact("invalid") }} />,
  );
  expect(screen.getByText("Counteract").parentElement).toHaveTextContent("[invalid]");
});

it("distinguishes an unspecified ritual caster count from zero and keeps empty checks honest", () => {
  render(
    <SpellReference
      spell={{
        ...spellFixture(),
        ritual: knownFact({
          primary_check: knownFact("Arcana (legendary) or Occultism (legendary)"),
          secondary_casters: { ...numberFact(0), state: "null", value: null },
          secondary_checks: knownFact(""),
        }),
      }}
    />,
  );
  expect(screen.getByText("Secondary casters").parentElement).toHaveTextContent(
    "Not specified",
  );
  expect(screen.getByText("Secondary checks").parentElement).toHaveTextContent(
    "None listed",
  );
});

it("shows prerequisites and effect duration from the shared projection", () => {
  const activity = {
    ...activityFixture("Effect"),
    effect_duration: knownFact({
      duration: knownFact("1 round"),
      sustained: knownFact(true),
      expiry: knownFact("At the end of the turn"),
    }),
  };
  render(
    <>
      <ContentFields
        fields={[
          {
            locator: {
              record: "feats:aaaaaaaaaaaaaaaa",
              embedded_path: [],
              field: "/system/prerequisites/value/0/value",
            },
            role: "prerequisite",
            source_fingerprint: null,
            body: {
              kind: "html",
              html: "trained in Medicine; Battle Medicine or Medic Dedication",
              controls: [],
            },
          },
        ]}
        onReference={() => {}}
      />
      <ActorActivity activity={activity} onReference={() => {}} />
    </>,
  );
  expect(screen.getByText("Prerequisites").parentElement).toHaveTextContent(
    "trained in Medicine; Battle Medicine or Medic Dedication",
  );
  expect(screen.getByText("Duration").parentElement).toHaveTextContent("1 round");
  expect(screen.getByText("Expires").parentElement).toHaveTextContent(
    "At the end of the turn",
  );
  expect(screen.getByText("Sustained").parentElement).toHaveTextContent("Yes");
});

it("distinguishes empty journals from unavailable collections and keeps selected page identity", () => {
  const base = {
    identity: summaryFixture("journals:aaaaaaaaaaaaaaaa"),
    classification: "Journal",
    content: [],
    embedded: [],
  };
  const { rerender } = render(
    <RecordPresentation
      presentation={{
        ...base,
        body: { kind: "journal", value: { pages: knownFact([]) } },
      }}
      onReference={() => {}}
    />,
  );
  expect(screen.getByText("No pages.")).toBeVisible();
  rerender(
    <RecordPresentation
      presentation={{
        ...base,
        body: { kind: "journal", value: { pages: unavailableFact("invalid") } },
      }}
      onReference={() => {}}
    />,
  );
  expect(screen.getByText(/Pages unavailable/)).toHaveTextContent("[invalid]");
  expect(screen.queryByText("No pages.")).toBeNull();
  rerender(
    <RecordPresentation
      presentation={{
        ...base,
        identity: { ...base.identity, title: "Page identity" },
        content: [
          {
            locator: {
              record: "journals:aaaaaaaaaaaaaaaa",
              embedded_path: [],
              field: "/text/content",
            },
            role: "description",
            source_fingerprint: null,
            body: { kind: "html", html: "<p>Page body</p>", controls: [] },
          },
        ],
        body: {
          kind: "journal_page",
          value: { show_body_heading: knownFact(false), media_notice: null },
        },
      }}
      onReference={() => {}}
    />,
  );
  expect(screen.getByRole("heading", { name: "Page identity" })).toBeVisible();
  expect(screen.getByText("Page body")).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Description" })).toBeNull();
});

it("keeps table roll guidance behind an accessible information button", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <TableRollControl
        selection={{
          record_key: "tables:test",
          embedded_path: [],
          fields: [],
          passage: null,
          source_fingerprint: null,
        }}
        artifactGeneration="test"
        formula="1d6"
        rollNote="Rolls may repeat."
        onResult={() => {}}
      />
    </ConfigProvider>,
  );
  expect(screen.queryByText("Rolls may repeat.")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "About table rolls" }));
  await waitFor(() => expect(screen.getByText("Rolls may repeat.")).toBeVisible());
});
