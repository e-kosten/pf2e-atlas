import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import type { RuleGroupType } from "react-querybuilder";
import { CollectionValueInput } from "./CollectionValueInput";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import { DEFAULT_SEARCH_STATE } from "./searchState";

it("shows local errors for a malformed optional amount and clears them on correction", async () => {
  const editor = editorFixture([
    fieldFixture("movement.speeds", "collection", "Movement Speeds"),
    {
      ...fieldFixture("movement.speeds[].type", "string", "Movement type"),
      choices: ["fly"],
    },
    fieldFixture("movement.speeds[].feet", "number", "Speed"),
  ]);
  const field = editor.groups[0].fields[0];
  field.collection_control = {
    category_field_id: "movement.speeds[].type",
    amount_field_id: "movement.speeds[].feet",
  };
  function Harness() {
    const [query, setQuery] = useState<RuleGroupType>({ combinator: "and", rules: [] });
    return (
      <CollectionValueInput
        field={field}
        editor={editor}
        query={query}
        onChange={setQuery}
        workspace={{
          search: DEFAULT_SEARCH_STATE,
          setSearch: vi.fn(),
          filterEditor: editor,
        }}
      />
    );
  }
  render(
    <QueryClientProvider client={new QueryClient()}>
      <Harness />
    </QueryClientProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Add speed" }));
  const input = screen.getByRole("textbox", { name: "Speed" });
  fireEvent.change(input, { target: { value: "abc" } });
  expect(screen.getByRole("status")).toHaveTextContent("Enter a finite numeric value");
  fireEvent.change(input, { target: { value: "30" } });
  await waitFor(() => expect(screen.queryByRole("status")).not.toBeInTheDocument());
  expect(input).toHaveValue("30");
});
