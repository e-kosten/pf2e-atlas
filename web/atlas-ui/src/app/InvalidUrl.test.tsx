import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { createMemoryHistory } from "@tanstack/react-router";
import { RouterTestContext } from "../test/router";
import { encodeFilterUrl } from "../shared/filters/filterUrl";
import { InvalidUrl, LinkNotFound } from "./InvalidUrl";
import { ReadingUrlError } from "./readingUrl";
import { createAtlasRouter } from "./router";

it("offers page recovery for unexpected failures without exposing the exception or blaming the link", () => {
  render(
    <RouterTestContext>
      <InvalidUrl error={new Error("decoder at /private/reference.sqlite")} />
    </RouterTestContext>,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("This page couldn’t open");
  expect(screen.getByRole("button", { name: "Reload page" })).toBeVisible();
  expect(
    screen.queryByText(
      /decoder|private\/reference.sqlite|This link couldn’t be restored/,
    ),
  ).toBeNull();
});

it.each(["search", "list"])(
  "offers native whole-record and independent %s criteria recovery for an invalid reading address",
  async (kind) => {
    const filter = encodeFilterUrl({
      kind: "in",
      clause_id: "example",
      field: "kind",
      values: ["spell"],
    });
    const path =
      kind === "search"
        ? "/search/records/spells:heal"
        : "/lists/research/records/spells:heal";
    const router = createAtlasRouter(
      createMemoryHistory({
        initialEntries: [`${path}?q=heal&filter=${filter}&rank=bad`],
      }),
    );
    await router.load();
    const error = Object.assign(new Error("Search validation failed"), {
      cause: new ReadingUrlError("Invalid spell rank"),
    });
    render(
      <RouterTestContext router={router}>
        <InvalidUrl error={error} />
      </RouterTestContext>,
    );
    expect(screen.getByText("Invalid spell rank")).toBeVisible();
    const actions = within(screen.getByRole("group", { name: "Link recovery" }));
    expect(actions.getAllByRole("link")).toHaveLength(3);
    expect(screen.getByRole("link", { name: "Open record" })).toHaveAttribute(
      "href",
      "/records/spells:heal",
    );
    const results = screen.getByRole("link", { name: "Return to results" });
    expect(results).toHaveAttribute(
      "href",
      `${kind === "search" ? "/search" : "/lists/research"}?q=heal&filter=${filter}`,
    );
    await act(async () => {
      fireEvent.click(results);
    });
    await waitFor(() =>
      expect(router.latestLocation.href).toBe(
        `${kind === "search" ? "/search" : "/lists/research"}?q=heal&filter=${filter}`,
      ),
    );
    expect(router.history.length).toBe(1);
  },
);

it("keeps an unknown route visible and offers a native Open Search action", async () => {
  const router = createAtlasRouter(
    createMemoryHistory({ initialEntries: ["/record/mistyped"] }),
  );
  await router.load();
  render(
    <RouterTestContext router={router}>
      <LinkNotFound />
    </RouterTestContext>,
  );
  expect(screen.getByText("Link not found")).toBeVisible();
  expect(screen.getByRole("link", { name: "Open Search" })).toHaveAttribute(
    "href",
    "/search",
  );
  expect(router.latestLocation.pathname).toBe("/record/mistyped");
  expect(
    router.state.matches[router.state.matches.length - 1]?.staticData.section,
  ).toBe("reference");
});
