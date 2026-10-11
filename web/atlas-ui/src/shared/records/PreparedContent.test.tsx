import { fireEvent, render, screen } from "@testing-library/react";
import { PreparedContent } from "./PreparedContent";
import { detailFixture } from "../../test/fixtures";
import { ConfigProvider, theme } from "antd";
describe("prepared source HTML", () => {
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
    expect(reference).toHaveAttribute("role", "button");
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
