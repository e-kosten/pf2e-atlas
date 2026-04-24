import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { OntologyDomainModel } from "../../src/domain/ontology-types.js";
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

afterEach(() => {
  cleanup();
});

describe("OntologyInspectScreen", () => {
  it("keeps the explorer mounted while showing a pending route transition", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyInspectScreen
          routeData={{ model: createOntologyModel() }}
          onExit={vi.fn()}
          transitionStatus={{ kind: ROUTE_TRANSITION_STATUS_KIND.PENDING, message: "Opening browse/search..." }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    const frame = app.lastFrame();
    expect(frame).toContain("Search Semantics");
    expect(frame).toContain("Explorer Entries");
    expect(frame).toContain("Pathfinder Monster Core | 320");
    expect(frame).toContain("Loading next view | Opening browse/search...");
  });
});
