import { act, render, screen } from "@testing-library/react";
import { LoadingStatus } from "./LoadingStatus";

afterEach(() => vi.useRealTimers());

it("delays both the spinner and loading text without changing the reserved slot", () => {
  vi.useFakeTimers();
  const view = render(<LoadingStatus label="Searching…" />);
  const label = screen.getByText("Searching…");
  const slot = label.parentElement;
  expect(label).not.toBeVisible();
  expect(screen.queryByRole("status")).not.toBeInTheDocument();
  act(() => vi.advanceTimersByTime(499));
  expect(label).not.toBeVisible();
  act(() => vi.advanceTimersByTime(1));
  expect(screen.getByRole("status", { name: "Searching…" })).toBe(slot);
  expect(label).toBeVisible();
  view.unmount();
});

it("does not carry a previous operation's delay into a new request", () => {
  vi.useFakeTimers();
  const view = render(<LoadingStatus label="First request" />);
  act(() => vi.advanceTimersByTime(400));
  view.rerender(<></>);
  act(() => vi.advanceTimersByTime(200));
  view.rerender(<LoadingStatus label="Second request" />);
  expect(screen.getByText("Second request")).not.toBeVisible();
  act(() => vi.advanceTimersByTime(499));
  expect(screen.queryByRole("status")).not.toBeInTheDocument();
  view.unmount();
});
