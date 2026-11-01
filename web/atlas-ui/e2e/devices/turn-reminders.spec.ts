import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";

test("allowing a restricted reaction retains keyboard focus without spending it", async ({
  page,
}) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Restricted reaction ${Date.now()}` },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  const resourceActions: string[] = [];
  page.on("request", (request) => {
    if (request.method() === "POST" && request.url().endsWith("/turn-resources")) {
      resourceActions.push(request.postDataJSON().action.kind);
    }
  });
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name: "Restricted reaction hero",
            initiative: 20,
            max_hp: 20,
            current_hp: 20,
          },
        })
      ).ok(),
    ).toBeTruthy();
    const detail = await (await page.request.get(path)).json();
    const actor = detail.participants[0];
    const catalog = await (
      await page.request.get(
        `${path}/participants/${actor.participant_key}/condition-definitions`,
      )
    ).json();
    const stunned = catalog.conditions.find(
      (condition: { name: string }) => condition.name === "Stunned",
    );
    const added = await page.request.post(
      `${path}/participants/${actor.participant_key}/conditions`,
      {
        data: {
          request_id: `stunned-${Date.now()}`,
          expected_encounter_revision: detail.revision,
          participant_key: actor.participant_key,
          condition_ref: stunned.condition_ref,
          event_intent: "apply",
          decisions: { keep_upright: false },
          scope: { kind: "everyone" },
          extra_action: { kind: "unspecified" },
          immunity_override: false,
          parameters: { kind: "none" },
          value: 1,
        },
      },
    );
    expect(added.ok(), await added.text()).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await page
      .getByRole("button", { name: "Restricted reaction hero", exact: true })
      .click();
    const allow = page.getByRole("button", { name: "Allow reaction", exact: true });
    await allow.focus();
    await page.keyboard.press("Enter");
    const reaction = page.getByRole("button", { name: "Spend reaction", exact: true });
    await expect(reaction).toHaveAttribute("aria-disabled", "true");
    await page.evaluate(
      () =>
        new Promise<void>((resolve) => {
          requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
        }),
    );
    await expect(reaction).toBeFocused();
    await expect(
      page.getByText("Cannot react · Stunned", { exact: true }),
    ).toBeVisible();
    await page.keyboard.press("Enter");
    await page.keyboard.press("Space");
    await expect(reaction).toBeFocused();
    expect(resourceActions).toEqual(["grant_reaction"]);
    const saved = await (await page.request.get(path)).json();
    expect(saved.participants[0].turn_resources.reactions_remaining).toBe(1);
    await expect(
      page.getByRole("button", { name: "Restore reaction", exact: true }),
    ).toHaveCount(0);

    await page
      .getByRole("button", { name: "Remove Stunned", exact: true })
      .last()
      .click();
    await expect(reaction).toHaveAttribute("aria-disabled", "false");
    await reaction.focus();
    await page.keyboard.press("Enter");
    await expect(
      page.getByRole("button", { name: "Restore reaction", exact: true }),
    ).toBeFocused();
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

for (const outcome of [
  "table",
  "roll",
  "automatic success",
  "automatic failure",
] as const) {
  test(`Next turn presents one notification and ${outcome} recovery uses the current turn`, async ({
    page,
  }, info) => {
    const created = await page.request.post("/api/encounters", {
      data: { name: `Turn reminders ${Date.now()}` },
    });
    const { encounter } = await created.json();
    const path = `/api/encounters/${encounter.encounter_key}`;
    try {
      for (const [display_name, initiative] of [
        ["Outgoing hero", 20],
        ["Incoming hero", 10],
        ["No initiative", null],
      ] as const) {
        expect(
          (
            await page.request.post(`${path}/participants/manual`, {
              data: {
                encounter_ref: encounter.encounter_key,
                display_name,
                ...(initiative === null ? {} : { initiative }),
                max_hp: 30,
                current_hp: 30,
              },
            })
          ).ok(),
        ).toBeTruthy();
      }
      let detail = await (await page.request.get(path)).json();
      const actor = detail.participants.find(
        (p: { display_name: string }) => p.display_name === "Outgoing hero",
      );
      const incoming = detail.participants.find(
        (p: { display_name: string }) => p.display_name === "Incoming hero",
      );
      const catalog = await (
        await page.request.get(
          `${path}/participants/${actor.participant_key}/condition-definitions`,
        )
      ).json();
      const definition = catalog.conditions.find(
        (c: { editor_kind: string }) => c.editor_kind === "persistent_damage",
      );
      expect(
        (
          await page.request.post(`${path}/turn`, {
            data: {
              encounter_ref: encounter.encounter_key,
              participant_key: actor.participant_key,
            },
          })
        ).ok(),
      ).toBeTruthy();
      detail = await (await page.request.get(path)).json();
      const added = await page.request.post(
        `${path}/participants/${actor.participant_key}/conditions`,
        {
          data: {
            request_id: `persistent-${Date.now()}`,
            expected_encounter_revision: detail.revision,
            event_intent: "apply",
            decisions: { keep_upright: false },
            participant_key: actor.participant_key,
            condition_ref: definition.condition_ref,
            scope: { kind: "everyone" },
            extra_action: { kind: "unspecified" },
            immunity_override: false,
            parameters: {
              kind: "persistent_damage",
              formula: "1d6",
              damage_type: "fire",
              multiplier: "normal",
              recovery: { kind: "standard" },
              episode_id: null,
            },
          },
        },
      );
      expect(added.ok(), await added.text()).toBeTruthy();
      await page.goto(`/encounters/${encounter.slug}`);
      await page
        .getByRole("button", { name: "Next turn", exact: true })
        .filter({ visible: true })
        .click();
      const reminders = page.getByRole("dialog", {
        name: "Turn reminders",
        exact: true,
      });
      await expect(reminders).toBeVisible();
      detail = await (await page.request.get(path)).json();
      expect(detail.current_turn_participant_key).toBe(incoming.participant_key);
      const original = await (await page.request.get(`${path}/turn-review`)).json();
      let boundary = original.outgoing.boundary_key;
      const advancedRevision = detail.revision;
      await page.screenshot({
        path: info.outputPath("outgoing-reminders.png"),
        fullPage: true,
        animations: "disabled",
      });
      await reminders
        .getByRole("button", { name: "Close", exact: true })
        .last()
        .click();
      expect((await (await page.request.get(path)).json()).revision).toBe(
        advancedRevision,
      );
      await page.reload();
      await expect(reminders).not.toBeVisible();
      await expect(
        page.getByRole("button", { name: "Outgoing turn reminders", exact: true }),
      ).toHaveCount(0);
      await page
        .getByRole("button", { name: "Next turn", exact: true })
        .filter({ visible: true })
        .click();
      await expect
        .poll(
          async () =>
            (await (await page.request.get(path)).json()).current_turn_participant_key,
        )
        .toBe(actor.participant_key);
      if (info.project.name.startsWith("phone")) {
        await expect(
          page.getByRole("button", { name: "Back to roster", exact: true }),
        ).toBeVisible();
        await expect(
          page.getByRole("heading", { name: "Outgoing hero", exact: true }),
        ).toBeVisible();
      }
      await expect(reminders).not.toBeVisible();
      const turn = page.getByRole("region", { name: "Turn", exact: true });
      await expect(turn).toBeVisible();
      await expect(turn.getByRole("button", { name: /Actions:/ })).toHaveCount(0);
      await expect(turn.getByText("Actions", { exact: true })).toBeVisible();
      await expect(turn.getByRole("button", { name: /^Spend action / })).toHaveCount(3);
      await page
        .getByRole("button", { name: "Next turn", exact: true })
        .filter({ visible: true })
        .click();
      await expect(reminders).toBeVisible();
      const currentReview = await (
        await page.request.get(`${path}/turn-review`)
      ).json();
      expect(currentReview.outgoing.round_number).toBe(2);
      expect(currentReview.outgoing.boundary_key).not.toBe(boundary);
      boundary = currentReview.outgoing.boundary_key;
      await expect(
        reminders.getByRole("combobox", { name: "Outgoing turn", exact: true }),
      ).toHaveCount(0);
      await expect(
        reminders.getByRole("button", { name: "Handled at table", exact: true }),
      ).toHaveCount(0);
      if (outcome !== "table") {
        await reminders.getByText("Table results", { exact: true }).click();
        await reminders
          .getByRole("spinbutton", { name: "Fire damage amount", exact: true })
          .fill("2");
        await reminders.getByRole("button", { name: /^Apply \d+ damage$/ }).click();
        await expect(
          reminders.getByText("Total: 2 damage applied", { exact: true }),
        ).toBeVisible();
        const afterDamage = await (await page.request.get(path)).json();
        expect(
          afterDamage.participants.find(
            (participant: { participant_key: string }) =>
              participant.participant_key === actor.participant_key,
          ).current_hp,
        ).toBe(28);
        await reminders
          .getByRole("button", { name: "Roll Fire recovery", exact: true })
          .click();
        await expect(
          page.getByRole("spinbutton", { name: "Recovery DC", exact: true }),
        ).toHaveValue("15");
        if (outcome !== "roll")
          await page
            .getByRole("spinbutton", { name: "Recovery DC", exact: true })
            .fill(outcome === "automatic success" ? "0" : "100");
        const anchor = reminders.getByRole("button", {
          name: "Roll Fire recovery",
          exact: true,
        });
        const retainedAnchor = await anchor.elementHandle();
        const before = await anchor.boundingBox();
        const completed = page.waitForResponse(
          (response) =>
            response.url().endsWith("/persistent-recovery") &&
            response.request().method() === "POST",
        );
        const roll = page.getByRole("button", { name: "Roll", exact: true });
        await roll.focus();
        await roll.press("Enter");
        const response = await completed;
        expect(response.ok()).toBeTruthy();
        const receipt = await response.json();
        if (outcome === "roll") {
          expect(receipt.roll).not.toBeNull();
          await expect(page.locator(".atlas-roll").last()).toBeVisible();
        } else {
          expect(receipt.roll).toBeNull();
          expect(receipt.success).toBe(outcome === "automatic success");
        }
        await expect(
          page.getByText(
            `${receipt.success ? "Recovered" : "Recovery failed"} · DC ${receipt.dc}`,
            { exact: true },
          ),
        ).toBeVisible();
        await expect(reminders).toBeVisible();
        expect(await retainedAnchor!.evaluate((node) => node.isConnected)).toBe(true);
        await expect(
          reminders.getByRole("button", { name: "View Fire recovery", exact: true }),
        ).toBeFocused();
        const after = await retainedAnchor!.boundingBox();
        expect(before).not.toBeNull();
        expect(after).not.toBeNull();
        expect(Math.abs(after!.x - before!.x)).toBeLessThanOrEqual(1);
        expect(Math.abs(after!.y - before!.y)).toBeLessThanOrEqual(1);
        expect(Math.abs(after!.width - before!.width)).toBeLessThanOrEqual(1);
        await page.screenshot({
          path: info.outputPath("persistent-shared-dice.png"),
          fullPage: true,
          animations: "disabled",
        });
        const result = await (
          await page.request.get(
            `${path}/persistent-review?participant_key=${actor.participant_key}&boundary_key=${encodeURIComponent(boundary)}`,
          )
        ).json();
        expect(result.recovery_results[0]).toEqual(receipt);
        await page.keyboard.press("Escape");
        await expect(page.locator(".ant-popover:visible")).toHaveCount(0);
        await expect(
          reminders.getByRole("button", { name: "View Fire recovery", exact: true }),
        ).toBeFocused();
        await expect(reminders).toBeVisible();
        expect(receipt.dc).toBe(
          outcome === "roll" ? 15 : outcome === "automatic success" ? 0 : 100,
        );
        expect(result.damage_groups).toHaveLength(receipt.success ? 0 : 1);
        const revisionBeforeClose = (await (await page.request.get(path)).json())
          .revision;
        await reminders.getByRole("button", { name: "Close", exact: true }).click();
        await expect(reminders).not.toBeVisible();
        expect((await (await page.request.get(path)).json()).revision).toBe(
          revisionBeforeClose,
        );
        return;
      }
      // A recovery resolved at the table uses the ordinary clear affordance.
      await reminders
        .getByRole("button", { name: "Close", exact: true })
        .last()
        .click();
      const backToRoster = page.getByRole("button", {
        name: "Back to roster",
        exact: true,
      });
      if (await backToRoster.isVisible()) await backToRoster.click();
      await page.locator(`[data-participant-key="${actor.participant_key}"]`).click();
      await page
        .getByRole("button", { name: "Remove Persistent Damage", exact: true })
        .click();
      await expect
        .poll(async () => {
          const latest = await (await page.request.get(path)).json();
          return latest.participants.find(
            (p: { participant_key: string }) =>
              p.participant_key === actor.participant_key,
          ).persistent_damage_groups.length;
        })
        .toBe(0);
      const latest = await (await page.request.get(path)).json();
      expect(
        latest.participants.find(
          (p: { participant_key: string }) =>
            p.participant_key === actor.participant_key,
        ).current_hp,
      ).toBe(30);
      const result = await (
        await page.request.get(
          `${path}/persistent-review?participant_key=${actor.participant_key}&boundary_key=${encodeURIComponent(boundary)}`,
        )
      ).json();
      expect(result.recovery_results).toEqual([]);
    } finally {
      await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
    }
  });
}

test("missing initiative gives a visible reason and direct assignment action", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `No initiative ${Date.now()}` },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name: "Unassigned hero",
            current_hp: 20,
            max_hp: 20,
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await expect(
      page.getByText("Assign initiative to start turn progression.", { exact: true }),
    ).toBeVisible();
    await page.screenshot({
      path: info.outputPath("initiative-required.png"),
      fullPage: true,
      animations: "disabled",
    });
    await page.getByRole("button", { name: "Assign initiative", exact: true }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

test("single source exposes edit and remove without disclosure or phone overflow", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Single source ${Date.now()}` },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name: "Single source hero",
            current_hp: 20,
            max_hp: 20,
            initiative: 20,
          },
        })
      ).ok(),
    ).toBeTruthy();
    const detail = await (await page.request.get(path)).json();
    const actor = detail.participants[0];
    const catalog = await (
      await page.request.get(
        `${path}/participants/${actor.participant_key}/condition-definitions`,
      )
    ).json();
    const definition = catalog.conditions.find(
      (c: { editor_kind: string }) => c.editor_kind === "persistent_damage",
    );
    const added = await page.request.post(
      `${path}/participants/${actor.participant_key}/conditions`,
      {
        data: {
          request_id: `single-${Date.now()}`,
          expected_encounter_revision: detail.revision,
          event_intent: "apply",
          decisions: { keep_upright: false },
          participant_key: actor.participant_key,
          condition_ref: definition.condition_ref,
          scope: { kind: "everyone" },
          extra_action: { kind: "unspecified" },
          immunity_override: false,
          source_participant_key: actor.participant_key,
          note: "Burning oil",
          duration: {
            amount: 2,
            unit: "rounds",
            timing: { participant_key: actor.participant_key, event: "end" },
          },
          parameters: {
            kind: "persistent_damage",
            formula: "1d6",
            damage_type: "fire",
            multiplier: "double",
            recovery: { kind: "specified", dc: 20 },
            episode_id: null,
          },
        },
      },
    );
    expect(added.ok(), await added.text()).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    const select = page.getByRole("button", {
      name: "Single source hero",
      exact: true,
    });
    if (info.project.use.hasTouch) await select.tap();
    else await select.click();
    await expect(
      page.getByRole("link", { name: "Persistent Damage · 1d6 ×2 Fire", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByText("Source: Single source hero", { exact: true }),
    ).toBeVisible();
    await expect(page.getByText("Burning oil", { exact: true })).toBeVisible();
    await expect(page.getByText("2 rounds", { exact: true })).toBeVisible();
    const remove = page.getByRole("button", {
      name: `Remove ${definition.name}`,
      exact: true,
    });
    await expect(remove).toBeVisible();
    await expect(
      page.getByRole("button", { name: `Edit ${definition.name}`, exact: true }),
    ).toBeVisible();
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
    ).toBe(true);
    await page.screenshot({
      path: info.outputPath("single-source-direct-controls.png"),
      fullPage: true,
      animations: "disabled",
    });
    if (info.project.use.hasTouch) await remove.tap();
    else await remove.click();
    await expect(remove).toHaveCount(0);
    await expect(page.getByRole("dialog", { name: /Remove|Confirm/ })).toHaveCount(0);
    await expect
      .poll(
        async () =>
          (await (await page.request.get(path)).json()).participants[0]
            .persistent_damage_groups.length,
      )
      .toBe(0);
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

test("expired condition removal reports failure and retries without an uncaught error", async ({
  page,
}, info) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const { encounter } = await (
    await page.request.post("/api/encounters", {
      data: { name: `Expiry failure ${Date.now()}` },
    })
  ).json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    for (const [display_name, initiative] of [
      ["Outgoing hero", 20],
      ["Incoming hero", 10],
    ] as const) {
      expect(
        (
          await page.request.post(`${path}/participants/manual`, {
            data: {
              encounter_ref: encounter.encounter_key,
              display_name,
              initiative,
              max_hp: 30,
              current_hp: 30,
            },
          })
        ).ok(),
      ).toBeTruthy();
    }
    let detail = await (await page.request.get(path)).json();
    const actor = detail.participants.find(
      (p: { display_name: string }) => p.display_name === "Outgoing hero",
    );
    await page.request.post(`${path}/turn`, {
      data: {
        encounter_ref: encounter.encounter_key,
        participant_key: actor.participant_key,
      },
    });
    detail = await (await page.request.get(path)).json();
    const catalog = await (
      await page.request.get(
        `${path}/participants/${actor.participant_key}/condition-definitions`,
      )
    ).json();
    const hidden = catalog.conditions.find(
      (c: { name: string }) => c.name === "Hidden",
    );
    expect(
      (
        await page.request.post(
          `${path}/participants/${actor.participant_key}/conditions`,
          {
            data: {
              request_id: crypto.randomUUID(),
              expected_encounter_revision: detail.revision,
              participant_key: actor.participant_key,
              condition_ref: hidden.condition_ref,
              event_intent: "apply",
              decisions: { keep_upright: false },
              scope: { kind: "everyone" },
              extra_action: { kind: "unspecified" },
              immunity_override: false,
              parameters: { kind: "none" },
              duration: {
                amount: 1,
                unit: "rounds",
                timing: { participant_key: actor.participant_key, event: "end" },
              },
            },
          },
        )
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await page.getByRole("button", { name: "Next turn", exact: true }).click();
    const popup = page.getByRole("dialog", { name: "Turn reminders", exact: true });
    const remove = popup.getByRole("button", {
      name: "Remove Hidden from Outgoing hero",
      exact: true,
    });
    await expect(remove).toBeEnabled();
    let attempts = 0;
    await page.route("**/conditions/*", async (route) => {
      if (route.request().method() === "DELETE" && attempts++ === 0) {
        await route.fulfill({
          status: 503,
          contentType: "application/json",
          body: JSON.stringify({
            error: { code: "service_busy", message: "Injected removal failure" },
          }),
        });
      } else await route.continue();
    });
    if (info.project.use.hasTouch) await remove.tap();
    else await remove.click();
    await expect(
      popup.getByText("Turn action could not be confirmed", { exact: true }),
    ).toBeVisible();
    await expect(remove).toBeEnabled();
    expect(attempts).toBe(1);
    expect(errors).toEqual([]);
    if (info.project.use.hasTouch) await remove.tap();
    else await remove.click();
    await expect(remove).toBeHidden();
    expect(attempts).toBe(2);
    const after = await (await page.request.get(path)).json();
    expect(
      after.participants.find(
        (p: { participant_key: string }) => p.participant_key === actor.participant_key,
      ).conditions,
    ).toEqual([]);
    expect(errors).toEqual([]);
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

test("real turn counts retain explanations and guard an explicit action", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Wire counts ${info.project.name} ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  const detail = async () => (await page.request.get(path)).json();
  try {
    expect(
      (
        await page.request.post(`${path}/participants/record`, {
          data: {
            encounter_ref: encounter.encounter_key,
            record_ref: "browser-actors:sentinel00000001",
            quantity: 1,
            initiative: -1,
          },
        })
      ).ok(),
    ).toBeTruthy();
    expect(
      (
        await page.request.post(`${path}/participants/record`, {
          data: {
            encounter_ref: encounter.encounter_key,
            record_ref: "browser-hazards:hazard0000000001",
            quantity: 1,
          },
        })
      ).ok(),
    ).toBeTruthy();
    let state = await detail();
    const actor = state.participants.find(
      (participant: { display_name: string }) =>
        participant.display_name === "Atlas Sentinel",
    );
    const hazard = state.participants.find(
      (participant: { participant_kind: string }) =>
        participant.participant_kind === "hazard",
    );
    expect(hazard).not.toHaveProperty("turn_resources");
    expect(actor.turn_resources.regular_total).toBeNull();
    const catalog = await (
      await page.request.get(
        `${path}/participants/${actor.participant_key}/condition-definitions`,
      )
    ).json();
    const slowed = catalog.conditions.find(
      (condition: { name: string }) => condition.name === "Slowed",
    );
    expect(
      (
        await page.request.post(
          `${path}/participants/${actor.participant_key}/conditions`,
          {
            data: {
              request_id: crypto.randomUUID(),
              expected_encounter_revision: state.revision,
              participant_key: actor.participant_key,
              condition_ref: slowed.condition_ref,
              event_intent: "apply",
              decisions: { keep_upright: false },
              scope: { kind: "everyone" },
              extra_action: { kind: "unspecified" },
              immunity_override: false,
              parameters: { kind: "none" },
              value: 1,
            },
          },
        )
      ).ok(),
    ).toBeTruthy();
    state = await detail();
    expect(
      (
        await page.request.post(`${path}/advance`, {
          data: { request_id: crypto.randomUUID(), expected_revision: state.revision },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Atlas Sentinel, Current turn", exact: true }),
      info,
    );
    const calculation = page.getByRole("button", {
      name: /Actions: calculation details/,
    });
    await activate(calculation, info);
    const explanation = page.getByRole("dialog", {
      name: "Actions: calculation details",
      exact: true,
    });
    await expect(explanation.getByRole("row", { name: /Slowed.*-1/ })).toBeVisible();
    await expect(explanation.getByRole("row", { name: /Total.*2/ })).toBeVisible();
    await activate(
      explanation.getByRole("button", { name: "Close explanation", exact: true }),
      info,
    );
    const spend = page.getByRole("button", { name: "Spend action 1", exact: true });
    await expect(spend).toBeEnabled();
    await activate(spend, info);
    await expect(
      page.getByRole("button", { name: "Restore action 1", exact: true }),
    ).toBeEnabled();
    const saved = await detail();
    expect(
      saved.participants.find(
        (participant: { participant_key: string }) =>
          participant.participant_key === actor.participant_key,
      ).turn_resources.regular_remaining,
    ).toBe(1);
    await noPageOverflow(page);
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

for (const recovery of ["Reset encounter", "Rest overnight"] as const) {
  test(`${recovery} clears outgoing reminders across reload without reopening history`, async ({
    page,
  }, info) => {
    const created = await page.request.post("/api/encounters", {
      data: { name: `Reminder lifecycle ${info.project.name} ${Date.now()}` },
    });
    expect(created.ok()).toBeTruthy();
    const { encounter } = await created.json();
    const path = `/api/encounters/${encounter.encounter_key}`;
    try {
      for (const [display_name, initiative] of [
        ["Outgoing hero", 20],
        ["Incoming hero", 10],
      ] as const) {
        const added = await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name,
            initiative,
            max_hp: 30,
            current_hp: 30,
          },
        });
        expect(added.ok()).toBeTruthy();
      }
      let detail = await (await page.request.get(path)).json();
      const actor = detail.participants.find(
        (participant: { display_name: string }) =>
          participant.display_name === "Outgoing hero",
      );
      const assigned = await page.request.post(`${path}/turn`, {
        data: {
          encounter_ref: encounter.encounter_key,
          participant_key: actor.participant_key,
        },
      });
      expect(assigned.ok()).toBeTruthy();
      detail = await (await page.request.get(path)).json();
      const catalog = await (
        await page.request.get(
          `${path}/participants/${actor.participant_key}/condition-definitions`,
        )
      ).json();
      const hidden = catalog.conditions.find(
        (condition: { name: string }) => condition.name === "Hidden",
      );
      const applied = await page.request.post(
        `${path}/participants/${actor.participant_key}/conditions`,
        {
          data: {
            request_id: crypto.randomUUID(),
            expected_encounter_revision: detail.revision,
            participant_key: actor.participant_key,
            condition_ref: hidden.condition_ref,
            event_intent: "apply",
            decisions: { keep_upright: false },
            scope: { kind: "everyone" },
            extra_action: { kind: "unspecified" },
            immunity_override: false,
            parameters: { kind: "none" },
            duration: {
              amount: 1,
              unit: "rounds",
              timing: { participant_key: actor.participant_key, event: "end" },
            },
          },
        },
      );
      expect(applied.ok(), await applied.text()).toBeTruthy();
      await page.goto(`/encounters/${encounter.slug}`);
      await activate(
        page.getByRole("button", { name: "Next turn", exact: true }),
        info,
      );
      const reminders = page.getByRole("dialog", {
        name: "Turn reminders",
        exact: true,
      });
      await expect(reminders).toBeVisible();
      await expect(
        reminders.getByRole("button", {
          name: "Remove Hidden from Outgoing hero",
          exact: true,
        }),
      ).toBeVisible();
      const before = await (await page.request.get(`${path}/turn-review`)).json();
      expect(before.outgoing.participant_key).toBe(actor.participant_key);
      expect(before.outgoing.reminders).not.toEqual([]);
      expect(before.outgoing.boundary_key).toBeTruthy();
      await activate(
        reminders.getByRole("button", { name: "Close", exact: true }).last(),
        info,
      );
      await activate(
        page.getByRole("button", { name: "Encounter settings", exact: true }),
        info,
      );
      await activate(page.getByRole("menuitem", { name: recovery, exact: true }), info);
      const review = page.getByRole("dialog", { name: recovery, exact: true });
      await expect(review).toBeVisible();
      if (recovery === "Rest overnight")
        await activate(
          review.getByRole("radio", { name: "Full HP", exact: true }),
          info,
        );
      const submit = review.getByRole("button", { name: recovery, exact: true });
      await expect(submit).toBeEnabled();
      await activate(submit, info);
      await expect(review).toBeHidden();
      await expect
        .poll(
          async () =>
            (await (await page.request.get(`${path}/turn-review`)).json()).outgoing,
        )
        .toBeNull();
      await page.reload();
      await expect(
        page.getByRole("button", { name: "Encounter settings", exact: true }),
      ).toBeVisible();
      const reloaded = await (await page.request.get(`${path}/turn-review`)).json();
      expect(reloaded.outgoing).toBeNull();
      await expect(reminders).toBeHidden();
      await expect(
        page.getByRole("button", { name: "Outgoing turn reminders", exact: true }),
      ).toHaveCount(0);
      await expect(
        page.getByRole("button", {
          name: "Remove Hidden from Outgoing hero",
          exact: true,
        }),
      ).toHaveCount(0);
      await noPageOverflow(page);
    } finally {
      await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
    }
  });
}

test("manual trackers persist minute countdowns and restart after elapsing", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Manual tracker ${Date.now()}` },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name: "Tracker dragon",
            initiative: 20,
            max_hp: 20,
            current_hp: 20,
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Tracker dragon", exact: true }),
      info,
    );
    await activate(
      page.getByRole("button", { name: "Add tracker", exact: true }),
      info,
    );
    const dialog = page.getByRole("dialog", { name: "Add tracker", exact: true });
    await expect(dialog).toBeVisible();
    await dialog.getByLabel("Name", { exact: true }).fill("Breath ready");
    await activate(dialog.getByRole("combobox", { name: "Duration unit" }), info);
    await activate(
      page.getByText("minutes", { exact: true }).filter({ visible: true }),
      info,
    );
    await activate(
      dialog.getByRole("button", { name: "Save tracker", exact: true }),
      info,
    );
    await expect(dialog).not.toBeVisible();
    await expect(page.getByText("10 rounds remaining", { exact: true })).toBeVisible();
    await noPageOverflow(page);
    await page.reload();
    const participant = page.getByRole("button", {
      name: /^Tracker dragon(?:, Current turn)?$/,
    });
    await activate(participant, info);
    await expect(page.getByText("10 rounds remaining", { exact: true })).toBeVisible();
    await activate(
      page.getByRole("button", { name: "Encounter settings", exact: true }),
      info,
    );
    await activate(
      page.getByRole("menuitem", { name: "Edit encounter", exact: true }),
      info,
    );
    const settings = page.getByRole("dialog", { name: "Settings", exact: true });
    await expect(
      settings.getByRole("spinbutton", { name: "Seconds per round" }),
    ).toHaveValue("6");
    await settings.getByRole("spinbutton", { name: "Seconds per round" }).fill("30");
    await activate(settings.getByRole("button", { name: "Save", exact: true }), info);
    await expect(settings).not.toBeVisible();
    await expect(page.getByText("2 rounds remaining", { exact: true })).toBeVisible();
    let detail = await (await page.request.get(path)).json();
    expect(detail.encounter.round_duration_seconds).toBe(30);
    expect(
      (
        await page.request.post(`${path}/advance`, {
          data: {
            request_id: `tracker-start-${Date.now()}`,
            expected_revision: detail.revision,
          },
        })
      ).ok(),
    ).toBeTruthy();
    for (let tick = 0; tick < 2; tick++) {
      detail = await (await page.request.get(path)).json();
      const response = await page.request.post(`${path}/advance`, {
        data: {
          request_id: `tracker-tick-${tick}-${Date.now()}`,
          expected_revision: detail.revision,
        },
      });
      expect(response.ok(), await response.text()).toBeTruthy();
    }
    await page.reload();
    await activate(participant, info);
    await expect(page.getByRole("status").filter({ hasText: "Elapsed" })).toBeVisible();
    await activate(
      page.getByRole("button", { name: "Restart Breath ready", exact: true }),
      info,
    );
    await expect(page.getByText("2 rounds remaining", { exact: true })).toBeVisible();
    const editTracker = page.getByRole("button", {
      name: "Edit tracker Breath ready",
      exact: true,
    });
    await editTracker.focus();
    await expect(
      page.getByRole("tooltip", { name: "Edit", exact: true }),
    ).toBeVisible();
    await editTracker.press("Enter");
    await expect(page.getByRole("tooltip", { name: "Edit", exact: true })).toHaveCount(
      0,
    );
    const edit = page.getByRole("dialog", { name: "Edit tracker", exact: true });
    await edit.getByLabel("Name", { exact: true }).fill("Cold breath ready");
    await activate(
      edit.getByRole("button", { name: "Save tracker", exact: true }),
      info,
    );
    await expect(edit).not.toBeVisible();
    await expect(page.getByText("Cold breath ready", { exact: true })).toBeVisible();
    await activate(
      page.getByRole("button", {
        name: "Remove tracker Cold breath ready",
        exact: true,
      }),
      info,
    );
    await expect(
      page.getByText("Cold breath ready", { exact: true }),
    ).not.toBeVisible();
    await noPageOverflow(page);
  } finally {
    await page.request.delete(path);
  }
});

for (const boundary of ["start", "end"] as const) {
  test(`tracker completion notifies once at turn ${boundary} and can be removed`, async ({
    page,
  }, info) => {
    const created = await page.request.post("/api/encounters", {
      data: { name: `Tracker notification ${boundary} ${Date.now()}` },
    });
    const { encounter } = await created.json();
    const path = `/api/encounters/${encounter.encounter_key}`;
    try {
      expect(
        (
          await page.request.post(`${path}/participants/manual`, {
            data: {
              encounter_ref: encounter.encounter_key,
              display_name: "Tracker owner",
              initiative: 20,
              max_hp: 20,
              current_hp: 20,
            },
          })
        ).ok(),
      ).toBeTruthy();
      await page.goto(`/encounters/${encounter.slug}`);
      await activate(
        page.getByRole("button", { name: "Tracker owner", exact: true }),
        info,
      );
      await activate(page.getByRole("button", { name: "Add tracker" }), info);
      const editor = page.getByRole("dialog", { name: "Add tracker", exact: true });
      await editor.getByLabel("Name", { exact: true }).fill("Countdown");
      await expect(editor.getByRole("radio", { name: "Starts on" })).toBeChecked();
      if (boundary === "end")
        await activate(editor.getByText("Ends on", { exact: true }), info);
      await activate(editor.getByRole("button", { name: "Save tracker" }), info);
      await expect(editor).not.toBeVisible();
      const advance = () =>
        activate(
          page
            .getByRole("button", { name: "Next turn", exact: true })
            .filter({ visible: true }),
          info,
        );
      await activate(
        page
          .getByRole("button", { name: /^(Start|Next) turn$/ })
          .filter({ visible: true }),
        info,
      );
      const notification = page.getByRole("dialog", {
        name: "Turn reminders",
        exact: true,
      });
      if (boundary === "end") {
        await expect(notification).not.toBeVisible();
        await advance();
      }
      await expect(
        notification.getByRole("heading", { name: "Countdown" }),
      ).toBeVisible();
      await expect(
        notification.getByRole("status").filter({ hasText: "Elapsed" }),
      ).toBeVisible();
      await activate(
        notification.getByRole("button", { name: "Close", exact: true }).last(),
        info,
      );
      let detail = await (await page.request.get(path)).json();
      expect(detail.participants[0].manual_trackers[0].remaining_rounds).toBe(0);
      await advance();
      await expect(notification).not.toBeVisible();
      await activate(page.getByRole("button", { name: "Restart Countdown" }), info);
      await advance();
      await expect(
        notification.getByRole("heading", { name: "Countdown" }),
      ).toBeVisible();
      await activate(
        notification.getByRole("button", {
          name: "Remove tracker Countdown from Tracker owner",
          exact: true,
        }),
        info,
      );
      await expect(
        notification.getByRole("status").filter({ hasText: "Removed" }),
      ).toBeVisible();
      detail = await (await page.request.get(path)).json();
      expect(detail.participants[0].manual_trackers).toEqual([]);
      await noPageOverflow(page);
    } finally {
      await page.request.delete(path);
    }
  });
}

test("end and start reminders use their own actors and background sheets", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Boundary actors ${Date.now()}` },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    for (const [display_name, initiative] of [
      ["Adventurer", 20],
      ["Green Dragon", 10],
    ] as const) {
      expect(
        (
          await page.request.post(`${path}/participants/manual`, {
            data: {
              encounter_ref: encounter.encounter_key,
              display_name,
              initiative,
              max_hp: 20,
              current_hp: 20,
            },
          })
        ).ok(),
      ).toBeTruthy();
    }
    for (const [owner, event, name] of [
      ["Adventurer", "end", "End countdown"],
      ["Green Dragon", "start", "Start countdown"],
    ] as const) {
      const detail = await (await page.request.get(path)).json();
      const participant = detail.participants.find(
        (p: { display_name: string }) => p.display_name === owner,
      );
      expect(
        (
          await page.request.post(
            `${path}/participants/${participant.participant_key}/manual-trackers`,
            {
              data: {
                request_id: `phase-${event}-${Date.now()}`,
                participant_key: participant.participant_key,
                expected_encounter_revision: detail.revision,
                expected_participant_revision: participant.revision,
                action: {
                  kind: "add",
                  name,
                  duration: {
                    amount: 1,
                    unit: "rounds",
                    timing: { participant_key: participant.participant_key, event },
                  },
                },
              },
            },
          )
        ).ok(),
      ).toBeTruthy();
    }
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page
        .getByRole("button", { name: "Start turn", exact: true })
        .filter({ visible: true }),
      info,
    );
    const popup = page.getByRole("dialog", { name: "Turn reminders", exact: true });
    await expect(popup).not.toBeVisible();
    await expect(
      page.locator(".encounter-record-pane").getByRole("button", {
        name: "Edit Participant name",
        includeHidden: true,
      }),
    ).toBeVisible();
    await activate(
      page
        .getByRole("button", { name: "Next turn", exact: true })
        .filter({ visible: true }),
      info,
    );
    const identity = popup.locator(".encounter-turn-review__identity");
    const backgroundActor = page
      .locator(".encounter-record-pane")
      .getByRole("button", { name: "Edit Participant name", includeHidden: true });
    await expect(
      identity.getByRole("heading", { name: "Adventurer", exact: true }),
    ).toBeVisible();
    await expect(popup.getByText("End countdown", { exact: true })).toBeVisible();
    await expect(popup.getByText("Start countdown", { exact: true })).toHaveCount(0);
    await expect(backgroundActor).toHaveText("Adventurer");
    const committed = await (await page.request.get(path)).json();
    await activate(popup.getByRole("button", { name: "Continue", exact: true }), info);
    await expect(
      identity.getByRole("heading", { name: "Green Dragon", exact: true }),
    ).toBeVisible();
    await expect(backgroundActor).toHaveText("Green Dragon");
    await expect(popup.getByText("Start countdown", { exact: true })).toBeVisible();
    await expect(popup.getByText("End countdown", { exact: true })).toHaveCount(0);
    expect((await (await page.request.get(path)).json()).revision).toBe(
      committed.revision,
    );
    const remove = popup.getByRole("button", {
      name: "Remove tracker Start countdown from Green Dragon",
      exact: true,
    });
    await expect(remove).toHaveText("Remove");
    await activate(remove, info);
    await expect(popup.getByRole("status")).toHaveText("Removed");
    await activate(
      popup.getByRole("button", { name: "Close", exact: true }).last(),
      info,
    );
    await expect(popup).not.toBeVisible();
    await expect(backgroundActor).toHaveText("Green Dragon");
    await expect(page.locator(".encounter-record-pane")).toBeFocused();
    await noPageOverflow(page);
  } finally {
    await page.request.delete(path);
  }
});
