import { expect, test } from "./fixtures";
import type { AddEncounterRecordParticipantRequest } from "../../src/generated/atlas";

for (const width of [1440, 390, 320]) {
  test(`participant name edits persist at ${width}px and keep the creature's other state`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    const search = await page.request.post("/api/search/page", {
      data: {
        expected_artifact_generation: null,
        criteria: {
          kind: "list_records",
          filter: { kind: "in", field: "kind", values: ["creature"] },
        },
        page: { number: 1, size: 1 },
      },
    });
    expect(search.ok()).toBeTruthy();
    const record = (await search.json()).rows[0].record;
    const created = await page.request.post("/api/encounters", {
      data: { name: `Inline names ${width} ${Date.now()}` },
    });
    expect(created.ok()).toBeTruthy();
    const { encounter } = await created.json();
    const path = `/api/encounters/${encounter.encounter_key}`;
    try {
      expect(
        (
          await page.request.post(`${path}/participants/record`, {
            data: {
              encounter_ref: encounter.encounter_key,
              record_ref: record.record_key,
              quantity: 1,
              initiative: 17,
            } satisfies AddEncounterRecordParticipantRequest,
          })
        ).ok(),
      ).toBeTruthy();
      const original = (await (await page.request.get(path)).json()).participants[0];
      await page.goto(`/encounters/${encounter.slug}`);
      await page
        .getByRole("button", { name: original.display_name, exact: true })
        .click();
      const trigger = page.getByRole("button", {
        name: "Edit Participant name",
        exact: true,
      });
      const editor = page.getByRole("textbox", {
        name: "Participant name",
        exact: true,
      });
      await trigger.focus();
      await page.keyboard.press("Enter");
      await expect(editor).toBeFocused();
      await editor.fill("Cancelled name");
      await editor.press("Escape");
      await expect(trigger).toBeFocused();
      await expect(trigger).toHaveText(original.display_name);

      await trigger.click();
      await editor.fill("   ");
      await editor.press("Enter");
      await expect(page.getByRole("alert")).toContainText("Enter a participant name.");
      await editor.fill("  Creature captain  ");
      await editor.press("Enter");
      await expect(trigger).toHaveText("Creature captain");
      await expect(trigger).toBeFocused();
      if (width >= 768) {
        await expect(
          page.getByRole("button", { name: "Creature captain", exact: true }),
        ).toHaveAttribute("aria-pressed", "true");
      }
      const saved = (await (await page.request.get(path)).json()).participants[0];
      expect(saved).toMatchObject({
        display_name: "Creature captain",
        record_key: original.record_key,
        participant_variant: original.participant_variant,
        initiative: original.initiative,
        current_hp: original.current_hp,
        max_hp: original.max_hp,
        temporary_hp: original.temporary_hp,
        hp_origin: original.hp_origin,
        side: original.side,
      });

      await trigger.click();
      const longName = "Captain of the remarkably long named goblin expedition "
        .repeat(3)
        .trim();
      await editor.fill(longName);
      await page
        .getByRole("button", { name: "Participant settings", exact: true })
        .focus();
      await expect(trigger).toHaveText(longName);
      const nameBounds = (await trigger.boundingBox())!;
      const traitsBounds = (await page
        .locator(".encounter-reference__identity > .badge-row")
        .boundingBox())!;
      expect(traitsBounds.y).toBeGreaterThanOrEqual(
        nameBounds.y + nameBounds.height - 1,
      );
      if (width < 768) {
        const kindBounds = (await page
          .locator(".encounter-reference__title > .record-reference__kind")
          .boundingBox())!;
        expect(kindBounds.y).toBeGreaterThanOrEqual(
          nameBounds.y + nameBounds.height - 1,
        );
      }
      await trigger.click();
      await expect(editor).toBeFocused();
      await editor.press("Escape");
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth + 1,
        ),
      ).toBe(true);
      await page.reload();
      if (width < 768)
        await page.getByRole("button", { name: longName, exact: true }).click();
      await expect(trigger).toHaveText(longName);
      await expect(
        page.getByRole("heading", { name: longName, exact: true }),
      ).toBeVisible();
    } finally {
      await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
    }
  });
}
