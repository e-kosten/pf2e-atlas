import { expect, test } from "./fixtures";
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
        const before = await anchor.boundingBox();
        const completed = page.waitForResponse(
          (response) =>
            response.url().endsWith("/persistent-recovery") &&
            response.request().method() === "POST",
        );
        await page.getByRole("button", { name: "Roll", exact: true }).click();
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
            `${receipt.success ? "Recovered" : "Still active"} · DC ${receipt.dc}`,
            { exact: true },
          ),
        ).toBeVisible();
        await expect(reminders).toBeVisible();
        const after = await anchor.boundingBox();
        expect(before).not.toBeNull();
        expect(after).not.toBeNull();
        expect(Math.abs(after!.x - before!.x)).toBeLessThanOrEqual(1);
        expect(Math.abs(after!.y - before!.y)).toBeLessThanOrEqual(1);
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
    await expect(popup.getByText("Turn action failed", { exact: true })).toBeVisible();
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
