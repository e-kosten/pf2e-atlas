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
            id: "spell:family:coast",
            kind: "family",
            label: "coast",
            filterText: "coast coastal setting",
            listLabel: "coast | 1 tag",
            detailTitle: "Family Details",
            detailLines: [{ text: "coast", tone: "section" }],
            groupValues: {
              axis: "environment",
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
        <OntologyPickerScreen model={createPickerModel()} onApply={onApply} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("[TAGS]");
    expect(app.lastFrame()).toContain("Policy any");
    expect(app.lastFrame()).toContain("derivedTags: any=coastal_setting");

    app.stdin.write(" ");
    await flushInk();
    expect(app.lastFrame()).toContain("Policy all");
    expect(app.lastFrame()).toContain("derivedTags: all=coastal_setting");

    expect(onApply).not.toHaveBeenCalled();
    expect(app.lastFrame()).not.toContain("Harbor Haunt");
  });

  it("applies the latest selection when return follows a toggle immediately", async () => {
    const onApply = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyPickerScreen model={createPickerModel()} onApply={onApply} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("q");
    await flushInk();

    expect(onApply).toHaveBeenCalledWith({
      derivedTags: {
        any: ["coastal_setting"],
        all: [],
        exclude: [],
      },
    });
  });

  it("uses right-arrow to drill into selectable nodes with children instead of cycling them", async () => {
    const onApply = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyPickerScreen model={createPickerModel()} onApply={onApply} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    app.stdin.write("\u001b[C");
    await flushInk();

    expect(app.lastFrame()).toContain("Harbor Haunt");
    expect(app.lastFrame()).toContain("Policy off");
    expect(app.lastFrame()).toContain("derivedTags: any=coastal_setting");
    expect(app.lastFrame()).not.toContain("Policy all");
    expect(onApply).not.toHaveBeenCalled();
  });

  it("consumes space on non-selectable nodes instead of paging the list", async () => {
    const onApply = vi.fn();
    const model: OntologyDomainModel = {
      ...createPickerModel(),
      rootNodes: [
        createPickerModel().rootNodes[0]!,
        {
          id: "creature:field:languages",
          kind: "field",
          label: "languages",
          filterText: "languages",
          listLabel: "languages",
          detailTitle: "Metadata Field Details",
          detailLines: [{ text: "languages", tone: "section" }],
          children: [],
        },
      ],
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyPickerScreen model={model} onApply={onApply} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("derivedTags");
    expect(app.lastFrame()).toContain("languages");
    expect(app.lastFrame()).toContain("Focused: derivedTags");

    app.stdin.write(" ");
    await flushInk();

    expect(app.lastFrame()).toContain("derivedTags");
    expect(app.lastFrame()).toContain("languages");
    expect(app.lastFrame()).toContain("Focused: derivedTags");
    expect(onApply).not.toHaveBeenCalled();
  });

  it("returns current selections when q exits the picker", async () => {
    const onApply = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyPickerScreen model={createPickerModel()} onApply={onApply} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("q");
    await flushInk();

    expect(onApply).toHaveBeenCalledWith({
      derivedTags: {
        any: ["coastal_setting"],
        all: [],
        exclude: [],
      },
    });
  });

  it("returns current selections when left exits from the root", async () => {
    const onApply = vi.fn();
    const rootSelectableModel: OntologyDomainModel = {
      id: "searchSemantics",
      label: "Traits",
      description: "Picker test domain",
      rootNodes: [
        {
          id: "spell:trait:fire",
          kind: "trait",
          label: "fire",
          filterText: "fire",
          listLabel: "fire",
          detailTitle: "Trait Details",
          detailLines: [{ text: "fire", tone: "section" }],
          selection: {
            field: "traits",
            fieldLabel: "Traits",
            value: "fire",
            allowedStates: ["any", "all", "exclude"],
          },
        },
      ],
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyPickerScreen model={rootSelectableModel} onApply={onApply} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    app.stdin.write("\u001b[D");
    await flushInk();

    expect(onApply).toHaveBeenCalledWith({
      traits: {
        any: ["fire"],
        all: [],
        exclude: [],
      },
    });
  });

  it("loads lazy picker children without mutating the shared model", async () => {
    const onApply = vi.fn();
    const model: OntologyDomainModel = {
      id: "searchSemantics",
      label: "Traits",
      description: "Lazy picker domain",
      rootNodes: [
        {
          id: "spell:field:traits",
          kind: "field",
          label: "traits",
          filterText: "traits",
          listLabel: "traits",
          detailTitle: "Trait Details",
          detailLines: [{ text: "traits", tone: "section" }],
          loadChildren: () => [
            {
              id: "spell:trait:fire",
              kind: "trait",
              label: "fire",
              filterText: "fire",
              listLabel: "fire",
              detailTitle: "Trait Details",
              detailLines: [{ text: "fire", tone: "section" }],
              selection: {
                field: "traits",
                fieldLabel: "Traits",
                value: "fire",
                allowedStates: ["any", "all", "exclude"],
              },
            },
          ],
        },
      ],
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyPickerScreen model={model} onApply={onApply} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    app.stdin.write("q");
    await flushInk();

    expect(onApply).toHaveBeenCalledWith({
      traits: {
        any: ["fire"],
        all: [],
        exclude: [],
      },
    });
    expect(model.rootNodes[0]?.children).toBeUndefined();
  });
});
