import { App, Button, ConfigProvider, theme } from "antd";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { DangerActionButton } from "./DangerActionButton";
import { useConfirmDangerAction } from "./useConfirmDangerAction";

function CallbackConfirmation({
  onConfirm,
}: {
  onConfirm: () => void | Promise<unknown>;
}) {
  const { confirm, dialog } = useConfirmDangerAction();
  return (
    <>
      <Button
        onClick={() =>
          confirm({ title: "Remove participant?", okText: "Remove", onConfirm })
        }
      >
        Remove participant
      </Button>
      {dialog}
    </>
  );
}

it("cancels an idle destructive confirmation from its backdrop and returns to the actual trigger", async () => {
  const confirm = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <App>
        <DangerActionButton
          confirmTitle="Delete list?"
          confirmOkText="Delete"
          onConfirm={confirm}
        >
          Delete list
        </DangerActionButton>
      </App>
    </ConfigProvider>,
  );
  const trigger = screen.getByRole("button", { name: "Delete list" });
  fireEvent.click(trigger);
  await screen.findByRole("dialog");
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus(),
  );
  fireEvent.mouseDown(document.querySelector(".ant-modal-wrap")!);
  fireEvent.click(document.querySelector(".ant-modal-wrap")!);
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(confirm).not.toHaveBeenCalled();
  expect(trigger).toHaveFocus();
});

it("blocks destructive backdrop dismissal while pending and puts its retryable failure inside the dialog", async () => {
  let reject!: (error: Error) => void;
  const confirm = vi
    .fn()
    .mockReturnValueOnce(
      new Promise((_yes, no) => {
        reject = no;
      }),
    )
    .mockResolvedValueOnce(undefined);
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <App>
        <DangerActionButton
          confirmTitle="Delete list?"
          confirmContent="Its saved entries will also be removed."
          confirmOkText="Delete"
          onConfirm={confirm}
        >
          Delete list
        </DangerActionButton>
      </App>
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Delete list" }));
  const dialog = await screen.findByRole("dialog");
  await waitFor(() => expect(dialog).toBeVisible());
  fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
  await waitFor(() => expect(confirm).toHaveBeenCalledOnce());
  fireEvent.mouseDown(document.querySelector(".ant-modal-wrap")!);
  fireEvent.click(document.querySelector(".ant-modal-wrap")!);
  expect(dialog).toBeVisible();
  expect(within(dialog).getByRole("button", { name: "Cancel" })).toBeDisabled();
  await act(async () => reject(new Error("Service unavailable")));
  expect(await within(dialog).findByRole("alert")).toHaveTextContent(
    "Delete could not be confirmed",
  );
  expect(
    within(dialog).getByText("Its saved entries will also be removed."),
  ).toBeVisible();
  expect(within(dialog).getByRole("button", { name: "Cancel" })).toBeEnabled();
  fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(confirm).toHaveBeenCalledTimes(2);
});

describe("contextual danger confirmations", () => {
  it.each(["button", "callback"])(
    "keeps delayed and rejected %s confirmation open with visible failure",
    async (caller) => {
      let reject!: (error: Error) => void;
      const pending = new Promise<void>((_resolve, no) => {
        reject = no;
      });
      const confirm = vi.fn().mockReturnValueOnce(pending).mockResolvedValue(undefined);
      render(
        <ConfigProvider theme={{ token: { motion: false } }}>
          <App component={false}>
            {caller === "button" ? (
              <DangerActionButton
                confirmTitle="Remove participant?"
                confirmOkText="Remove"
                onConfirm={confirm}
              >
                Remove participant
              </DangerActionButton>
            ) : (
              <CallbackConfirmation onConfirm={confirm} />
            )}
          </App>
        </ConfigProvider>,
      );
      fireEvent.click(screen.getByRole("button", { name: "Remove participant" }));
      const dialog = await screen.findByRole("dialog");
      fireEvent.click(screen.getByRole("button", { name: /^Remove$/ }));
      await waitFor(() => expect(confirm).toHaveBeenCalledOnce());
      expect(dialog).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
      fireEvent.keyDown(dialog, { key: "Escape", keyCode: 27 });
      expect(dialog).toBeInTheDocument();
      await act(async () => reject(new Error("Delete unavailable")));
      expect(
        await screen.findByText("Remove could not be confirmed"),
      ).toBeInTheDocument();
      expect(dialog).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Cancel" })).not.toBeDisabled();
      fireEvent.click(screen.getByRole("button", { name: /^Remove$/ }));
      await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
      expect(confirm).toHaveBeenCalledTimes(2);
    },
  );
  it.each(["button", "callback"])(
    "preserves theme, cancellation and exact confirmation for the %s caller",
    async (caller) => {
      const confirm = vi.fn();
      render(
        <ConfigProvider
          prefixCls="themed"
          theme={{ algorithm: theme.darkAlgorithm, token: { motion: false } }}
        >
          <App component={false}>
            {caller === "button" ? (
              <DangerActionButton
                confirmTitle="Remove participant?"
                confirmOkText="Remove"
                onConfirm={confirm}
              >
                Remove participant
              </DangerActionButton>
            ) : (
              <CallbackConfirmation onConfirm={confirm} />
            )}
          </App>
        </ConfigProvider>,
      );
      const trigger = screen.getByRole("button", { name: "Remove participant" });
      trigger.focus();
      fireEvent.click(trigger);
      expect(await screen.findByRole("dialog")).toHaveClass("themed-modal");
      fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
      await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
      expect(confirm).not.toHaveBeenCalled();
      expect(trigger).toHaveFocus();
      fireEvent.click(trigger);
      fireEvent.click(await screen.findByRole("button", { name: /^Remove$/ }));
      await waitFor(() => expect(confirm).toHaveBeenCalledTimes(1));
    },
  );
});

it("keeps the removal popover open through pending and failed writes, then permits retry", async () => {
  let reject!: (error: Error) => void;
  const confirm = vi
    .fn()
    .mockReturnValueOnce(
      new Promise<void>((_yes, no) => {
        reject = no;
      }),
    )
    .mockResolvedValue(undefined);
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <App>
        <DangerActionButton
          confirmation="popover"
          confirmTitle="Remove from list?"
          confirmOkText="Remove"
          onConfirm={confirm}
        >
          Remove entry
        </DangerActionButton>
      </App>
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Remove entry" }));
  fireEvent.click(await screen.findByRole("button", { name: "Remove" }));
  expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  await waitFor(() => expect(screen.getByText("Remove from list?")).toBeVisible());
  await act(async () => reject(new Error("Cannot remove")));
  expect(await screen.findByText("Unable to remove. Try again.")).toBeVisible();
  await waitFor(() => expect(screen.getByText("Remove from list?")).toBeVisible());
  fireEvent.click(screen.getByRole("button", { name: "Remove" }));
  await waitFor(() =>
    expect(screen.queryByText("Remove from list?")).not.toBeVisible(),
  );
  expect(confirm).toHaveBeenCalledTimes(2);
});

it("retains synchronous removal failures for explicit retry", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <App>
        <DangerActionButton
          confirmation="popover"
          confirmTitle="Remove from list?"
          confirmOkText="Remove"
          onConfirm={() => {
            throw new Error("Synchronous failure");
          }}
        >
          Remove entry
        </DangerActionButton>
      </App>
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Remove entry" }));
  fireEvent.click(await screen.findByRole("button", { name: "Remove" }));
  await waitFor(() =>
    expect(screen.getByText("Unable to remove. Try again.")).toBeVisible(),
  );
  expect(screen.getByText("Remove from list?")).toBeVisible();
});
