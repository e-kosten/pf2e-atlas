import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { Button, ConfigProvider, Tooltip } from "antd";
import { createRef, StrictMode, type ComponentRef } from "react";
import { PaneIconButton, PaneIconLink } from "./PaneAction";

it("connects native Ant Tooltip hover and focus to the same pane button and ref", async () => {
  const ref = createRef<ComponentRef<typeof Button>>();
  const click = vi.fn();
  const focus = vi.fn();
  render(
    <StrictMode>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <Tooltip
          destroyOnHidden
          mouseEnterDelay={0}
          mouseLeaveDelay={0}
          title="Choose a saved list"
          trigger={["hover", "focus"]}
        >
          <PaneIconButton
            ref={ref}
            icon={<span aria-hidden>+</span>}
            label="Add to saved list"
            onClick={click}
            onFocus={focus}
          />
        </Tooltip>
      </ConfigProvider>
    </StrictMode>,
  );
  const button = screen.getByRole("button", { name: "Add to saved list" });
  expect(ref.current).toBe(button);
  expect(button).toHaveClass("pane-toggle");
  expect(button).toHaveAttribute("title", "Add to saved list");
  fireEvent.mouseEnter(button);
  expect(await screen.findByRole("tooltip")).toHaveTextContent("Choose a saved list");
  fireEvent.mouseLeave(button);
  await waitFor(() => expect(screen.queryByRole("tooltip")).not.toBeInTheDocument());
  act(() => button.focus());
  expect(button).toHaveFocus();
  expect(focus).toHaveBeenCalledTimes(1);
  expect(await screen.findByRole("tooltip")).toHaveTextContent("Choose a saved list");
  fireEvent.click(button);
  expect(click).toHaveBeenCalledTimes(1);
  act(() => button.blur());
  await waitFor(() => expect(screen.queryByRole("tooltip")).not.toBeInTheDocument());
});

it("preserves disabled actions and native link attributes, events and refs", () => {
  const click = vi.fn();
  const ref = createRef<ComponentRef<typeof Button>>();
  const { rerender } = render(
    <PaneIconButton
      disabled
      icon={<span aria-hidden>−</span>}
      label="Remove participant"
      onClick={click}
    />,
  );
  const disabled = screen.getByRole("button", { name: "Remove participant" });
  expect(disabled).toBeDisabled();
  fireEvent.click(disabled);
  expect(click).not.toHaveBeenCalled();
  const keyDown = vi.fn();
  rerender(
    <PaneIconLink
      ref={ref}
      className="record-link"
      href="/records/equipment:staff"
      icon={<span aria-hidden>↗</span>}
      label="Open full page"
      onKeyDown={keyDown}
      target="_blank"
      title="Read this record"
    />,
  );
  const link = screen.getByRole("link", { name: "Open full page" });
  expect(ref.current).toBe(link);
  expect(link).toHaveAttribute("href", "/records/equipment:staff");
  expect(link).toHaveAttribute("target", "_blank");
  expect(link).toHaveAttribute("title", "Read this record");
  expect(link).toHaveClass("pane-toggle", "record-link");
  fireEvent.keyDown(link, { key: "Enter" });
  expect(keyDown).toHaveBeenCalledTimes(1);
});
