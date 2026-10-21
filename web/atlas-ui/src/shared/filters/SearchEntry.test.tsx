import { ConfigProvider } from "antd";
import { fireEvent, render, screen } from "@testing-library/react";
import { SearchEntry } from "./SearchEntry";

it("keeps keyword edits local until Enter, then restores incoming committed criteria", () => {
  const commit = vi.fn();
  const view = render(<SearchEntry query="fire" onSearch={commit} />);
  const input = screen.getByRole("searchbox", { name: "Search records" });
  fireEvent.change(input, { target: { value: "ice" } });
  expect(commit).not.toHaveBeenCalled();
  fireEvent.keyDown(input, { key: "Enter", keyCode: 13 });
  expect(commit).toHaveBeenCalledWith("ice");
  view.rerender(<SearchEntry query="acid" onSearch={commit} />);
  expect(input).toHaveValue("acid");
});

it("inherits pending form disabling from the Ant provider", () => {
  render(
    <ConfigProvider componentDisabled>
      <SearchEntry query="fire" onSearch={vi.fn()} />
    </ConfigProvider>,
  );
  expect(screen.getByRole("searchbox")).toBeDisabled();
});

it("clears committed criteria when the last character is deleted", () => {
  const commit = vi.fn();
  render(<SearchEntry query="no matches" onSearch={commit} />);
  fireEvent.change(screen.getByRole("searchbox"), { target: { value: "" } });
  expect(commit).toHaveBeenCalledWith("");
});

it("replaces text entry with a selectable Similar title and restores focus after clearing", () => {
  const clear = vi.fn();
  const view = render(
    <SearchEntry
      query=""
      onSearch={vi.fn()}
      similar={{ title: "Leather Armor", onClear: clear }}
    />,
  );
  expect(screen.queryByRole("searchbox")).not.toBeInTheDocument();
  expect(screen.getByText("Similar to")).toBeVisible();
  expect(screen.getByText("Leather Armor")).toBeVisible();
  expect(screen.getByText("Leather Armor")).not.toHaveAttribute("contenteditable");
  fireEvent.click(screen.getByRole("button", { name: "Clear similar criterion" }));
  expect(clear).toHaveBeenCalledOnce();
  view.rerender(<SearchEntry query="" onSearch={vi.fn()} />);
  expect(screen.getByRole("searchbox")).toHaveFocus();
});
