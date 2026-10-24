import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { Button, ConfigProvider } from "antd";
import { detailFixture } from "../../test/fixtures";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";

it("focuses the reference heading on open, preserves action focus during updates and restores the opener", async () => {
  const opener = document.createElement("button");
  opener.textContent = "Read reference";
  document.body.append(opener);
  opener.focus();
  const onClose = vi.fn();
  const props = {
    anchor: opener,
    detail: detailFixture("spells:fear", "Fear"),
    actions: <Button>Add to saved list</Button>,
    onClose,
    onOpenFullPage: vi.fn(),
    onReference: vi.fn(),
  };
  const { rerender, unmount } = render(
    <RecordReferenceDrawer {...props} loading={false} />,
    {
      wrapper: ({ children }) => (
        <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
      ),
    },
  );
  try {
    const heading = screen.getByRole("heading", { name: "Fear" });
    const action = screen.getByRole("button", { name: "Add to saved list" });
    await waitFor(() => expect(heading).toHaveFocus());
    expect(action).not.toHaveFocus();

    action.focus();
    rerender(<RecordReferenceDrawer {...props} loading />);
    expect(screen.getByRole("heading", { name: "Fear" })).toBe(heading);
    expect(action).toHaveFocus();
    rerender(<RecordReferenceDrawer {...props} loading={false} />);
    expect(action).toHaveFocus();

    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(opener).toHaveFocus();
  } finally {
    unmount();
    opener.remove();
  }
});
