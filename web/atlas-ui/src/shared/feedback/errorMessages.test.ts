import { AtlasApiError } from "../../api/atlasApi";
import { AtlasConnectionError } from "../../api/availability";
import { AtlasCompatibilityError } from "../../api/compatibility";
import { errorExplanation, userErrorMessage } from "./errorMessages";

it.each([400, 500, 200])(
  "uses contextual safe copy for unknown HTTP %s failures",
  (status) => {
    const error = new AtlasApiError(status, "<html>proxy /private/secret SQL</html>");
    expect(userErrorMessage(error, "Could not load this record.")).toBe(
      "Could not load this record.",
    );
    expect(error.message).toContain("proxy /private/secret SQL");
  },
);

it("explains expected conflicts without leaking backend keys or revisions", () => {
  const error = new AtlasApiError(409, "actor:abc revision 17 != 16", {
    code: "encounter_participant_conflict",
    message: "actor:abc revision 17 != 16",
  });
  expect(errorExplanation(error)).toBe(
    "This participant changed. Review the current values before trying again.",
  );
});

it("uses a safe fallback for malformed or unknown error codes", () => {
  expect(
    errorExplanation({ code: "constructor", message: "private internals" }),
  ).toBeUndefined();
  expect(errorExplanation({ code: { detail: "private internals" } })).toBeUndefined();
});

it("does not describe an unconfirmed write as unsaved", () => {
  const cause = new TypeError("fetch interrupted");
  const error = new AtlasConnectionError(true, cause);
  expect(userErrorMessage(error)).toContain("could not be confirmed");
  expect(userErrorMessage(error)).toContain("Check the current values");
  expect(error.cause).toBe(cause);
  expect(userErrorMessage(new AtlasCompatibilityError("old"))).toBe("");
});
