import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { Button, ConfigProvider, Popover } from "antd";
import { createRef, type ComponentRef } from "react";
import { InlineContext, InlineIconButton, InlineIconLink } from "./InlineAction";

it("opens caption-only help on a direct click, keeps a focus-before-click open, and dismisses outside/Escape without focus loss", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <input aria-label="Another control" />
      <InlineIconButton
        label="XP award calculation"
        title="Encounter XP divided between PCs"
        icon={<span aria-hidden>i</span>}
      />
    </ConfigProvider>,
  );
  const button = screen.getByRole("button", { name: "XP award calculation" });
  fireEvent.click(button);
  expect(await screen.findByRole("tooltip")).toHaveTextContent(
    "Encounter XP divided between PCs",
  );
  expect(button).toHaveFocus();
  fireEvent.click(button);
  await waitFor(() => expect(screen.getByRole("tooltip")).toBeVisible());
  const input = screen.getByRole("textbox", { name: "Another control" });
  fireEvent.pointerDown(input);
  act(() => input.focus());
  await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
  expect(input).toHaveFocus();
  act(() => button.focus());
  await waitFor(() => expect(screen.getByRole("tooltip")).toBeVisible());
  fireEvent.click(button);
  fireEvent.keyDown(button, { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
  expect(button).toHaveFocus();
});

it("forwards a controlled overlay's ref/events and opens the caller's Popover on click", async () => {
  const ref = createRef<ComponentRef<typeof Button>>();
  const key = vi.fn();
  const focus = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <Popover content="Caller-owned qualification" trigger="click">
        <InlineIconButton
          ref={ref}
          tooltip={false}
          label="Condition details"
          icon={<span aria-hidden>i</span>}
          onKeyDown={key}
          onFocus={focus}
        />
      </Popover>
    </ConfigProvider>,
  );
  const button = screen.getByRole("button", { name: "Condition details" });
  expect(ref.current).toBe(button);
  act(() => button.focus());
  expect(focus).toHaveBeenCalledOnce();
  fireEvent.keyDown(button, { key: "Enter" });
  expect(key).toHaveBeenCalledOnce();
  fireEvent.click(button);
  await waitFor(() =>
    expect(screen.getByText("Caller-owned qualification")).toBeVisible(),
  );
});

it("keeps one original content anchor, exact copy and caller-owned native activation", () => {
  const ref = createRef<ComponentRef<typeof Button>>();
  const click = vi.fn();
  const text = "A descriptive linked record name";
  const { container } = render(
    <InlineContext
      actions={
        <InlineIconLink
          ref={ref}
          label="Read explanation"
          href="/records/explanation"
          tooltip={false}
          icon={<span aria-hidden />}
          onClick={click}
        />
      }
    >
      <a href="/records/original">{text}</a>
    </InlineContext>,
  );
  expect(container.textContent).toBe(text);
  expect(container.querySelectorAll('a[href="/records/original"]')).toHaveLength(1);
  const action = screen.getByRole("link", { name: "Read explanation" });
  expect(ref.current).toBe(action);
  expect(action).toHaveAttribute("href", "/records/explanation");
  expect(fireEvent.click(action, { metaKey: true })).toBe(true);
  expect(click).toHaveBeenCalledOnce();
  expect(click.mock.calls[0][0].metaKey).toBe(true);
});
