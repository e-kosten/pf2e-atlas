import { fireEvent, render, screen } from "@testing-library/react";
import { useRef, useState } from "react";
import { EncounterModal, EncounterModalProvider } from "./EncounterModal";
function Form() {
  const area = useRef<HTMLDivElement>(null);
  const [draft, setDraft] = useState("");
  return (
    <EncounterModalProvider area={area}>
      <div ref={area}>
        <main className="workspace-grid" />
      </div>
      <EncounterModal
        title="Encounter form"
        open
        wrapClassName="test-encounter-wrapper"
        classNames={{ body: "test-encounter-body" }}
      >
        <input
          aria-label="Draft"
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
        />
      </EncounterModal>
    </EncounterModalProvider>
  );
}
it("anchors within visible work area without moving, losing focus, or resetting scroll when typing", () => {
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
    this: HTMLElement,
  ) {
    return this.classList.contains("workspace-grid")
      ? {
          top: 100,
          left: 40,
          right: 1040,
          bottom: 900,
          width: 1000,
          height: 800,
          x: 40,
          y: 100,
          toJSON() {
            return {};
          },
        }
      : new DOMRect();
  });
  render(<Form />);
  const wrapper = document.querySelector<HTMLElement>(".test-encounter-wrapper")!;
  expect(wrapper.style.paddingTop).toBe("153.44px");
  expect(wrapper.style.paddingLeft).toBe("56px");
  const before = wrapper.getAttribute("style");
  const body = document.querySelector<HTMLElement>(".test-encounter-body")!;
  body.scrollTop = 25;
  const input = screen.getByRole("textbox", { name: "Draft" });
  input.focus();
  fireEvent.change(input, { target: { value: "Typing" } });
  expect(wrapper.getAttribute("style")).toBe(before);
  expect(input).toHaveFocus();
  expect(body.scrollTop).toBe(25);
});
it("uses usable viewport space when the encounter workspace is completely outside it", () => {
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
    top: -900,
    left: 0,
    right: 1000,
    bottom: -100,
    width: 1000,
    height: 800,
    x: 0,
    y: -900,
    toJSON() {
      return {};
    },
  });
  render(<Form />);
  const wrapper = document.querySelector<HTMLElement>(".test-encounter-wrapper")!;
  expect(wrapper.style.paddingTop).toBe("61.44px");
  expect(wrapper.style.paddingBottom).toBe("16px");
});

it("lets a short participant panel use the remaining viewport for form fields", () => {
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
    top: 100,
    left: 0,
    right: 1000,
    bottom: 430,
    width: 1000,
    height: 330,
    x: 0,
    y: 100,
    toJSON() {
      return {};
    },
  });
  render(<Form />);
  const wrapper = document.querySelector<HTMLElement>(".test-encounter-wrapper")!;
  expect(wrapper.style.paddingTop).toBe("153.44px");
  expect(wrapper.style.paddingBottom).toBe("16px");
});
