import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { EntityPageDocument } from "../../src/app/ontology/entity-page.js";
import type { OntologyDomainModel } from "../../src/domain/ontology-types.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import { OntologyInspectScreen } from "../../src/tui/ontology-explorer/inspect-screen.js";
import { ROUTE_TRANSITION_STATUS_KIND } from "../../src/tui/route-transition-status.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import { browseQuery, scopeFilter } from "../helpers/search-request-fixture.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createOntologyModel(): OntologyDomainModel {
  return {
    id: "search-semantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [
      {
        id: "creature:publicationTitle:monster-core",
        kind: "value",
        label: "Pathfinder Monster Core",
        filterText: "pathfinder monster core",
        listLabel: "Pathfinder Monster Core | 320",
        detailTitle: "Filter Value",
        detailLines: [{ text: "Pathfinder Monster Core", tone: "section" }],
        query: browseQuery("Browse records with this value", {
          filter: scopeFilter("creature"),
          limit: 20,
        }),
      },
    ],
  };
}

function createMetricOntologyModel(): OntologyDomainModel {
  return {
    id: "search-semantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [
      {
        id: "creature:actorMetrics:hp.value",
        kind: "metric",
        label: "Hit Points",
        filterText: "hit points",
        listLabel: "hp.",
        detailTitle: "Metric",
        detailLines: [{ text: "Hit Points", tone: "section" }],
        query: browseQuery("Browse creatures by hit points", {
          filter: scopeFilter("creature"),
          limit: 20,
        }),
      },
    ],
  };
}

function createRecordOntologyModel(): OntologyDomainModel {
  return {
    id: "search-semantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [
      {
        id: "spell:test-fireball",
        kind: "record",
        label: "Fireball",
        filterText: "fireball",
        listLabel: "Fireball",
        detailTitle: "Record",
        detailLines: [{ text: "Record detail fallback" }],
        query: browseQuery("Browse fireball references", {
          filter: scopeFilter("spell"),
          limit: 20,
        }),
      },
    ],
  };
}

function createRecordPageDocument(): EntityPageDocument {
  const searchPivot = {
    kind: "searchPivot" as const,
    label: "Creatures (2)",
    request: browseQuery("Browse creatures that reference Fireball", {
      filter: scopeFilter("creature"),
      limit: 20,
    }).request,
  };

  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    traits: ["Fire"],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused page summary." }],
        targets: [],
      },
      {
        id: "references",
        kind: "references",
        title: "References",
        blocks: [
          {
            kind: "targetList",
            targets: [searchPivot],
          },
        ],
        targets: [searchPivot],
      },
    ],
  };
}

function createTargetFirstRecordPageDocument(): EntityPageDocument {
  const document = createRecordPageDocument();
  return {
    ...document,
    sections: [
      {
        ...document.sections[1]!,
        blocks: [
          { kind: "text", text: "Target intro 1" },
          { kind: "text", text: "Target intro 2" },
          { kind: "text", text: "Target intro 3" },
          { kind: "text", text: "Target intro 4" },
          ...document.sections[1]!.blocks,
        ],
      },
      document.sections[0]!,
    ],
  };
}

function createServices(document: EntityPageDocument | null = null) {
  return {
    config: {} as never,
    user: {
      entityPages: {
        buildDocument: vi.fn(),
        buildDocumentByRecordKey: vi.fn((recordKey: string) =>
          document && recordKey === document.recordKey ? document : null,
        ),
        buildDetailLines: vi.fn(),
      },
      ontology: {} as never,
      pageRelations: {} as never,
      search: {} as never,
    },
    dev: {
      tagRefinement: {} as never,
    },
    close: vi.fn(),
  } as const;
}

function renderInspectScreen(
  element: React.JSX.Element,
  services = createServices(),
) {
  return render(
    <DerivedTagTerminalProvider>
      <Pf2eTerminalAppServicesProvider services={services as never}>{element}</Pf2eTerminalAppServicesProvider>
    </DerivedTagTerminalProvider>,
  );
}

afterEach(() => {
  cleanup();
});

describe("OntologyInspectScreen", () => {
  it("keeps the explorer mounted while showing a pending route transition", async () => {
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createOntologyModel() }}
        onExit={vi.fn()}
        transitionStatus={{ kind: ROUTE_TRANSITION_STATUS_KIND.PENDING, message: "Opening browse/search..." }}
      />,
    );

    await flushInk();
    await flushInk();

    const frame = app.lastFrame();
    expect(frame).toContain("Search Semantics");
    expect(frame).toContain("Explorer Entries");
    expect(frame).toContain("Pathfinder Monster Core | 320");
    expect(frame).toContain("Loading next view | Opening browse/search...");
  });

  it("opens the shared numeric scalar editor for metric nodes through the inspect host adapter", async () => {
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createMetricOntologyModel() }}
        onExit={vi.fn()}
      />,
    );

    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("hp.");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Creature Statistics / Hit Points");
    expect(app.lastFrame()).toContain("Enter `5`, `!=5`, `>5`, `>=5`, `<5`, `<=5`, or `3-8`.");
  });

  it("renders record nodes through the shared entity page pipeline", async () => {
    const services = createServices(createRecordPageDocument());
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createRecordOntologyModel() }}
        onExit={vi.fn()}
      />,
      services,
    );

    await flushInk();
    await flushInk();

    const frame = app.lastFrame();
    expect(frame).toContain("Fireball");
    expect(frame).toContain("Summary");
    expect(frame).toContain("A focused page summary.");
    expect(frame).toContain("References");
    expect(frame).toContain("Creatures (2)");
    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-fireball");
  });

});
