import { App, Button, ConfigProvider, theme } from "antd";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { DangerActionButton } from "./DangerActionButton";
import { useConfirmDangerAction } from "./useConfirmDangerAction";

function CallbackConfirmation({ onConfirm }: { onConfirm: () => void }) {
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
