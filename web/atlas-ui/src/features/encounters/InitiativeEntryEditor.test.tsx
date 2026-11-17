import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useRef, useState } from "react";
import { InitiativeEntryEditor } from "./InitiativeEntryEditor";

it.each([false, true])(
  "closes a saved custom turn after pending clears when writes remain disabled: %s",
  async (writesDisabled) => {
    let finishSave!: (value: boolean) => void;
    const save = new Promise<boolean>((resolve) => {
      finishSave = resolve;
    });
    const onSave = vi.fn(() => save);
    function Editor({ pending, disabled }: { pending: boolean; disabled: boolean }) {
      const opener = useRef<HTMLButtonElement>(null);
      const pane = useRef<HTMLElement>(null);
      const [open, setOpen] = useState(true);
      return (
        <section ref={pane} tabIndex={-1} aria-label="Custom turn pane">
          <button ref={opener} disabled={disabled}>
            Edit custom turn
          </button>
          {open && (
            <InitiativeEntryEditor
              initial={{ display_name: "Lair action", initiative: 20 }}
              disabled={disabled}
              pending={pending}
              onSave={onSave}
              onClose={() => setOpen(false)}
              getDismissFocus={() => ({ opener: opener.current, pane: pane.current })}
            />
          )}
        </section>
      );
    }
    const { rerender } = render(<Editor pending={false} disabled={false} />);
    const opener = screen.getByRole("button", { name: "Edit custom turn" });
    const pane = screen.getByRole("region", { name: "Custom turn pane" });
    const box = new DOMRect(0, 0, 100, 30);
    const rects = Object.assign([box], {
      item: (index: number) => (index === 0 ? box : null),
    });
    vi.spyOn(opener, "getClientRects").mockReturnValue(rects);
    vi.spyOn(pane, "getClientRects").mockReturnValue(rects);
    const submit = screen.getByRole("button", { name: "Save" });
    submit.focus();
    fireEvent.click(submit);
    await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
    rerender(<Editor pending disabled />);
    await act(async () => finishSave(true));
    expect(screen.getByRole("dialog", { name: "Edit custom turn" })).toBeVisible();
    expect(opener).toBeDisabled();
    rerender(<Editor pending={false} disabled={writesDisabled} />);
    await waitFor(() => expect(writesDisabled ? pane : opener).toHaveFocus());
    expect(screen.queryByRole("dialog", { name: "Edit custom turn" })).toBeNull();
  },
);
