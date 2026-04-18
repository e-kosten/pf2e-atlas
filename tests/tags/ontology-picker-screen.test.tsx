import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { OntologyDomainModel } from "../../src/types.js";
import { OntologyPickerScreen } from "../../src/tui/ontology-explorer/picker-screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createPickerModel(): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Derived Tags",
    description: "Picker test domain",
    rootNodes: [
      {
        id: "spell:field:derivedTags",
        kind: "field",
        label: "derivedTags",
        filterText: "derived tags",
        listLabel: "derivedTags",
        detailTitle: "Metadata Field Details",
        detailLines: [{ text: "derivedTags", tone: "section" }],
        childPresentation: {
          mode: "grouped",
          groupBy: "axis",
          render: "inline",
        },
        children: [
          {
            id: "spell:derivedTags:coastal_setting",
            kind: "tag",
            label: "coastal_setting",
            filterText: "coastal setting",
            listLabel: "coastal_setting",
            detailTitle: "Tag Details",
            detailLines: [{ text: "coastal_setting", tone: "section" }],
            groupValues: {
              axis: "environment",
            },
            selection: {
              field: "derivedTags",
              fieldLabel: "Derived Tags",
              value: "coastal_setting",
              allowedStates: ["any", "all", "exclude"],
            },
            children: [
              {
                id: "record:1",
                kind: "record",
                label: "Harbor Haunt",
                filterText: "harbor haunt",
                listLabel: "Harbor Haunt",
                detailTitle: "Record Details",
                detailLines: [{ text: "Harbor Haunt", tone: "section" }],
              },
            ],
          },
        ],
      },
    ],
  };
}

describe("ontology picker screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("treats enter and space the same for selectable nodes even when they have children", async () => {
    const onApply = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyPickerScreen
          model={createPickerModel()}
          onApply={onApply}
          onCancel={vi.fn()}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Environment");
    expect(app.lastFrame()).toContain("Policy any");
    expect(app.lastFrame()).toContain("derivedTags: any=coastal_setting");

    app.stdin.write(" ");
    await flushInk();
    expect(app.lastFrame()).toContain("Policy all");
    expect(app.lastFrame()).toContain("derivedTags: all=coastal_setting");

    expect(onApply).not.toHaveBeenCalled();
    expect(app.lastFrame()).not.toContain("Harbor Haunt");
  });
});
