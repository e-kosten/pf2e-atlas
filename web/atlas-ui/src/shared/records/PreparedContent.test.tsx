import { fireEvent, render, screen } from "@testing-library/react";
import { PreparedContent } from "./PreparedContent";
import { destinationHref } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { detailFixture } from "../../test/fixtures";
import { ConfigProvider, theme } from "antd";
import * as api from "../../api/atlasApi";

it.each(["click", "Enter", " "])(
  "keeps missing-context damage as prose on %s",
  async (interaction) => {
    const content = detailFixture("actors:source", "Source", "rules:other").presentation
      .content[0];
    content.body = {
      kind: "html",
      inclusions: [],
      html: '<span data-atlas-interaction="0">Contextual damage</span>',
      controls: [
        {
          ordinal: 0,
          control: {
            kind: "damage",
            formula: "(@actor.level)d6[fire]",
            options: { traits: "fire" },
            literal: null,
            roll: null,
          },
        },
      ],
    };
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <PreparedContent content={content} onReference={vi.fn()} />
      </ConfigProvider>,
    );
    const text = screen.getByText("Contextual damage");
    expect(text).not.toHaveAttribute("role");
    expect(text).not.toHaveAttribute("tabindex");
    if (interaction === "click") fireEvent.click(text);
    else fireEvent.keyDown(text, { key: interaction });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.queryByRole("button")).toBeNull();
  },
);

describe("prepared source HTML", () => {
  it("renders compact effects with native destinations and refreshes without duplicate controls", () => {
    const detail = detailFixture("spells:source", "Source", "effects:target");
    const field = detail.presentation.content[0];
    field.body = {
      kind: "html",
      inclusions: [],
      html: '<p>Success <span data-atlas-reference="0">Spell Effect: Success</span></p>',
      controls: [],
    };
    const edge = {
      ...detail.relationships[0],
      effect_label: { text: "Effect", title: "Spell Effect: Success" },
    };
    const onReference = vi.fn();
    const { rerender } = render(
      <PreparedContent
        content={field}
        relationships={[edge]}
        onReference={onReference}
      />,
    );
    const link = screen.getByRole("link", { name: "Preview Spell Effect: Success" });
    expect(link).toHaveTextContent("Effect");
    expect(link).toHaveAttribute(
      "href",
      destinationHref(
        recordDestination({
          recordKey: edge.target!.record_key,
          selection: edge.target!,
        }),
      ),
    );
    fireEvent.click(link, { ctrlKey: true });
    expect(onReference).not.toHaveBeenCalled();
    fireEvent.click(link);
    expect(onReference).toHaveBeenCalledWith(
      edge.target!.record_key,
      link,
      edge.target,
    );
    rerender(
      <PreparedContent
        content={field}
        relationships={[{ ...edge }]}
        onReference={onReference}
      />,
    );
    expect(
      screen.getAllByRole("link", { name: "Preview Spell Effect: Success" }),
    ).toHaveLength(1);
  });
  it("retains semantic inline glyphs and readable text without inventing controls", () => {
    const content = detailFixture("actors:ghoul", "Ghoul", "rules:outer").presentation
      .content[0];
    content.body = {
      kind: "html",
      inclusions: [],
      html: '<p>Spend <span data-atlas-action-glyph="two_actions">two actions</span> or a <span data-atlas-action-glyph="reaction">reaction</span>.</p>',
      controls: [],
    };
    const callback = vi.fn();
    const { container } = render(
      <PreparedContent content={content} onReference={callback} />,
    );
    expect(screen.getByText("two actions")).toHaveAttribute(
      "data-atlas-action-glyph",
      "two_actions",
    );
    expect(screen.getByText("reaction")).toHaveAttribute(
      "data-atlas-action-glyph",
      "reaction",
    );
    expect(container.textContent).toBe("Spend two actions or a reaction.");
    fireEvent.click(screen.getByText("two actions"));
    expect(callback).not.toHaveBeenCalled();
    expect(screen.queryByRole("button")).toBeNull();
  });
  it("handles nested internal references once for click, Enter and Space", () => {
    const detail = detailFixture("actors:ghoul", "Ghoul", "rules:outer");
    const content = detail.presentation.content[0];
    content.body = {
      kind: "html",
      inclusions: [],
      html: '<span data-atlas-reference="0">Outer <span data-atlas-reference="1">Inner</span></span>',
      controls: [],
    };
    const inner = {
      ...detail.relationships[0],
      ordinal: 1,
      target: { ...detail.relationships[0].target!, record_key: "rules:inner" },
    };
    const callback = vi.fn();
    render(
      <PreparedContent
        content={content}
        relationships={[detail.relationships[0], inner]}
        onReference={callback}
      />,
    );
    const marker = screen.getByText("Inner");
    fireEvent.click(marker);
    fireEvent.keyDown(marker, { key: "Enter" });
    fireEvent.keyDown(marker, { key: " " });
    expect(callback).toHaveBeenCalledTimes(3);
    for (const call of callback.mock.calls) expect(call[0]).toBe("rules:inner");
  });
  it("keeps checks as text within references and leaves native child links uncancelled", () => {
    const detail = detailFixture("actors:ghoul", "Ghoul", "rules:outer");
    const content = detail.presentation.content[0];
    content.body = {
      kind: "html",
      inclusions: [],
      html: '<span data-atlas-reference="0">Outer <span data-atlas-interaction="0">Reflex</span><a data-atlas-reference="1" href="https://example.com">External</a><a href="https://example.net">Native</a></span>',
      controls: [
        {
          ordinal: 0,
          control: { kind: "check", statistic: "reflex", options: { dc: "20" } },
        },
      ],
    };
    const external = {
      ...detail.relationships[0],
      ordinal: 1,
      target: null,
      url: "https://example.com",
      status: "resolved_url",
    };
    const callback = vi.fn();
    render(
      <PreparedContent
        content={content}
        relationships={[detail.relationships[0], external]}
        onReference={callback}
      />,
    );
    const marker = screen.getByText("Reflex");
    fireEvent.click(marker);
    fireEvent.keyDown(marker, { key: "Enter" });
    fireEvent.keyDown(marker, { key: " " });
    expect(screen.queryByRole("button", { name: "Reflex" })).toBeNull();
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(callback).toHaveBeenCalledTimes(3);
    for (const name of ["External", "Native"]) {
      const link = screen.getByRole("link", { name });
      const click = new MouseEvent("click", { bubbles: true, cancelable: true });
      link.addEventListener("click", () => {
        expect(click.defaultPrevented).toBe(false);
      });
      // Avoid jsdom navigation while observing event ownership at the document boundary.
      const preventNavigation = (event: Event) => {
        expect(event.defaultPrevented).toBe(false);
        event.preventDefault();
      };
      document.addEventListener("click", preventNavigation, { once: true });
      link.dispatchEvent(click);
      const key = new KeyboardEvent("keydown", {
        key: "Enter",
        bubbles: true,
        cancelable: true,
      });
      link.dispatchEvent(key);
      expect(key.defaultPrevented).toBe(false);
    }
    expect(callback).toHaveBeenCalledTimes(3);
  });
  it("keeps unresolved prose noninteractive in dark mode", async () => {
    const content = detailFixture("actors:ghoul", "Ghoul", "rules:outer").presentation
      .content[0];
    content.body = {
      kind: "html",
      inclusions: [],
      html: '<span data-atlas-interaction="0">Contextual damage</span>',
      controls: [
        {
          ordinal: 0,
          control: {
            kind: "damage",
            formula: "(@actor.level)d6[fire]",
            options: { traits: "fire" },
            literal: null,
            roll: null,
          },
        },
      ],
    };
    render(
      <ConfigProvider prefixCls="themed" theme={{ algorithm: theme.darkAlgorithm }}>
        <PreparedContent content={content} onReference={vi.fn()} />
      </ConfigProvider>,
    );
    const text = screen.getByText("Contextual damage");
    fireEvent.keyDown(text, { key: "Enter" });
    expect(text).not.toHaveAttribute("tabindex");
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  it("renders HTML semantics and references from field-local facts", () => {
    const detail = detailFixture("actors:ghoul", "Ghoul", "rules:nested");
    const content = detail.presentation.content[0];
    const callback = vi.fn();
    render(
      <PreparedContent
        content={content}
        relationships={detail.relationships}
        onReference={callback}
      />,
    );
    fireEvent.click(screen.getByText("Nested Rule"));
    expect(callback).toHaveBeenCalledWith(
      "rules:nested",
      expect.anything(),
      detail.relationships[0].target,
    );
    const reference = screen.getByText("Nested Rule");
    expect(reference).toHaveAttribute(
      "href",
      destinationHref(
        recordDestination({
          recordKey: "rules:nested",
          selection: detail.relationships[0].target!,
        }),
      ),
    );
    expect(reference).not.toHaveAttribute("role", "button");
    expect(reference).toHaveAttribute("tabindex", "0");
    fireEvent.keyDown(reference, { key: "Enter" });
    fireEvent.keyDown(reference, { key: " " });
    expect(callback).toHaveBeenCalledTimes(3);
  });
  it("strips dangerous markup while unknown commands remain inert", () => {
    const content = detailFixture("actors:ghoul", "Ghoul", "rules:nested").presentation
      .content[0];
    content.body = {
      kind: "html",
      inclusions: [],
      html: '<h2>Heading</h2><script>alert(1)</script><a href="javascript:alert(1)">Unsafe</a><span data-atlas-interaction="0">Unknown command</span>',
      controls: [
        {
          ordinal: 0,
          control: {
            kind: "command",
            command: "Mystery",
            arguments: "",
            options: {},
            literal: null,
            roll: null,
          },
        },
      ],
    };
    const { container } = render(
      <PreparedContent content={content} onReference={vi.fn()} />,
    );
    expect(screen.getByRole("heading", { name: "Heading" })).toBeInTheDocument();
    expect(container.querySelector("script")).toBeNull();
    expect(screen.getByText("Unsafe")).not.toHaveAttribute("href");
    fireEvent.click(screen.getByText("Unknown command"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  it("keeps validated external links native while blocked references stay inert", () => {
    const detail = detailFixture("actors:ghoul", "Ghoul", "rules:nested");
    const content = detail.presentation.content[0];
    content.body = {
      kind: "html",
      inclusions: [],
      html: '<p><a data-atlas-reference="0" href="https://example.com">External source</a><a data-atlas-reference="1" href="https://example.com/blocked">Blocked</a></p>',
      controls: [],
    };
    const safe = {
      ...detail.relationships[0],
      target: null,
      url: "https://example.com",
      status: "resolved_url",
    };
    const blocked = { ...safe, ordinal: 1, status: "blocked", url: null };
    render(
      <PreparedContent
        content={content}
        relationships={[safe, blocked]}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByRole("link", { name: "External source" })).toHaveAttribute(
      "href",
      "https://example.com",
    );
    expect(screen.getByText("External source")).not.toHaveAttribute("role", "button");
    expect(screen.getByText("Blocked")).not.toHaveAttribute("href");
  });
});

it("retains exact native local addresses, modifiers and target intent through listener replacement and unavailable refresh", () => {
  const detail = detailFixture("actors:parent", "Parent", "rules:linked");
  const target = {
    ...detail.relationships[0].target!,
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 3 } } }],
    field: "system.description.value",
    source_fingerprint: "copy-a",
  };
  detail.relationships[0].target = target;
  const callback = vi.fn();
  const nextCallback = vi.fn();
  const props = {
    content: detail.presentation.content[0],
    relationships: detail.relationships,
    onReference: callback,
  };
  const { rerender } = render(<PreparedContent {...props} />);
  const link = screen.getByRole("link", { name: "Nested Rule" });
  expect(link).toHaveAttribute(
    "href",
    destinationHref(
      recordDestination({ recordKey: target.record_key, selection: target }),
    ),
  );
  for (const modifiers of [
    { metaKey: true },
    { ctrlKey: true },
    { shiftKey: true },
    { altKey: true },
    { button: 1 },
  ])
    expect(fireEvent.click(link, modifiers)).toBe(true);
  expect(fireEvent.keyDown(link, { key: "Enter", metaKey: true })).toBe(true);
  expect(callback).not.toHaveBeenCalled();
  link.setAttribute("target", "_blank");
  expect(fireEvent.click(link)).toBe(true);
  expect(callback).not.toHaveBeenCalled();
  link.removeAttribute("target");
  rerender(<PreparedContent {...props} onReference={nextCallback} />);
  fireEvent.click(screen.getByRole("link", { name: "Nested Rule" }));
  expect(nextCallback).toHaveBeenCalledOnce();
  expect(callback).not.toHaveBeenCalled();
  rerender(
    <PreparedContent
      {...props}
      relationships={[{ ...detail.relationships[0], target: null }]}
      onReference={nextCallback}
    />,
  );
  const unavailable = screen.getByText("Nested Rule");
  expect(unavailable).toHaveAttribute("aria-disabled", "true");
  expect(unavailable).not.toHaveAttribute("href");
  expect(unavailable).not.toHaveAttribute("tabindex");
  fireEvent.click(unavailable);
  expect(nextCallback).toHaveBeenCalledOnce();
});

it("opens verified item context without changing the original citation", () => {
  const detail = detailFixture("equipment:wand", "Wand", "spells:heal");
  const field = detail.presentation.content[0];
  field.body = {
    kind: "html",
    inclusions: [],
    html: '<p>Cast <span data-atlas-reference="0">Heal</span>.</p>',
    controls: [],
  };
  const edge = {
    ...detail.relationships[0],
    contextual_target: {
      ...detail.relationships[0].target!,
      record_key: "equipment:wand",
      embedded_path: [
        { collection: "/system/spell", identity: { Stable: "localspell" } },
      ],
      spell_rank: 2,
    },
  };
  const original = structuredClone(edge.target);
  const onReference = vi.fn();
  render(
    <PreparedContent
      content={field}
      relationships={[edge]}
      onReference={onReference}
    />,
  );
  const link = screen.getByRole("link", { name: "Heal" });
  expect(link).toHaveAttribute(
    "href",
    destinationHref(
      recordDestination({
        recordKey: "equipment:wand",
        selection: edge.contextual_target,
      }),
    ),
  );
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith(
    "equipment:wand",
    link,
    edge.contextual_target,
  );
  expect(edge.target).toEqual(original);
});

it("keeps authored table nodes, links and horizontal position across unchanged-content updates", () => {
  const detail = detailFixture("spells:ritual", "Ritual", "rules:other");
  const content = detail.presentation.content[0];
  content.body = {
    kind: "html",
    inclusions: [],
    html: '<p>Before</p><table><caption>Creature creation</caption><thead><tr><th>Level</th><th>Cost</th></tr></thead><tbody><tr><td>2</td><td><span data-atlas-reference="0">Nested Rule</span></td></tr></tbody></table><p>After</p>',
    controls: [],
  };
  const onReference = vi.fn();
  const { rerender } = render(
    <PreparedContent
      content={content}
      relationships={detail.relationships}
      onReference={onReference}
    />,
  );
  const table = screen.getByRole("table", { name: "Creature creation" });
  const region = table.parentElement!;
  region.scrollLeft = 72;
  const link = screen.getByRole("link", { name: "Nested Rule" });
  link.focus();
  rerender(
    <PreparedContent
      content={{ ...content, body: { ...content.body } }}
      relationships={[...detail.relationships]}
      onReference={onReference}
    />,
  );
  expect(screen.getByRole("table", { name: "Creature creation" })).toBe(table);
  expect(table.parentElement).toBe(region);
  expect(region.scrollLeft).toBe(72);
  expect(link).toHaveFocus();
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledOnce();
  expect(screen.getByText("Before")).toBeVisible();
  expect(screen.getByText("After")).toBeVisible();
  rerender(
    <PreparedContent
      content={{
        ...content,
        body: {
          kind: "html",
          inclusions: [],
          html: "<p>Replacement record</p>",
          controls: [],
        },
      }}
      onReference={onReference}
    />,
  );
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByText("Replacement record")).toBeVisible();
});

it("binds inclusion links to their own exact targets while checks remain plain text", () => {
  const detail = detailFixture("spells:host", "Host", "actions:host-link");
  const field = detail.presentation.content[0];
  const nested = {
    ...detail.relationships[0],
    target: { ...detail.relationships[0].target!, record_key: "actions:nested" },
  };
  field.body = {
    kind: "html",
    html: '<p><span data-atlas-reference="0">Host link</span></p><section data-atlas-inclusion="1"><p><span data-atlas-reference="0">Nested link</span> <span data-atlas-interaction="0">Nested check</span></p></section>',
    controls: [
      {
        ordinal: 0,
        control: { kind: "check", statistic: "Host check", options: { dc: "10" } },
      },
    ],
    inclusions: [
      {
        namespace: 1,
        source: { ...field.locator, record: "actions:nested" },
        references: [nested],
        controls: [
          {
            ordinal: 0,
            control: {
              kind: "check",
              statistic: "Nested check",
              options: { dc: "22" },
            },
          },
        ],
      },
    ],
  };
  const onReference = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <PreparedContent
        content={field}
        relationships={detail.relationships}
        onReference={onReference}
      />
    </ConfigProvider>,
  );
  const link = screen.getByRole("link", { name: "Nested link" });
  expect(link.getAttribute("href")).toContain("actions:nested");
  fireEvent.click(link, { ctrlKey: true });
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(onReference.mock.calls[0][0]).toBe("actions:nested");
  fireEvent.click(screen.getByText("Nested check"));
  expect(screen.queryByRole("button", { name: "Nested check" })).toBeNull();
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(onReference).toHaveBeenCalledTimes(1);
});

it.each([false, true])(
  "preserves ranked effect context for %s compact rendering and native navigation",
  (compact) => {
    const detail = detailFixture("spells:form", "Form", "effects:bear");
    const field = detail.presentation.content[0];
    field.body = {
      kind: "html",
      inclusions: [],
      html: '<p><span data-atlas-reference="0">Bear</span></p>',
      controls: [],
    };
    const original = structuredClone(detail.relationships[0].target!);
    const contextual = { ...original, spell_rank: 4 };
    const edge = {
      ...detail.relationships[0],
      contextual_target: contextual,
      effect_label: compact ? { text: "Bear", title: "Animal Form (Bear)" } : null,
    };
    const onReference = vi.fn();
    render(
      <PreparedContent
        content={field}
        relationships={[edge]}
        onReference={onReference}
      />,
    );
    const link = screen.getByRole("link", {
      name: compact ? "Preview Animal Form (Bear)" : "Bear",
    });
    expect(link).toHaveAttribute(
      "href",
      destinationHref(
        recordDestination({ recordKey: contextual.record_key, selection: contextual }),
      ),
    );
    expect(link).not.toHaveAttribute("title", "Spell from this item");
    fireEvent.click(link, { ctrlKey: true });
    expect(onReference).not.toHaveBeenCalled();
    fireEvent.click(link);
    expect(onReference).toHaveBeenCalledWith(contextual.record_key, link, contextual);
    expect(edge.target).toEqual(original);
  },
);

it.each(["source", "participant"] as const)(
  "keeps fixed damage plain and resets its result when the %s binding changes",
  async (binding) => {
    const roll = vi.spyOn(api, "rollContent").mockResolvedValue({
      components: [
        {
          label: "fire damage",
          outcome: {
            formula: "1d6",
            total: 4,
            expression: {
              kind: "dice",
              dice: [{ sides: 6, value: 4, kept: true }],
              total: 4,
            },
          },
        },
      ],
    });
    const field = detailFixture("items:source", "Source", "rules:other").presentation
      .content[0];
    const request = {
      source: field.locator,
      source_fingerprint: "current",
      content_interpretation_version: "foundry-content/v12",
      ordinal: 1,
      authored_formula: "1d6[fire]",
      authored_command: null,
      encounter: {
        encounter_ref: "example",
        participant_key: "participant",
        participant_revision: 1n,
        artifact_generation: "generation",
        condition_id: null,
      },
    };
    field.body = {
      kind: "html",
      inclusions: [],
      html: '<p><span data-atlas-interaction="0">1 fire damage</span>; <span data-atlas-interaction="1"><span data-atlas-roll-formula="">1d6</span> fire damage</span></p>',
      controls: [
        {
          ordinal: 0,
          control: {
            kind: "damage",
            formula: "1[fire]",
            options: {},
            literal: {
              heading: "Fire damage",
              components: [
                {
                  formula: "1",
                  label: "fire damage",
                  unit: null,
                  expression: { kind: "number", value: 1 },
                },
              ],
            },
            roll: null,
          },
        },
        {
          ordinal: 1,
          control: {
            kind: "damage",
            formula: "1d6[fire]",
            options: {},
            literal: {
              heading: "Fire damage",
              components: [
                {
                  formula: "1d6",
                  label: "fire damage",
                  unit: null,
                  expression: { kind: "dice", sides: [6] },
                },
              ],
            },
            roll: request,
          },
        },
      ],
    };
    const props = { content: field, onReference: vi.fn() };
    const { rerender } = render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <PreparedContent {...props} />
      </ConfigProvider>,
    );
    expect(screen.getByText("1 fire damage")).not.toHaveAttribute("role", "button");
    const formula = screen.getByText("1d6", { selector: "[data-atlas-roll-formula]" });
    const roller = screen.getByRole("button", { name: "Roll 1d6 fire damage" });
    expect(formula.nextElementSibling).toContainElement(roller);
    fireEvent.click(roller);
    await screen.findByRole("status", { name: "1d6: 4" });
    rerender(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <PreparedContent {...props} content={structuredClone(field)} />
      </ConfigProvider>,
    );
    expect(await screen.findByRole("status", { name: "1d6: 4" })).toBeVisible();
    expect(roll).toHaveBeenCalledTimes(1);
    expect(roll).toHaveBeenCalledWith(request);
    const changed = structuredClone(field);
    if (
      changed.body.kind === "html" &&
      changed.body.controls[1].control.kind === "damage"
    ) {
      changed.body.controls[1].control.roll =
        binding === "source"
          ? {
              ...request,
              source_fingerprint: "replacement",
            }
          : {
              ...request,
              encounter: { ...request.encounter, participant_revision: 2n },
            };
    }
    rerender(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <PreparedContent {...props} content={changed} />
      </ConfigProvider>,
    );
    expect(screen.getByRole("button", { name: "Roll 1d6 fire damage" })).toBeVisible();
    expect(screen.queryByRole("status", { name: "1d6: 4" })).toBeNull();
    expect(roll).toHaveBeenCalledTimes(1);
    roll.mockRestore();
  },
);

it("gives repeated inclusions of the same authored damage independent roll controls", () => {
  const field = detailFixture("items:host", "Host", "rules:other").presentation
    .content[0];
  const source = {
    record: "items:included",
    embedded_path: [],
    field: "/system/description/value",
  };
  const control = {
    ordinal: 0,
    control: {
      kind: "damage" as const,
      formula: "1d6[fire]",
      options: {},
      literal: {
        heading: "Fire damage",
        components: [
          {
            formula: "1d6",
            label: "fire damage",
            unit: null,
            expression: { kind: "dice" as const, sides: [6] },
          },
        ],
      },
      roll: {
        source,
        source_fingerprint: "current",
        content_interpretation_version: "foundry-content/v12",
        ordinal: 0,
        authored_formula: "1d6[fire]",
        authored_command: null,
      },
    },
  };
  field.body = {
    kind: "html",
    controls: [],
    html: '<section data-atlas-inclusion="0"><span data-atlas-interaction="0">1d6 fire damage</span></section><section data-atlas-inclusion="1"><span data-atlas-interaction="0">1d6 fire damage</span></section>',
    inclusions: [0, 1].map((namespace) => ({
      namespace,
      source,
      controls: [control],
      references: [],
    })),
  };
  render(<PreparedContent content={field} onReference={vi.fn()} />);
  expect(screen.getAllByRole("button", { name: "Roll 1d6 fire damage" })).toHaveLength(
    2,
  );
});

it.each(["r", "gmr", "br"])(
  "renders /%s captions with the shared source-bound inline roller",
  async (command) => {
    const roll = vi.spyOn(api, "rollContent").mockResolvedValue({
      components: [
        {
          label: "rounds",
          outcome: {
            formula: "1d4",
            total: 3,
            expression: {
              kind: "dice",
              dice: [{ sides: 4, value: 3, kept: true }],
              total: 3,
            },
          },
        },
      ],
    });
    const field = detailFixture("items:source", "Source", "rules:other").presentation
      .content[0];
    const request = {
      source: field.locator,
      source_fingerprint: "current",
      content_interpretation_version: "foundry-content/v13",
      ordinal: 0,
      authored_formula: "1d4 #rounds",
      authored_command: command,
    };
    field.body = {
      kind: "html",
      inclusions: [],
      html: '<p>Wait <span data-atlas-interaction="0"><span data-atlas-roll-formula="">1d4</span> rounds</span>.</p>',
      controls: [
        {
          ordinal: 0,
          control: {
            kind: "command",
            command,
            arguments: "1d4 #rounds",
            options: {},
            literal: {
              heading: "Duration",
              components: [
                {
                  formula: "1d4",
                  label: "rounds",
                  unit: { singular: "round", plural: "rounds" },
                  expression: { kind: "dice", sides: [4] },
                },
              ],
            },
            roll: request,
          },
        },
      ],
    };
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <PreparedContent content={field} onReference={vi.fn()} />
      </ConfigProvider>,
    );
    const button = screen.getByRole("button", { name: "Roll 1d4 rounds" });
    expect(
      screen.getByText("1d4", { selector: "[data-atlas-roll-formula]" })
        .nextElementSibling,
    ).toContainElement(button);
    fireEvent.click(button);
    expect(await screen.findByRole("status", { name: "1d4: 3 rounds" })).toBeVisible();
    expect(roll).toHaveBeenCalledWith(request);
    roll.mockRestore();
  },
);

it("keeps an authored self-action label readable without reopening the same action", () => {
  const detail = detailFixture("actions:seek", "Seek", "actions:seek");
  const content = detail.presentation.content[0];
  content.body = {
    kind: "html",
    inclusions: [],
    html: '<p>A secret <span data-atlas-reference="0">Perception <span data-atlas-action-glyph="one_action">one action</span></span> check.</p>',
    controls: [],
  };
  const reference = { ...detail.relationships[0], self_action_reference: true };
  const onReference = vi.fn();
  render(
    <PreparedContent
      content={content}
      relationships={[reference]}
      onReference={onReference}
    />,
  );
  const label = screen.getByText("Perception", {
    exact: false,
    selector: "[data-atlas-reference]",
  });
  expect(label).not.toHaveAttribute("aria-disabled");
  expect(label).not.toHaveAttribute("tabindex");
  expect(screen.queryByRole("link", { name: /Perception/ })).toBeNull();
  expect(screen.queryByRole("button", { name: /Perception/ })).toBeNull();
  fireEvent.click(label);
  expect(onReference).not.toHaveBeenCalled();
  expect(reference.target?.record_key).toBe("actions:seek");
});

it.each([
  {
    text: "DC 28 basic Reflex save",
    control: {
      kind: "check" as const,
      statistic: "reflex",
      options: { dc: "28", basic: "" },
    },
  },
  {
    text: "120-foot line (10 feet wide)",
    control: {
      kind: "template" as const,
      shape: "line",
      options: { distance: "120", width: "10" },
    },
  },
])(
  "renders $text as plain text without a popup or keyboard stop",
  ({ text, control }) => {
    const field = detailFixture("items:source", "Source", "rules:target").presentation
      .content[0];
    field.body = {
      kind: "html",
      html: `<p><span data-atlas-interaction="0" role="button" tabindex="0">${text}</span></p>`,
      controls: [{ ordinal: 0, control }],
      inclusions: [],
    };
    const onReference = vi.fn();
    render(<PreparedContent content={field} onReference={onReference} />);
    const label = screen.getByText(text);
    expect(label).not.toHaveAttribute("role");
    expect(label).not.toHaveAttribute("tabindex");
    fireEvent.click(label);
    fireEvent.keyDown(label, { key: "Enter" });
    fireEvent.keyDown(label, { key: " " });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.queryByRole("button")).toBeNull();
    expect(onReference).not.toHaveBeenCalled();
  },
);
