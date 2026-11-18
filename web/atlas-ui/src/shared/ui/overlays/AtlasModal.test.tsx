import { render, screen, waitFor } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import { DesktopTabContext } from "../../../app/DesktopTabContext";
import { AtlasModal } from "./AtlasModal";

it("suspends a tab modal without reporting a close, and completes an inactive logical close once", async () => {
  const panel = document.createElement("section");
  document.body.append(panel);
  const afterClose = vi.fn();
  const afterOpenChange = vi.fn();
  const tab = {
    id: "overlay-tab",
    activation: 0,
    panel: () => panel,
    title: () => {},
    theme: () => {},
    guard: () => () => {},
  };
  const tree = (active: boolean, open: boolean, showOpener = true) => (
    <DesktopTabContext.Provider value={{ ...tab, active }}>
      <button hidden={!showOpener}>Open draft</button>
      <main id="atlas-main" tabIndex={-1}>
        Application
      </main>
      <AtlasModal
        open={open}
        title="Edit draft"
        destroyOnHidden
        afterClose={afterClose}
        afterOpenChange={afterOpenChange}
      >
        <textarea aria-label="Draft" defaultValue="Original draft" />
      </AtlasModal>
    </DesktopTabContext.Provider>
  );
  const view = render(tree(true, false), { container: panel });
  try {
    const opener = screen.getByRole("button", { name: "Open draft" });
    opener.focus();
    view.rerender(tree(true, true));
    const textarea = await screen.findByRole<HTMLTextAreaElement>("textbox", {
      name: "Draft",
    });
    textarea.value = "Retained draft";
    await waitFor(() => expect(afterOpenChange.mock.calls).toEqual([[true]]));

    view.rerender(tree(false, true));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(afterClose).not.toHaveBeenCalled();
    expect(afterOpenChange.mock.calls).toEqual([[true]]);

    view.rerender(tree(true, true));
    expect(await screen.findByRole("textbox", { name: "Draft" })).toBe(textarea);
    expect(textarea).toHaveValue("Retained draft");
    view.rerender(tree(false, true));
    view.rerender(tree(true, true));
    view.rerender(tree(false, true));
    view.rerender(tree(false, false));
    await waitFor(() => expect(afterClose).toHaveBeenCalledOnce());
    expect(afterOpenChange.mock.calls).toEqual([[true], [false]]);

    view.rerender(tree(true, false));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(afterClose).toHaveBeenCalledOnce();

    opener.focus();
    view.rerender(tree(true, true));
    const freshDraft = await screen.findByRole("textbox", { name: "Draft" });
    await waitFor(() =>
      expect(afterOpenChange.mock.calls).toEqual([[true], [false], [true]]),
    );
    view.rerender(tree(false, true));
    view.rerender(tree(true, true, false));
    freshDraft.focus();
    view.rerender(tree(true, false, false));
    await waitFor(() => expect(screen.getByRole("main")).toHaveFocus());
    expect(afterClose).toHaveBeenCalledTimes(2);
    expect(afterOpenChange.mock.calls).toEqual([[true], [false], [true], [false]]);
  } finally {
    view.unmount();
    panel.remove();
  }
});
