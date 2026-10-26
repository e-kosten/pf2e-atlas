import { ConfigProvider, Modal } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { HelpTooltip } from "./HelpTooltip";

it("opens from a native help button and closes on Escape before its containing modal", async () => {
  const cancel = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <Modal open title="Settings" onCancel={cancel}>
        <HelpTooltip label="About example">A readable explanation.</HelpTooltip>
      </Modal>
    </ConfigProvider>,
  );
  const help = screen.getByRole("button", { name: "About example" });
  expect(help.tagName).toBe("BUTTON");
  expect(help).toHaveAttribute("aria-expanded", "false");
  act(() => help.focus());
  await waitFor(() =>
    expect(screen.getByText("A readable explanation.")).toBeVisible(),
  );
  expect(help).toHaveAttribute("aria-expanded", "true");
  expect(help).toHaveAccessibleDescription("A readable explanation.");
  expect(help).toHaveFocus();
  fireEvent.keyDown(help, { key: "Escape" });
  await waitFor(() => expect(screen.queryByText("A readable explanation.")).toBeNull());
  expect(help).toHaveAttribute("aria-expanded", "false");
  expect(help).toHaveFocus();
  expect(cancel).not.toHaveBeenCalled();
  expect(screen.getByRole("dialog", { name: "Settings" })).toBeVisible();
  const modalClose = screen.getByRole("button", { name: "Close" });
  act(() => modalClose.focus());
  fireEvent.pointerDown(help);
  fireEvent.mouseDown(help);
  act(() => help.focus());
  fireEvent.pointerUp(help);
  fireEvent.mouseUp(help);
  fireEvent.click(help);
  await waitFor(() =>
    expect(screen.getByText("A readable explanation.")).toBeVisible(),
  );
  expect(help).toHaveAttribute("aria-expanded", "true");
  expect(help).toHaveFocus();
  fireEvent.keyDown(help, { key: "Escape" });
  expect(cancel).not.toHaveBeenCalled();
});

it("shows hover help without moving focus and dismisses it before modal Escape", async () => {
  const cancel = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <Modal open title="Settings" onCancel={cancel}>
        <input aria-label="Query" />
        <HelpTooltip label="About example">A readable explanation.</HelpTooltip>
      </Modal>
    </ConfigProvider>,
  );
  const input = screen.getByRole("textbox", { name: "Query" });
  const help = screen.getByRole("button", { name: "About example" });
  input.focus();
  fireEvent.mouseEnter(help);
  await waitFor(() =>
    expect(screen.getByText("A readable explanation.")).toBeVisible(),
  );
  expect(input).toHaveFocus();
  fireEvent.keyDown(input, { key: "Escape" });
  await waitFor(() => expect(screen.queryByText("A readable explanation.")).toBeNull());
  expect(help).toHaveFocus();
  expect(cancel).not.toHaveBeenCalled();
});
