import "@testing-library/jest-dom/vitest";
import { act, cleanup } from "@testing-library/react";
import type {} from "vitest/jsdom";

const browserWindow = jsdom.window;
for (const name of ["localStorage", "sessionStorage"] as const) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    get: () => browserWindow[name],
  });
}

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// jsdom has no layout engine. Real resize/position behavior is covered by browser checks.
class TestResizeObserver implements ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
vi.stubGlobal("ResizeObserver", TestResizeObserver);

// Ant's NODE_ENV=test constant IDs merge distinct portals in its Escape stack.
// Exercise normal React IDs for cross-overlay and accessible-name behavior.
vi.stubEnv("NODE_ENV", "development");

afterAll(async () => {
  cleanup();
  vi.useRealTimers();
  // Ant Form debounces empty feedback for 10ms. Let queued feedback finish after
  // unmount while jsdom still exists, before Vitest disposes this file's window.
  await act(async () => {
    await new Promise<void>((resolve) => setTimeout(resolve, 20));
  });
});

Element.prototype.getAnimations = () => [];
