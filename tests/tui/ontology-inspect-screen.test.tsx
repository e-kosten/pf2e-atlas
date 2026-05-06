import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { EntityPageDocument } from "../../src/app/ontology/entity-page.js";
import type { OntologyDomainModel } from "../../src/domain/ontology-types.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import { createNoopTerminalDebugTraceService } from "../../src/tui/debug-trace.js";
import { OntologyInspectScreen } from "../../src/tui/ontology-explorer/inspect-screen.js";
import { ROUTE_TRANSITION_STATUS_KIND } from "../../src/tui/route-transition-status.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import { browseQuery, scopeFilter } from "../helpers/search-request-fixture.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function pressDown(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[B");
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

function createClassificationRecordPageDocument(): EntityPageDocument {
  const categoryTarget = {
    kind: "searchPivot" as const,
    label: "Category: Spell",
    request: browseQuery("Browse spell records", {
      filter: scopeFilter("spell"),
      limit: 50,
    }).request,
  };

  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    traits: [],
    sections: [
      {
        id: "classification",
        kind: "classification",
        title: "Classification",
        blocks: [{ kind: "targetList", targets: [categoryTarget] }],
        targets: [categoryTarget],
      },
    ],
  };
}

function createAonRecordPageDocument(): EntityPageDocument {
  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    aonLink: {
      kind: "external",
      label: "Open in Archives of Nethys",
      href: "https://2e.aonprd.com/Spells.aspx?ID=1530",
      plainTextFallback: "Open in Archives of Nethys",
    },
    traits: [],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused page summary." }],
        targets: [],
      },
    ],
  };
}

function createPreviewTargetRecordPageDocument(): EntityPageDocument {
  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    traits: ["Fire"],
    sections: [
      {
        id: "references",
        kind: "references",
        title: "References",
        blocks: [
          { kind: "text", text: "Target intro 1" },
          { kind: "text", text: "Target intro 2" },
          { kind: "text", text: "Target intro 3" },
          { kind: "text", text: "Target intro 4" },
          {
            kind: "targetList",
            targets: [
              {
                kind: "record",
                label: "Chain Lightning",
                recordKey: "spell:test-chain-lightning",
                action: "preview",
              },
            ],
          },
        ],
        targets: [
          {
            kind: "record",
            label: "Chain Lightning",
            recordKey: "spell:test-chain-lightning",
            action: "preview",
          },
        ],
      },
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused page summary." }],
        targets: [],
      },
    ],
  };
}

function createScrollableTargetRecordPageDocument(): EntityPageDocument {
  const references = Array.from({ length: 18 }, (_, index) => ({
    kind: "record" as const,
    label: `Reference ${String(index + 1).padStart(2, "0")}`,
    recordKey: `spell:reference-${index + 1}`,
    action: "preview" as const,
  }));

  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    traits: [],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused page summary." }],
        targets: [],
      },
      {
        id: "details",
        kind: "details",
        title: "Details",
        blocks: [
          {
            kind: "factList",
            facts: [
              { label: "Traditions", value: "Arcane" },
              { label: "Cast", value: "2 actions" },
              { label: "Range", value: "500 feet" },
              { label: "Area", value: "20-foot burst" },
            ],
          },
        ],
        targets: [],
      },
      {
        id: "references",
        kind: "references",
        title: "References",
        blocks: [{ kind: "targetList", targets: references }],
        targets: references,
      },
    ],
  };
}

function createPreviewTargetDocument(): EntityPageDocument {
  return {
    recordKey: "spell:test-chain-lightning",
    title: "Chain Lightning",
    identityLine: "Spell | Rank 6 | Common | Pathfinder Player Core",
    traits: ["Electricity"],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "Preview-only target summary." }],
        targets: [],
      },
    ],
  };
}

function createInlineTraitRecordPageDocument(): EntityPageDocument {
  const fireTraitTarget = {
    kind: "searchPivot" as const,
    label: "Trait: Fire",
    request: browseQuery("Browse fire spells", {
      filter: scopeFilter("spell"),
      limit: 20,
    }).request,
  };

  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    traits: ["Fire"],
    traitTargets: [fireTraitTarget],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused page summary." }],
        targets: [],
      },
    ],
  };
}

function createInlineUuidRecordPageDocument(): EntityPageDocument {
  const target = {
    kind: "record" as const,
    label: "Chain Lightning",
    recordKey: "spell:test-chain-lightning" as const,
    action: "preview" as const,
  };

  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    traits: ["Fire"],
    sections: [
      {
        id: "description",
        kind: "description",
        title: "Description",
        blocks: [
          {
            kind: "text",
            text: "Reference Chain Lightning in the storm.",
            segments: [
              { text: "Reference " },
              { text: "Chain Lightning", target },
              { text: " in the storm." },
            ],
          },
        ],
        targets: [],
      },
    ],
  };
}

function createServices(documents: EntityPageDocument | readonly EntityPageDocument[] | null = null) {
  const documentList = documents == null ? [] : Array.isArray(documents) ? [...documents] : [documents];
  const documentsByRecordKey = new Map(documentList.map((document) => [document.recordKey, document]));
  return {
    config: {} as never,
    debug: createNoopTerminalDebugTraceService(),
    user: {
      entityPages: {
        buildDocument: vi.fn(),
        buildDocumentByRecordKey: vi.fn((recordKey: string) =>
          documentsByRecordKey.get(recordKey) ?? null,
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
    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-fireball", {
      recordTargetAction: "preview",
    });
  });

  it("previews record page targets in place through the ontology inspect host", async () => {
    const sourceDocument = createPreviewTargetRecordPageDocument();
    const targetDocument = createPreviewTargetDocument();
    const services = createServices([sourceDocument, targetDocument]);
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createRecordOntologyModel() }}
        onExit={vi.fn()}
      />,
      services,
    );

    await flushInk();
    await flushInk();

    app.stdin.write("\t");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    const frame = app.lastFrame();
    expect(frame).toContain("Chain Lightning");
    expect(frame).toContain("Preview-only target summary.");
    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-fireball", {
      recordTargetAction: "preview",
    });
    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-chain-lightning", {
      recordTargetAction: "preview",
    });
  });

  it("keeps offscreen selected targets visible in the ontology inspect host", async () => {
    const services = createServices(createScrollableTargetRecordPageDocument());
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createRecordOntologyModel() }}
        onExit={vi.fn()}
      />,
      services,
    );

    await flushInk();
    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Fireball | References");
    expect(app.lastFrame()).toContain("Reference 01");
    expect(app.lastFrame()).not.toContain("Reference 18");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("↑/↓ target");

    app.stdin.write("G");
    await flushInk();

    expect(app.lastFrame()).toContain("↑/↓ target");
    expect(app.lastFrame()).toContain("Reference 18");
  });

  it("activates inline trait targets through the ontology inspect host", async () => {
    const onActivatePageTarget = vi.fn(() => true);
    const sourceDocument = createInlineTraitRecordPageDocument();
    const services = createServices(sourceDocument);
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createRecordOntologyModel() }}
        onActivatePageTarget={onActivatePageTarget}
        onExit={vi.fn()}
      />,
      services,
    );

    await flushInk();
    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("Traits: Fire");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onActivatePageTarget).toHaveBeenCalledWith(sourceDocument.traitTargets?.[0]);
  });

  it("activates Classification row pivots through the ontology inspect host", async () => {
    const onActivatePageTarget = vi.fn(() => true);
    const sourceDocument = createClassificationRecordPageDocument();
    const services = createServices(sourceDocument);
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createRecordOntologyModel() }}
        onActivatePageTarget={onActivatePageTarget}
        onExit={vi.fn()}
      />,
      services,
    );

    await flushInk();
    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("Classification");
    expect(app.lastFrame()).toContain("Category: Spell");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onActivatePageTarget).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "searchPivot",
        label: "Category: Spell",
      }),
    );
  });

  it("activates external Archives of Nethys targets through the ontology inspect host", async () => {
    const onActivatePageTarget = vi.fn(() => true);
    const sourceDocument = createAonRecordPageDocument();
    const services = createServices(sourceDocument);
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createRecordOntologyModel() }}
        onActivatePageTarget={onActivatePageTarget}
        onExit={vi.fn()}
      />,
      services,
    );

    await flushInk();
    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("Open in Archives of Nethys");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onActivatePageTarget).toHaveBeenCalledWith(sourceDocument.aonLink);
  });

  it("previews inline UUID record targets through the ontology inspect host", async () => {
    const sourceDocument = createInlineUuidRecordPageDocument();
    const targetDocument = createPreviewTargetDocument();
    const services = createServices([sourceDocument, targetDocument]);
    const app = renderInspectScreen(
      <OntologyInspectScreen
        routeData={{ model: createRecordOntologyModel() }}
        onExit={vi.fn()}
      />,
      services,
    );

    await flushInk();
    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("Reference Chain Lightning in the storm.");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Chain Lightning");
    expect(app.lastFrame()).toContain("Preview-only target summary.");
    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-chain-lightning", {
      recordTargetAction: "preview",
    });
  });

});
