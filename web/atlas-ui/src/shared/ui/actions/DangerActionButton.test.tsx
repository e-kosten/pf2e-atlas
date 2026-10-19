import { App, Button, ConfigProvider, theme } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { DangerActionButton } from "./DangerActionButton";
import { useConfirmDangerAction } from "./useConfirmDangerAction";

function CallbackConfirmation({
  onConfirm,
}: {
  onConfirm: () => void | Promise<unknown>;
}) {
  const confirm = useConfirmDangerAction();
  return (
    <Button
      onClick={() =>
        confirm({ title: "Remove participant?", okText: "Remove", onConfirm })
      }
    >
      Remove participant
    </Button>
  );
}

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
      expect(await screen.findByText("Delete unavailable")).toBeInTheDocument();
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
