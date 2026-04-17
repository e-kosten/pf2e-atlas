import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { OntologyDomainModel } from "../../src/types.js";
import { OntologyBrowserScreen } from "../../src/tui/ontology-explorer/screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createTestOntologyModel(): OntologyDomainModel {
  return {
    id: "derivedTags",
    label: "Derived Tags",
    description: "Test ontology domain",
    rootNodes: [
      {
        id: "spell",
        kind: "category",
        label: "Spell",
        filterText: "spell",
        listLabel: "spell | 1 family",
        detailTitle: "Category Details",
        detailLines: [{ text: "Spell", tone: "section" }],
        children: [
          {
            id: "spell:security",
            kind: "family",
            label: "security",
            filterText: "security alarm",
            listLabel: "security | 1 tag",
            detailTitle: "Family Details",
            detailLines: [{ text: "security", tone: "section" }],
          },
        ],
      },
    ],
  };
}

describe("ontology browser screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("treats q as search input while inline search is active", async () => {
    const model = createTestOntologyModel();
    const onExit = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen model={model} onExit={onExit} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("/");
    await flushInk();
    expect(app.lastFrame()).toContain("Search /");

    app.stdin.write("q");
    await flushInk();

    expect(onExit).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("Search /q");

    app.unmount();
  });
});
