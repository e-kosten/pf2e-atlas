import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { ConfigProvider, theme } from "antd";
import { DangerActionButton } from "./DangerActionButton";

describe("DangerActionButton", () => {
  it("creates destructive confirmation inside the configured app theme context", async () => {
    const onConfirm = vi.fn();
    render(
      <ConfigProvider prefixCls="atlas-test" theme={{ algorithm: theme.darkAlgorithm }}>
        <DangerActionButton
          confirmContent="This action cannot be undone."
          confirmOkText="Delete"
          confirmTitle="Delete encounter?"
          onConfirm={onConfirm}
        >
          Delete encounter
        </DangerActionButton>
      </ConfigProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "Delete encounter" }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveClass("atlas-test-modal");
    expect(within(dialog).queryByRole("button", { name: "Close" })).toBeNull();
    const confirm = within(dialog).getByRole("button", { name: "Delete" });
    expect(confirm).toHaveClass("atlas-test-btn");
    fireEvent.click(confirm);

    await waitFor(() => expect(onConfirm).toHaveBeenCalledTimes(1));
  });
});
