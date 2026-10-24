import { render, screen } from "@testing-library/react";
import { HazardReference } from "./HazardReference";
import {
  actorFixture,
  activityFixture,
  knownFact,
  numberFact,
} from "../../test/fixtures";
import type { ActorActivityView } from "../../generated/atlas";

it.each([false, true])(
  "keeps hazard activation before routine and additional actions, retaining early description and defenses (encounter=%s)",
  (encounter) => {
    const actor = actorFixture();
    const activities: ActorActivityView[] = [
      {
        ...activityFixture("Echoes of Defeat"),
        kind: "ability",
        ability_group: "activation",
        action_cost: knownFact({ label: "Free action", glyph: "free_action" }),
      },
      {
        ...activityFixture("Pitfall"),
        kind: "ability",
        ability_group: "activation",
        action_cost: knownFact({ label: "Reaction", glyph: "reaction" }),
      },
      activityFixture("Javelin"),
      {
        ...activityFixture("Additional action"),
        kind: "ability",
        ability_group: "primary",
      },
    ];
    actor.activities = knownFact(
      activities.map((a, index) => ({
        ...a,
        reading_order: index,
        navigation: {
          ...a.navigation,
          embedded_path: [
            { collection: "/items", identity: { Stable: `owned-${index}` } },
          ],
        },
      })),
    );
    render(
      <HazardReference
        hazard={{
          actor,
          complex: knownFact(true),
          stealth: numberFact(12),
          hardness: numberFact(5),
          defense_block: knownFact(true),
          show_saves: true,
        }}
        encounter={encounter}
        encounterContent={
          encounter
            ? {
                participantKey: "hazard",
                durability: <h3>Live HP and defenses</h3>,
                conditions: <h3>Current conditions</h3>,
              }
            : undefined
        }
        content={{
          description: <p>Complete hazard description and operating rules.</p>,
          detection: <p>Detection qualifications</p>,
          disable: <h3>Disable</h3>,
          routine: <h3>Routine</h3>,
          reset: <h3>Reset</h3>,
          other: null,
        }}
        onReference={vi.fn()}
      />,
    );
    const headings = screen.getAllByRole("heading").map((h) => h.textContent);
    const ordered = [
      "Detection",
      encounter ? "Live HP and defenses" : "Defenses",
      "Disable",
      ...(encounter ? ["Current conditions"] : []),
      "Initial activation",
      "Routine",
      "Strikes",
      "Special abilities",
      "Reset",
    ];
    for (let i = 1; i < ordered.length; i++)
      expect(headings.indexOf(ordered[i - 1])).toBeLessThan(
        headings.indexOf(ordered[i]),
      );
    expect(screen.getByText("Echoes of Defeat")).toBeVisible();
    expect(screen.getByText("Pitfall")).toBeVisible();
    expect(
      screen
        .getByText("Complete hazard description and operating rules.")
        .compareDocumentPosition(screen.getByRole("heading", { name: "Detection" })) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  },
);
