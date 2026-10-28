import { fireEvent, render, screen } from "@testing-library/react";
import { Button } from "antd";
import { ErrorNotice } from "./ErrorNotice";
import { AtlasApiError } from "../../api/atlasApi";
import { AtlasCompatibilityError } from "../../api/compatibility";
import { AtlasConnectionError } from "../../api/availability";

it("keeps an independent action failure actionable without exposing diagnostics", () => {
  const retry = vi.fn();
  render(
    <ErrorNotice
      error={
        new AtlasApiError(500, "SQL at /private/workspace.sqlite", {
          code: "internal_error",
          message: "SQL at /private/workspace.sqlite",
        })
      }
      title="Note could not be saved"
      action={<Button onClick={retry}>Retry note</Button>}
    />,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("Note could not be saved");
  expect(screen.queryByText(/SQL|workspace.sqlite/)).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry note" }));
  expect(retry).toHaveBeenCalledTimes(1);
});

it("leaves shared failures to the shell while keeping unrelated failures visible", () => {
  render(
    <>
      <ErrorNotice
        error={new AtlasCompatibilityError("old")}
        title="Encounter could not load"
      />
      <ErrorNotice
        error={new AtlasConnectionError(false, new TypeError("Failed to fetch"))}
        title="Conditions could not load"
      />
      <ErrorNotice
        error={new Error("raw response body")}
        title="Name could not be saved"
      />
    </>,
  );
  expect(screen.getAllByRole("alert")).toHaveLength(1);
  expect(screen.getByRole("alert")).toHaveTextContent("Name could not be saved");
  expect(
    screen.queryByText(
      /Encounter could not load|Conditions could not load|raw response body/,
    ),
  ).toBeNull();
});

it("announces secondary availability politely and keeps its recovery local", () => {
  render(
    <ErrorNotice
      compact
      error={new Error("decoder detail")}
      title="Recovery choices unavailable"
      action={<Button>Retry recovery</Button>}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent("Recovery choices unavailable");
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.getByRole("button", { name: "Retry recovery" })).toBeVisible();
});

it("keeps an unconfirmed write outcome beside its action without claiming it was unsaved", () => {
  render(
    <ErrorNotice
      error={new AtlasConnectionError(true, "private transport details")}
      title="List could not be created"
    />,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("Change could not be confirmed");
  expect(screen.getByRole("alert")).toHaveTextContent(
    "Check the current values after reconnecting",
  );
  expect(
    screen.queryByText(/List could not be created|private transport details/),
  ).toBeNull();
});

it("preserves a confirmed save and its read recovery during a shared outage", () => {
  const retryRead = vi.fn();
  render(
    <ErrorNotice
      outcome="saved"
      error={new AtlasConnectionError(false, "private diagnostic")}
      title="Saved, but the encounter could not refresh"
      action={<Button onClick={retryRead}>Retry encounter refresh</Button>}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent(
    "Saved, but the encounter could not refresh",
  );
  expect(screen.queryByRole("alert")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry encounter refresh" }));
  expect(retryRead).toHaveBeenCalledTimes(1);
});

it("preserves a partial save when a following write has an unknown outcome", () => {
  render(
    <ErrorNotice
      outcome="saved"
      error={new AtlasConnectionError(true, "private diagnostic")}
      title="Research was created. Adding the record could not be confirmed."
    />,
  );
  expect(screen.getByRole("alert")).toHaveTextContent(
    "Research was created. Adding the record could not be confirmed.",
  );
  expect(screen.queryByText("private diagnostic")).toBeNull();
});
