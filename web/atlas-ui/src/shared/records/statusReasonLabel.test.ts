import { statusReasonLabel } from "./statusReasonLabel";

it("formats capability reasons as concise labels without changing internal punctuation", () => {
  expect(statusReasonLabel("No", "Unconscious.")).toBe("No · Unconscious");
  expect(statusReasonLabel("No", "Cannot act; recovery is pending.")).toBe(
    "No · Cannot act; recovery is pending",
  );
  expect(statusReasonLabel("No", null)).toBe("No");
  expect(statusReasonLabel("No", "   ")).toBe("No");
});
