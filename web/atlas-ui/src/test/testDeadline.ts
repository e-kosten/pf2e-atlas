declare const __ATLAS_TEST_DEADLINE_SCALE__: number;

export function testDeadline(milliseconds: number): number {
  return milliseconds * __ATLAS_TEST_DEADLINE_SCALE__;
}
