import { expect, type Locator } from "@playwright/test";

/** Check every peer in a row that should fit at this viewport. */
export async function expectControlGaps(controls: Locator, gap: number, count: number) {
  const visible = controls.filter({ visible: true });
  await expect(visible).toHaveCount(count);
  await expect
    .poll(() =>
      visible.evaluateAll((elements) => {
        const boxes = elements.map((element) => element.getBoundingClientRect());
        return boxes.slice(1).map((next, index) => {
          const previous = boxes[index];
          if (next.top >= previous.bottom || previous.top >= next.bottom)
            return "wrapped";
          return Math.round(next.left - previous.right);
        });
      }),
    )
    .toEqual(Array(count - 1).fill(gap));
}
