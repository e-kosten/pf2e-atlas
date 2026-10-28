import { act, fireEvent, render, screen } from "@testing-library/react";
import { Button } from "antd";
import type { ReactNode } from "react";
import { isPrimaryActivation } from "../../../app/navigation";
import { IndexTable, RowTitleLink } from "./IndexTable";

const row = { key: "goblins", name: "Goblins" };

function renderIndex(content: ReactNode) {
  const onActivateRow = vi.fn();
  render(
    <IndexTable
      columns={[
        { title: "Name", render: () => content },
        { title: "Status", render: () => "Ready" },
      ]}
      dataSource={[row]}
      onActivateRow={onActivateRow}
      rowKey={(entry) => entry.key}
    />,
  );
  return { onActivateRow, tableRow: screen.getByText("Ready").closest("tr")! };
}

describe("IndexTable", () => {
  it("does not flash completed-empty copy and preserves rows through refresh", () => {
    const props = {
      columns: [{ title: "Name", dataIndex: "name" }],
      onActivateRow: vi.fn(),
      rowKey: (entry: typeof row) => entry.key,
      locale: { emptyText: "No records" },
    };
    const view = render(
      <IndexTable {...props} dataSource={[]} loading locale={{ emptyText: null }} />,
    );
    expect(screen.queryByText("No records")).not.toBeInTheDocument();
    view.rerender(<IndexTable {...props} dataSource={[row]} loading={false} />);
    const name = screen.getByText(row.name);
    view.rerender(<IndexTable {...props} dataSource={[row]} loading />);
    expect(screen.getByText(row.name)).toBe(name);
    view.rerender(
      <IndexTable {...props} dataSource={[]} loading={{ spinning: false }} />,
    );
    expect(screen.getByText("No records")).toBeInTheDocument();
    const empty = screen.getByText("No records");
    view.rerender(<IndexTable {...props} dataSource={[]} loading />);
    expect(screen.getByText("No records")).toBe(empty);
  });

  it("activates once from a plain cell click or Enter focused on the row", () => {
    const { onActivateRow, tableRow } = renderIndex(row.name);
    fireEvent.click(screen.getByText("Ready"));
    expect(onActivateRow).toHaveBeenCalledExactlyOnceWith(row);
    onActivateRow.mockClear();

    act(() => tableRow.focus());
    fireEvent.keyDown(tableRow, { key: "Enter" });
    expect(onActivateRow).toHaveBeenCalledExactlyOnceWith(row);
  });

  it.each(["Edit", "Delete"])(
    "leaves nested %s keyboard and click activation to its action",
    (label) => {
      const onAction = vi.fn();
      const { onActivateRow } = renderIndex(
        <Button onClick={onAction}>{label}</Button>,
      );
      const button = screen.getByRole("button", { name: label });
      act(() => button.focus());
      fireEvent.keyDown(button, { key: "Enter" });
      expect(onActivateRow).not.toHaveBeenCalled();

      // JSDOM does not synthesize the browser's click after keyboard activation.
      fireEvent.click(button);
      expect(onAction).toHaveBeenCalledTimes(1);
      expect(onActivateRow).not.toHaveBeenCalled();
    },
  );

  it("does not treat typing Enter in an embedded field as row activation", () => {
    const { onActivateRow } = renderIndex(<input aria-label="Name" />);
    const input = screen.getByRole("textbox", { name: "Name" });
    act(() => input.focus());
    fireEvent.keyDown(input, { key: "Enter" });
    fireEvent.click(input);
    expect(onActivateRow).not.toHaveBeenCalled();
  });

  it("uses the title link's navigation once without activating its row", () => {
    const onNavigate = vi.fn();
    const { onActivateRow } = renderIndex(
      <RowTitleLink
        href="/lists/goblins"
        onClick={(event) => {
          if (isPrimaryActivation(event)) {
            event.preventDefault();
            onNavigate();
          }
        }}
        title={row.name}
      />,
    );
    const link = screen.getByRole("link", { name: row.name });
    act(() => link.focus());
    fireEvent.keyDown(link, { key: "Enter" });
    expect(onActivateRow).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText(row.name));
    expect(onNavigate).toHaveBeenCalledTimes(1);
    expect(onActivateRow).not.toHaveBeenCalled();
    expect(link).toHaveAttribute("href", "/lists/goblins");
  });

  it.each([
    { metaKey: true },
    { ctrlKey: true },
    { shiftKey: true },
    { altKey: true },
    { button: 1 },
  ])("preserves native modified/nonprimary title clicks: %j", (modifiers) => {
    const onNavigate = vi.fn();
    const { onActivateRow } = renderIndex(
      <RowTitleLink
        href="/lists/goblins"
        onClick={(event) => {
          if (isPrimaryActivation(event)) {
            event.preventDefault();
            onNavigate();
          }
        }}
        title={row.name}
      />,
    );
    expect(
      fireEvent.click(screen.getByRole("link", { name: row.name }), modifiers),
    ).toBe(true);
    expect(onNavigate).not.toHaveBeenCalled();
    expect(onActivateRow).not.toHaveBeenCalled();
  });

  it.each([
    { metaKey: true },
    { ctrlKey: true },
    { shiftKey: true },
    { altKey: true },
    { button: 1 },
    { button: 2 },
  ])("ignores modified/nonprimary cell clicks: %j", (modifiers) => {
    const { onActivateRow } = renderIndex(row.name);
    fireEvent.click(screen.getByText("Ready"), modifiers);
    expect(onActivateRow).not.toHaveBeenCalled();
  });
});
