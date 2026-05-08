import type { DerivedTagTerminalPointerEvent, DerivedTagTerminalPointerRect, DerivedTagTerminalPointerRegion } from "./types.js";

const ESCAPE_CHARACTER = String.fromCharCode(27);
const SGR_MOUSE_PATTERN = /^\[<(\d+);(\d+);(\d+)([mM])$/;

export function parseDerivedTagTerminalPointerEvent(input: string): DerivedTagTerminalPointerEvent | undefined {
  const mouseInput = input.startsWith(ESCAPE_CHARACTER) ? input.slice(1) : input;
  const match = SGR_MOUSE_PATTERN.exec(mouseInput);
  if (!match) {
    return undefined;
  }

  const [, rawCode, rawX, rawY] = match;
  const code = Number(rawCode);
  const x = Number(rawX) - 1;
  const y = Number(rawY) - 1;

  if (!Number.isFinite(code) || !Number.isFinite(x) || !Number.isFinite(y) || x < 0 || y < 0) {
    return undefined;
  }

  if (code === 64) {
    return { kind: "wheel", x, y, deltaY: -1 };
  }
  if (code === 65) {
    return { kind: "wheel", x, y, deltaY: 1 };
  }
  if (code === 0 && match[4] === "M") {
    return { kind: "click", x, y, button: "left" };
  }
  if (code === 32 && match[4] === "M") {
    return { kind: "drag", x, y, button: "left" };
  }
  if (code === 0 && match[4] === "m") {
    return { kind: "release", x, y, button: "left" };
  }

  return undefined;
}

export function isPointInsideDerivedTagTerminalPointerRect(
  rect: DerivedTagTerminalPointerRect,
  point: Pick<DerivedTagTerminalPointerEvent, "x" | "y">,
): boolean {
  return (
    point.x >= rect.x &&
    point.y >= rect.y &&
    point.x < rect.x + rect.width &&
    point.y < rect.y + rect.height
  );
}

export function dispatchDerivedTagTerminalPointerEvent(
  regions: readonly (DerivedTagTerminalPointerRegion & { order: number })[],
  event: DerivedTagTerminalPointerEvent,
): boolean {
  const sortedRegions = [...regions].sort(
    (left, right) => (right.priority ?? 0) - (left.priority ?? 0) || right.order - left.order,
  );

  for (const region of sortedRegions) {
    if (!isPointInsideDerivedTagTerminalPointerRect(region.rect, event)) {
      continue;
    }
    if (region.onPointerEvent(event) !== false) {
      return true;
    }
  }

  return false;
}
