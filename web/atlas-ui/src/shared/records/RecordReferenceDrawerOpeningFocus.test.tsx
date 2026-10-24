import { act, render, screen } from "@testing-library/react";
import { Button, type DrawerProps } from "antd";
import { detailFixture } from "../../test/fixtures";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";

const drawer = vi.hoisted(() => ({
  afterOpenChange: undefined as DrawerProps["afterOpenChange"],
}));

vi.mock("antd", async (importOriginal) => {
  const actual = await importOriginal<typeof import("antd")>();
  return {
    ...actual,
    Drawer: (props: DrawerProps) => {
      drawer.afterOpenChange = props.afterOpenChange;
      return (
        <div role="dialog" aria-label="Reference preview" tabIndex={-1}>
          {props.title}
          {props.children}
        </div>
      );
    },
  };
});

function preview() {
  return render(
    <>
      <RecordReferenceDrawer
        anchor={null}
        detail={detailFixture("spells:fear", "Fear")}
        actions={<Button>Add to saved list</Button>}
        loading={false}
        onClose={vi.fn()}
        onOpenFullPage={vi.fn()}
        onReference={vi.fn()}
      />
      <button>Another action</button>
    </>,
  );
}

it.each(["body", "dialog"])(
  "focuses the heading when opening completes with focus on the %s",
  (initial) => {
    preview();
    if (initial === "dialog") screen.getByRole("dialog").focus();
    act(() => drawer.afterOpenChange?.(true));
    expect(screen.getByRole("heading", { name: "Fear" })).toHaveFocus();
  },
);

it.each(["Add to saved list", "Another action"])(
  "preserves %s focus when opening completes after an interaction",
  (name) => {
    preview();
    const destination = screen.getByRole("button", { name });
    destination.focus();
    act(() => drawer.afterOpenChange?.(true));
    expect(destination).toHaveFocus();
  },
);
