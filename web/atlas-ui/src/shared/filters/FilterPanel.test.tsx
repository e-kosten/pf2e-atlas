import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { FilterControls } from "./FilterPanel";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "./searchState";
import { editorFixture, fieldFixture } from "../../test/fixtures";
describe("maintained structured query editor", () => {
  it("shows backend fields and blocks unsafe numeric input", async () => {
    const onChange = vi.fn();
    function Harness() {
      const [search, setSearch] = useState<SearchFormState>({
        ...DEFAULT_SEARCH_STATE,
        filter: {
          kind: "compare",
          clause_id: "level",
          field: "actor.level",
          op: "gte",
          value: 1,
        },
      });
      return (
        <FilterControls
          includeSearch={false}
          filterState={{
            search,
            setSearch: (next) => {
              setSearch(next);
              onChange(next);
            },
            filterEditor: editorFixture([
              fieldFixture("actor.level", "number", "Actor level"),
            ]),
            filterValuesByField: {},
          }}
        />
      );
    }
    render(<Harness />);
    const input = await screen.findByRole("textbox", { name: "Actor level" });
    fireEvent.change(input, { target: { value: "9007199254740993" } });
    await waitFor(() =>
      expect(onChange).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filterError: expect.stringContaining("exact numeric range"),
        }),
      ),
    );
    expect(screen.getByText(/exact numeric range/)).toBeInTheDocument();
  });
  it("reports backend facet restrictions without disabling the builder", () => {
    render(
      <FilterControls
        filterState={{
          search: DEFAULT_SEARCH_STATE,
          setSearch: vi.fn(),
          filterEditor: editorFixture([fieldFixture("common.traits", "set", "Traits")]),
          filterValuesByField: {},
          errorMessage: "Contextual facets do not support OR/NOT",
        }}
      />,
    );
    expect(
      screen.getByText("Contextual facets do not support OR/NOT"),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Add rule")).toBeEnabled();
  });
});
