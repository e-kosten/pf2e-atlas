import { fireEvent, screen } from "@testing-library/react";
import type { SortDirection, SortField } from "../generated/atlas";

/** Use the Select's public keyboard/ARIA surface, including selecting its current value. */
export async function selectSortOption(name: string, label: string) {
  const input = await screen.findByRole("combobox", { name });
  fireEvent.mouseDown(input);
  for (let step = 0; step < 12; step++) {
    const target = screen.queryByRole("option", { name: label });
    if (target?.id && input.getAttribute("aria-activedescendant") === target.id) {
      fireEvent.keyDown(input, { key: "Enter", keyCode: 13 });
      return;
    }
    fireEvent.keyDown(input, { key: "ArrowDown", keyCode: 40 });
  }
  throw new Error(`Option ${label} was not reachable in ${name}`);
}

export async function chooseAttributeSort(field: SortField, direction: SortDirection) {
  fireEvent.click(screen.getByRole("button", { name: /^Sort:/ }));
  await screen.findByRole("dialog", { name: "Sort results" });
  await selectSortOption("Sort mode", "By field");
  await selectSortOption(
    "Sort field 1",
    { name: "Name", level: "Level / Rank", unit_price: "Unit price" }[field],
  );
  await selectSortOption(
    "Sort direction 1",
    field === "name"
      ? direction === "asc"
        ? "A–Z"
        : "Z–A"
      : direction === "asc"
        ? "Low to high"
        : "High to low",
  );
  fireEvent.keyDown(screen.getByRole("dialog", { name: "Sort results" }), {
    key: "Escape",
  });
}
