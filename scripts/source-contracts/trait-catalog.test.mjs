import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { extractTraitCatalog } from "./trait-catalog.mjs";

const fixture = fileURLToPath(new URL("./fixtures/trait-catalog/", import.meta.url));
const catalog = (result, name) => result.catalogs.find((entry) => entry.name === name);
const item = (result, name, identifier) => catalog(result, name).entries.find((entry) => entry.identifier === identifier);
function changeFixture(t, mutate) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "atlas-trait-catalog-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  fs.cpSync(fixture, temporary, { recursive: true });
  mutate(temporary);
  return temporary;
}

test("spreads and overrides preserve metadata separately for each catalog", () => {
  const result = extractTraitCatalog(fixture);
  assert.equal(result.complete, true);
  assert.equal(item(result, "equipmentTraits", "shared").label, "Override label");
  assert.equal(item(result, "weaponTraits", "shared").label, "Original label");
  assert.equal(item(result, "weaponTraits", "shared").source.line, 8);
  assert.equal(item(result, "equipmentTraits", "shared").source.line, 13);
  assert.deepEqual(catalog(result, "effectTraits").entries.map((entry) => entry.identifier), ["shared"]);
  assert.equal(catalog(result, "equipmentTraits").exposedInConfig, true);
  assert.equal(catalog(result, "equipmentTraits").exportedFromTraits, true);
  assert.equal(catalog(result, "baseTraits").exposedInConfig, false);
  assert.equal(catalog(result, "baseTraits").exportedFromTraits, false);
});

test("computed ranges and explicit parameterized description keys resolve without inference", () => {
  const result = extractTraitCatalog(fixture);
  assert.equal(item(result, "weaponTraits", "range-increment-30").labelKey, "PF2E.TraitRangeIncrement30");
  assert.equal(item(result, "weaponTraits", "range-increment-30").descriptionKey, "PF2E.RangeDescription");
  assert.equal(item(result, "equipmentTraits", "deadly-2d10").description, "Authored general Deadly description");
  assert.equal(item(result, "equipmentTraits", "fire").description, "Fire prose with @UUID[example].");
});

test("missing labels and descriptions are visible and trait/tag identities are separate", () => {
  const result = extractTraitCatalog(fixture);
  assert.equal(item(result, "equipmentTraits", "undocumented").descriptionKey, null);
  assert.equal(item(result, "equipmentTraits", "missing").label, null);
  assert.equal(item(result, "equipmentTraits", "missing").descriptionKey, "PF2E.MissingDescription");
  assert.equal(item(result, "equipmentTraits", "missing").description, null);
  assert.equal(catalog(result, "otherArmorTags").namespace, "otherTag");
  assert.equal(item(result, "otherArmorTags", "fire").label, "Fire tag");
  assert.equal(item(result, "otherArmorTags", "fire").description, null);
  assert.equal(catalog(result, "rarityTraits").namespace, "rarity");
  assert.deepEqual(result.diagnostics.map((entry) => entry.code), ["missing-localization", "missing-localization"]);
});

test("family evidence distinguishes runtime getter from declaration vocabulary", () => {
  const result = extractTraitCatalog(fixture);
  assert.deepEqual(catalog(result, "equipmentTraits").familyBindings.map(({ scope, family, basis }) => ({ scope, family, basis })), [
    { scope: "item", family: "equipment", basis: "runtime-validTraits" },
    { scope: "item", family: "equipment", basis: "declared-keyof-vocabulary" },
  ]);
  assert.deepEqual(catalog(result, "weaponTraits").familyBindings, []);
  assert.deepEqual(catalog(result, "otherArmorTags").familyBindings.map(({ scope, family, basis }) => ({ scope, family, basis })), [
    { scope: "item", family: "armor", basis: "catalog-key-type" },
  ]);
});

test("unrecognized reachable calls are diagnosed and never evaluated", (t) => {
  const changed = changeFixture(t, (root) => {
    const file = path.join(root, "src/scripts/config/traits.ts");
    fs.appendFileSync(file, '\nconst unsupportedTraits = dangerousRuntimeCall();\nexport { unsupportedTraits };\n');
    const index = path.join(root, "src/scripts/config/index.ts");
    fs.writeFileSync(index, fs.readFileSync(index, "utf8").replace('import { equipmentTraits,', 'import { unsupportedTraits, equipmentTraits,').replace('    equipmentTraits,', '    unsupportedTraits,\n    equipmentTraits,'));
  });
  const result = extractTraitCatalog(changed);
  assert.equal(result.complete, false);
  assert.equal(catalog(result, "unsupportedTraits").complete, false);
  assert.deepEqual(catalog(result, "unsupportedTraits").entries, []);
  assert.ok(result.diagnostics.some((entry) => entry.code === "unsupported-call" && entry.message.includes("dangerousRuntimeCall")));
  assert.equal(item(result, "equipmentTraits", "fire").label, "Fire");
});

test("unsupported range slug inputs make affected catalogs incomplete", (t) => {
  const changed = changeFixture(t, (root) => {
    fs.writeFileSync(path.join(root, "src/module/item/base/data/values.ts"), 'const RANGE_TRAITS = ["unknown-new-format"] as const; export { RANGE_TRAITS };');
  });
  const result = extractTraitCatalog(changed);
  assert.equal(result.complete, false);
  assert.equal(catalog(result, "weaponTraits").complete, false);
  assert.ok(result.diagnostics.some((entry) => entry.code === "unsupported-sluggify"));
});

test("repeat output is deterministic and authored changes yield an identifiable catalog diff", (t) => {
  const before = extractTraitCatalog(fixture);
  assert.equal(JSON.stringify(before), JSON.stringify(extractTraitCatalog(fixture)));
  const changed = changeFixture(t, (root) => {
    const file = path.join(root, "static/lang/en.json");
    const labels = JSON.parse(fs.readFileSync(file, "utf8"));
    labels.PF2E.Override = "Updated label";
    fs.writeFileSync(file, JSON.stringify(labels));
  });
  const after = extractTraitCatalog(changed);
  assert.notEqual(JSON.stringify(before), JSON.stringify(after));
  assert.equal(item(after, "equipmentTraits", "shared").label, "Updated label");
  assert.equal(item(after, "weaponTraits", "shared").label, "Original label");
  assert.equal(item(before, "equipmentTraits", "shared").source.file, item(after, "equipmentTraits", "shared").source.file);
});

test("malformed localization and non-string catalog metadata cannot count as complete", (t) => {
  const malformed = changeFixture(t, (root) => {
    fs.writeFileSync(path.join(root, "static/lang/en.json"), "{ invalid }");
  });
  const missing = extractTraitCatalog(malformed);
  assert.equal(missing.complete, false);
  assert.ok(missing.diagnostics.some((entry) => entry.code === "invalid-localization-json"));
  const changed = changeFixture(t, (root) => {
    const file = path.join(root, "src/scripts/config/traits.ts");
    fs.writeFileSync(file, fs.readFileSync(file, "utf8").replace('shared: "PF2E.Override"', 'shared: { invalid: "PF2E.Override" }'));
  });
  const invalid = extractTraitCatalog(changed);
  assert.equal(invalid.complete, false);
  assert.equal(catalog(invalid, "equipmentTraits").complete, false);
  assert.equal(item(invalid, "equipmentTraits", "shared").labelKey, null);
  assert.ok(invalid.diagnostics.some((entry) => entry.code === "invalid-localization-key"));
  assert.deepEqual(JSON.parse(JSON.stringify(invalid)), invalid);
});
