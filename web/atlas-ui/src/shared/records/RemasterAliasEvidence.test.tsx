import { render, screen } from "@testing-library/react";
import { RemasterAliasEvidence } from "./RemasterAliasEvidence";
import { remasterInformationFixture } from "../../test/remasterFixtures";

it("keeps supplied lookup reasons and human class scopes without an expanded source dump", () => {
  const citation = remasterInformationFixture().declarations[0].citations[0];
  render(
    <RemasterAliasEvidence
      matches={[
        {
          matched_text: "Stance Savant",
          lookup_context: {
            label: "Former name",
            supports: [
              {
                declaration_id: "fighter",
                kind_labels: ["Renamed"],
                context: ["Class features", "Fighter"],
                citations: [{ ...citation, revision: "internal-hash" }],
                citations_truncated: false,
              },
              {
                declaration_id: "ranger",
                kind_labels: ["Renamed"],
                context: ["Class features", "Ranger"],
                citations: [citation],
                citations_truncated: false,
              },
            ],
            truncated: false,
          },
        },
      ]}
    />,
  );
  expect(screen.getByText(/Former name: “Stance Savant”/)).toBeVisible();
  expect(
    screen.getByText("· Class features · Fighter; Class features · Ranger"),
  ).toBeVisible();
  expect(screen.queryByText(/internal-hash|Foundry|Atlas|Revision:/)).toBeNull();
  expect(document.querySelector("details")).toBeNull();
});
