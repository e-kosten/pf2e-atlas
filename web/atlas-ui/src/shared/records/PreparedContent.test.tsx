import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { PreparedContent } from "./PreparedContent";
import { atlasRoutePath } from "../../app/routes";
import { detailFixture } from "../../test/fixtures";
import { ConfigProvider, theme } from "antd";

it.each(["Escape", "backdrop"])(
  "dismisses source information with %s and restores its inline control",
  async (interaction) => {
    const content = detailFixture("actors:source", "Source", "rules:other").presentation
      .content[0];
    content.body = {
      kind: "html",
      html: '<span data-atlas-interaction="0">Fortitude check</span>',
      controls: [
        {
          ordinal: 0,
          control: { kind: "check", statistic: "Fortitude", options: { dc: "15" } },
        },
      ],
    };
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <PreparedContent content={content} onReference={vi.fn()} />
      </ConfigProvider>,
    );
    const trigger = screen.getByRole("button", { name: "Fortitude check" });
    fireEvent.click(trigger);
    const dialog = await screen.findByRole("dialog");
    if (interaction === "Escape") fireEvent.keyDown(dialog, { key: "Escape" });
    else {
      fireEvent.mouseDown(document.querySelector(".ant-modal-wrap")!);
      fireEvent.click(document.querySelector(".ant-modal-wrap")!);
    }
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(trigger).toHaveFocus();
  },
);

describe("prepared source HTML", () => {
  it("renders compact effects with native destinations and refreshes without duplicate controls", () => {
    const detail = detailFixture("spells:source", "Source", "effects:target");
    const field = detail.presentation.content[0];
    field.body = {
      kind: "html",
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
      atlasRoutePath({
        kind: "record",
        recordKey: edge.target!.record_key,
        selection: edge.target!,
      }),
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
  it("handles nested controls once and leaves native child links uncancelled", async () => {
    const detail = detailFixture("actors:ghoul", "Ghoul", "rules:outer");
    const content = detail.presentation.content[0];
    content.body = {
      kind: "html",
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
    expect(await screen.findAllByRole("dialog")).toHaveLength(3);
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
    expect(callback).not.toHaveBeenCalled();
  });
  it("opens keyboard control dialogs with the surrounding Ant configuration in dark mode", async () => {
    const content = detailFixture("actors:ghoul", "Ghoul", "rules:outer").presentation
      .content[0];
    content.body = {
      kind: "html",
      html: '<span data-atlas-interaction="0">Fortitude check</span>',
      controls: [
        {
          ordinal: 0,
          control: { kind: "check", statistic: "Fortitude", options: { dc: "15" } },
        },
      ],
    };
    render(
      <ConfigProvider prefixCls="themed" theme={{ algorithm: theme.darkAlgorithm }}>
        <PreparedContent content={content} onReference={vi.fn()} />
      </ConfigProvider>,
    );
    fireEvent.keyDown(screen.getByRole("button", { name: "Fortitude check" }), {
      key: "Enter",
    });
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveClass("themed-modal");
    expect(dialog).toHaveTextContent("Fortitude");
    expect(dialog).toHaveTextContent("15");
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
      atlasRoutePath({
        kind: "record",
        recordKey: "rules:nested",
        selection: detail.relationships[0].target!,
      }),
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
      html: '<h2>Heading</h2><script>alert(1)</script><a href="javascript:alert(1)">Unsafe</a><span data-atlas-interaction="0">Unknown command</span>',
      controls: [
        {
          ordinal: 0,
          control: { kind: "command", command: "Mystery", arguments: "", options: {} },
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
    atlasRoutePath({ kind: "record", recordKey: target.record_key, selection: target }),
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
  const original = edge.target;
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
    atlasRoutePath({
      kind: "record",
      recordKey: "equipment:wand",
      selection: edge.contextual_target,
    }),
  );
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith(
    "equipment:wand",
    link,
    edge.contextual_target,
  );
  expect(edge.target).toEqual(original);
});
