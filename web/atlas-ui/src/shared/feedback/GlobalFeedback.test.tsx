import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { Button } from "antd";
import { useState } from "react";
import {
  GlobalFeedback,
  GlobalFeedbackProvider,
  FeedbackModalFrame,
} from "./GlobalFeedback";
import { AtlasDrawer } from "../ui/overlays/AtlasDrawer";
import { AtlasModal } from "../ui/overlays/AtlasModal";

it("keeps one reachable recovery in an open editor, preserves its draft/focus, and returns recovery to the shell on close", async () => {
  let showFailure = () => {};
  function App() {
    const [open, setOpen] = useState(true);
    const [failed, setFailed] = useState(false);
    showFailure = () => setFailed(true);
    return (
      <GlobalFeedbackProvider>
        <section aria-label="Shell">
          <GlobalFeedback>
            {failed && (
              <div role="status">
                Reload Atlas to continue.<Button>Reload page</Button>
              </div>
            )}
          </GlobalFeedback>
        </section>
        <Button onClick={() => setOpen(true)}>Open editor</Button>
        <AtlasModal
          transitionName=""
          open={open}
          title="Draft editor"
          onCancel={() => setOpen(false)}
          footer={null}
        >
          <input aria-label="Name" defaultValue="" />
        </AtlasModal>
      </GlobalFeedbackProvider>
    );
  }
  render(<App />);
  const dialog = await screen.findByRole("dialog", { name: "Draft editor" });
  const input = within(dialog).getByRole("textbox", { name: "Name" });
  fireEvent.change(input, { target: { value: "Retained draft" } });
  input.focus();
  act(showFailure);
  await waitFor(() =>
    expect(within(dialog).getByRole("button", { name: "Reload page" })).toBeVisible(),
  );
  expect(screen.getAllByRole("status")).toHaveLength(1);
  expect(document.activeElement).toBe(input);
  expect(input).toHaveValue("Retained draft");
  fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
  await waitFor(() =>
    expect(
      within(screen.getByRole("region", { name: "Shell" })).getByRole("button", {
        name: "Reload page",
      }),
    ).toBeVisible(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Open editor" }));
  expect(await screen.findByRole("textbox", { name: "Name" })).toHaveValue(
    "Retained draft",
  );
});

it("chooses the innermost active host even when both frames mount together", () => {
  render(
    <GlobalFeedbackProvider>
      <GlobalFeedback>
        <span role="status">Shared recovery</span>
      </GlobalFeedback>
      <FeedbackModalFrame active>
        <section aria-label="Outer">
          <FeedbackModalFrame active>
            <section aria-label="Inner" />
          </FeedbackModalFrame>
        </section>
      </FeedbackModalFrame>
    </GlobalFeedbackProvider>,
  );
  const host = screen.getByRole("status").parentElement!;
  expect(
    host.closest(".feedback-modal-layout")?.querySelector('[aria-label="Inner"]'),
  ).not.toBeNull();
  expect(
    host.closest(".feedback-modal-layout")?.querySelector('[aria-label="Outer"]'),
  ).toBeNull();
  expect(screen.getAllByRole("status")).toHaveLength(1);
});

it("returns shared recovery from a closed nested drawer to its still-open editor", async () => {
  function App() {
    const [open, setOpen] = useState(true);
    return (
      <GlobalFeedbackProvider>
        <GlobalFeedback>
          <Button>Retry connection</Button>
        </GlobalFeedback>
        <AtlasModal open transitionName="" title="Editor" footer={null}>
          <AtlasDrawer open={open} title="Choices" onClose={() => setOpen(false)}>
            <input aria-label="Choice draft" defaultValue="Uncommitted" />
          </AtlasDrawer>
        </AtlasModal>
      </GlobalFeedbackProvider>
    );
  }
  render(<App />);
  const drawer = await screen.findByRole("dialog", { name: "Choices" });
  await waitFor(() =>
    expect(
      within(drawer).getByRole("button", { name: "Retry connection" }),
    ).toBeVisible(),
  );
  fireEvent.click(within(drawer).getByRole("button", { name: "Close" }));
  await waitFor(() =>
    expect(
      within(screen.getByRole("dialog", { name: "Editor" })).getByRole("button", {
        name: "Retry connection",
      }),
    ).toBeVisible(),
  );
  expect(screen.getAllByRole("button", { name: "Retry connection" })).toHaveLength(1);
});
