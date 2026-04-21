import { RuleTester } from "eslint";
import { describe, expect, it } from "vitest";

import localRules from "../../eslint-local-rules.js";
import { lintMessageTexts, lintRuleMessages } from "./eslint-config-test-helpers.js";

type TestedRuleModule = Parameters<RuleTester["run"]>[1];

const typedLocalRules = localRules as unknown as { rules: Record<string, TestedRuleModule> };

const ruleTester = new RuleTester({
  languageOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
  },
});

function getLocalRule(name: string): TestedRuleModule {
  const rule = typedLocalRules.rules[name];
  if (!rule) {
    throw new Error(`Missing local ESLint rule: ${name}`);
  }

  return rule;
}

async function expectRuleMessage(
  filePath: string,
  code: string,
  expectedMessage: string,
  ruleId = "no-restricted-imports",
) {
  const messages = lintMessageTexts(await lintRuleMessages(filePath, code, ruleId));

  expect(messages.some((message) => message.includes(expectedMessage))).toBe(true);
}

async function expectNoRuleMessages(filePath: string, code: string, ruleId = "no-restricted-imports") {
  expect(await lintRuleMessages(filePath, code, ruleId)).toHaveLength(0);
}

describe("eslint local architecture rules", () => {
  it("blocks stale user-facing search-screen terminology but allows internal import paths", () => {
    ruleTester.run("no-stale-search-screen-terminology", getLocalRule("no-stale-search-screen-terminology"), {
      valid: [
        {
          filename: "src/tui/search-screen/controller.ts",
          code: `
            import { buildStructuredDraftEntries } from "./structured-draft-support.js";
            const message = "Apply the staged query and return to the live editor.";
            export { buildStructuredDraftEntries, message };
          `,
        },
      ],
      invalid: [
        {
          filename: "src/tui/search-screen/controller.ts",
          code: `
            const message = "Commit draft";
            export { message };
          `,
          errors: [{ messageId: "noStaleSearchTerminology" }],
        },
        {
          filename: "src/tui/search-screen/controller.ts",
          code: `
            const message = \`Open the setup editor first.\`;
            export { message };
          `,
          errors: [{ messageId: "noStaleSearchTerminology" }],
        },
      ],
    });
  });

  it("blocks direct terminal event routing in feature code", () => {
    ruleTester.run("no-direct-terminal-event-routing", getLocalRule("no-direct-terminal-event-routing"), {
      valid: [
        {
          code: `
            const interactionAction = resolveTerminalInteractionAction(event, [{ id: "cancel" }]);
            const textIntent = resolveTerminalTextEntryIntent(event);
            if (interactionAction?.id === "cancel") {
              handleCancel();
            }
            if (textIntent?.kind === "append") {
              append(textIntent.text);
            }
          `,
        },
      ],
      invalid: [
        {
          code: `
            if (event.textInputAction === "cancel") {
              handleCancel();
            }
          `,
          errors: [{ messageId: "noDirectCancelHandling" }],
        },
        {
          code: `
            if (event.textInputAction === "submit") {
              handleSubmit();
            }
          `,
          errors: [{ messageId: "noDirectSubmitHandling" }],
        },
        {
          code: `
            if (event.textInputAction === "deleteBackward") {
              handleDelete();
            }
          `,
          errors: [{ messageId: "noDirectDeleteBackwardHandling" }],
        },
        {
          code: `
            if ("cancel" === event.textInputAction) {
              handleCancel();
            }
          `,
          errors: [{ messageId: "noDirectCancelHandling" }],
        },
        {
          code: `
            switch (event.textInputAction) {
              case "submit":
                handleSubmit();
                break;
            }
          `,
          errors: [{ messageId: "noDirectSubmitHandling" }],
        },
        {
          code: `
            if (event.systemAction === "interrupt") {
              exitApp();
            }
          `,
          errors: [{ messageId: "noDirectInterruptHandling" }],
        },
        {
          code: `
            if (event.printable) {
              append(event.printable);
            }
          `,
          errors: [{ messageId: "noDirectPrintableHandling" }, { messageId: "noDirectPrintableHandling" }],
        },
        {
          code: `
            if (event.isBackNavigationKey()) {
              onBack();
            }
          `,
          errors: [{ messageId: "noDirectBackHandling" }],
        },
        {
          code: `
            if (event.isTerminalQuitKey()) {
              onQuit();
            }
          `,
          errors: [{ messageId: "noDirectQuitHandling" }],
        },
      ],
    });
  });

  it("blocks direct JSON.parse outside approved decoder modules and allows explicit decoder boundaries", async () => {
    await expectRuleMessage(
      "src/search/ranking.ts",
      "const parsed = JSON.parse(serialized);\nexport { parsed };\n",
      "Route JSON decoding through an approved decoder or boundary module instead.",
      "arch/no-direct-json-parse",
    );

    await expectNoRuleMessages(
      "src/data/sql-row-decoding.ts",
      "const parsed = JSON.parse(serialized);\nexport { parsed };\n",
      "arch/no-direct-json-parse",
    );

    await expectNoRuleMessages(
      "src/tags/editorial/sessions/session-store.ts",
      "const parsed = JSON.parse(serialized);\nexport { parsed };\n",
      "arch/no-direct-json-parse",
    );
  });

  it("blocks DatabaseSync construction outside approved entry modules and allows composition roots", async () => {
    await expectRuleMessage(
      "src/data/service.ts",
      "const db = new DatabaseSync(path);\nexport { db };\n",
      "Open SQLite connections only in approved composition or entry modules.",
      "arch/no-direct-database-sync-construction",
    );

    await expectNoRuleMessages(
      "src/tags/cli/evaluation/evaluate-movement.ts",
      "const db = new DatabaseSync(path);\nexport { db };\n",
      "arch/no-direct-database-sync-construction",
    );
  });

  it("blocks raw SearchCategory and SearchSubcategory assertions in favor of parser helpers", async () => {
    const messages = await lintRuleMessages(
      "src/search/ranking.ts",
      `
        const category = raw as SearchCategory;
        const subcategories = raw as SearchSubcategory[];
        const typedCategory = <SearchCategory>raw;
        export { category, subcategories, typedCategory };
      `,
      "arch/no-search-category-assertion",
    );

    expect(messages).toHaveLength(3);

    await expectNoRuleMessages(
      "src/search/ranking.ts",
      "const category = normalizeSearchCategory(raw);\nexport { category };\n",
      "arch/no-search-category-assertion",
    );
  });

  it("blocks internal src/tags modules from importing the public tags barrel", async () => {
    await expectRuleMessage(
      "src/tags/runtime/derivation/api.ts",
      'export * from "../../index.js";\n',
      "Internal src/tags modules must not import the src/tags/index.js barrel.",
      "arch/no-internal-tags-barrel-imports",
    );

    await expectNoRuleMessages(
      "src/tags/runtime/derivation/api.ts",
      'export * from "./assignments.js";\n',
      "arch/no-internal-tags-barrel-imports",
    );
  });

  it("blocks non-UI package layers from importing tui modules", async () => {
    const cases = [
      {
        filePath: "src/app/ontology-service.ts",
        code: 'import { useDerivedTagTerminalInput } from "../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
      {
        filePath: "src/app/runtime.ts",
        code: 'import { useDerivedTagTerminalInput } from "../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
      {
        filePath: "src/data/service.ts",
        code: 'import { useDerivedTagTerminalInput } from "../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
      {
        filePath: "src/domain/categories.ts",
        code: 'import { useDerivedTagTerminalInput } from "../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
      {
        filePath: "src/search/ranking.ts",
        code: 'import { useDerivedTagTerminalInput } from "../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
      {
        filePath: "src/server/presenters.ts",
        code: 'import { useDerivedTagTerminalInput } from "../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
      {
        filePath: "src/tags/runtime/publication/catalog.ts",
        code: 'import { useDerivedTagTerminalInput } from "../../../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
      {
        filePath: "src/tags/runtime/matcher/engine.ts",
        code: 'import { useDerivedTagTerminalInput } from "../../../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
      },
    ];

    for (const testCase of cases) {
      const messages = lintMessageTexts(await lintRuleMessages(testCase.filePath, testCase.code));
      expect(
        messages.some((message) =>
          message.includes(
            "Non-UI application, data, domain, search, server, and tag modules must not import src/tui internals directly.",
          ),
        ),
      ).toBe(true);
    }
  });

  it("blocks non-tag modules from importing leaf derived-tag internals and allows the shared facade", async () => {
    await expectRuleMessage(
      "src/app/runtime.ts",
      'import { readDerivedTagCatalog } from "../tags/runtime/catalog-utils.js";\nexport const value = readDerivedTagCatalog;\n',
      "Outside src/tags, import derived-tag functionality through src/tags/index.js or another approved facade instead of leaf tag internals.",
    );

    await expectRuleMessage(
      "src/app/runtime.ts",
      'import { buildDerivedTagMigrationSession } from "../tags/editorial/session-builder.js";\nexport const value = buildDerivedTagMigrationSession;\n',
      "Outside src/tags, import derived-tag functionality through src/tags/index.js or another approved facade instead of leaf tag internals.",
    );

    await expectRuleMessage(
      "src/app/runtime.ts",
      'import { REVIEWED_DISCOVERY_RECORDS } from "../tags/reviews/discovery-reviewed-records.js";\nexport const value = REVIEWED_DISCOVERY_RECORDS;\n',
      "Outside src/tags, import derived-tag functionality through src/tags/index.js or another approved facade instead of leaf tag internals.",
    );

    await expectNoRuleMessages(
      "src/app/runtime.ts",
      'import { normalizeDerivedTag } from "../tags/index.js";\nexport const value = normalizeDerivedTag;\n',
    );
  });

  it("blocks non-tag modules from importing the transitional domain barrel and allows explicit domain modules", async () => {
    await expectRuleMessage(
      "src/app/runtime.ts",
      'import { SearchFilters } from "../domain/index.js";\nexport type Value = SearchFilters;\n',
      "Non-tag code must import domain contracts from explicit src/domain/* modules instead of the transitional src/domain/index.js barrel.",
    );

    await expectNoRuleMessages(
      "src/app/runtime.ts",
      'import { SearchFilters } from "../domain/search-types.js";\nexport type Value = SearchFilters;\n',
    );

    await expectNoRuleMessages(
      "src/tags/runtime/derivation/api.ts",
      'import { SearchFilters } from "../../../domain/index.js";\nexport type Value = SearchFilters;\n',
    );
  });

  it("blocks owner-specific shared utils imports in non-tag modules but keeps normalizeText and uniqueSorted available", async () => {
    await expectRuleMessage(
      "src/data/references.ts",
      'import { firstString, getNested, stripHtml } from "../shared/utils.js";\nexport { firstString, getNested, stripHtml };\n',
      "Non-tag code must import owner-specific helpers from their owning module instead of src/shared/utils.js. Keep shared/utils limited to true cross-layer primitives such as normalizeText and uniqueSorted.",
    );

    await expectRuleMessage(
      "src/search/runtime-search.ts",
      'import { clampLimit, clampOffset, bigramDice } from "../shared/utils.js";\nexport { clampLimit, clampOffset, bigramDice };\n',
      "Non-tag code must import owner-specific helpers from their owning module instead of src/shared/utils.js. Keep shared/utils limited to true cross-layer primitives such as normalizeText and uniqueSorted.",
    );

    await expectNoRuleMessages(
      "src/search/runtime-search.ts",
      'import { normalizeText, uniqueSorted } from "../shared/utils.js";\nexport { normalizeText, uniqueSorted };\n',
    );
  });

  it("limits DatabaseSync construction to the app storage boundary", async () => {
    const disallowedAppMessages = lintMessageTexts(
      await lintRuleMessages(
        "src/app/ontology-service.ts",
        'import { DatabaseSync } from "node:sqlite";\nexport const db = new DatabaseSync(":memory:");\n',
        "arch/no-direct-database-sync-construction",
      ),
    );
    const disallowedTuiMessages = lintMessageTexts(
      await lintRuleMessages(
        "src/tui/app-services.ts",
        'import { DatabaseSync } from "node:sqlite";\nexport const db = new DatabaseSync(":memory:");\n',
        "arch/no-direct-database-sync-construction",
      ),
    );
    const allowedStorageMessages = await lintRuleMessages(
      "src/app/storage-service.ts",
      'import { DatabaseSync } from "node:sqlite";\nexport const db = new DatabaseSync(":memory:");\n',
      "arch/no-direct-database-sync-construction",
    );

    expect(
      disallowedAppMessages.some((message) =>
        message.includes("Do not construct DatabaseSync here. Open SQLite connections only in approved composition"),
      ),
    ).toBe(true);
    expect(
      disallowedTuiMessages.some((message) =>
        message.includes("Do not construct DatabaseSync here. Open SQLite connections only in approved composition"),
      ),
    ).toBe(true);
    expect(allowedStorageMessages).toHaveLength(0);
  });

  it("allows designated tag UI entrypoints to import tui modules", async () => {
    const reviewUiMessages = await lintRuleMessages(
      "src/tags/editorial/ui/review-ui.tsx",
      'import { TerminalPaneScreen } from "../../../tui/terminal-ui.js";\nexport const Screen = TerminalPaneScreen;\n',
    );
    const reviewModelMessages = await lintRuleMessages(
      "src/tags/editorial/ui/review-screen-model.ts",
      'import { getTerminalPaneBodyHeight } from "../../../tui/terminal-ui.js";\nexport const value = getTerminalPaneBodyHeight;\n',
    );
    const reviewStateMessages = await lintRuleMessages(
      "src/tags/editorial/ui/review-screen-state.ts",
      'import { reduceDerivedTagTerminalTwoPaneState } from "../../../tui/two-pane-state.js";\nexport const value = reduceDerivedTagTerminalTwoPaneState;\n',
    );
    const workbenchMessages = await lintRuleMessages(
      "src/tags/cli/editorial/derived-tag-migration-workbench.ts",
      'import { runPf2eTerminalApp } from "../../tui/pf2e-app.js";\nexport const run = runPf2eTerminalApp;\n',
    );

    expect(reviewUiMessages).toHaveLength(0);
    expect(reviewModelMessages).toHaveLength(0);
    expect(reviewStateMessages).toHaveLength(0);
    expect(workbenchMessages).toHaveLength(0);
  });

  it("keeps review-screen support modules on framework-style tui import limits", async () => {
    const messages = lintMessageTexts(
      await lintRuleMessages(
        "src/tags/editorial/ui/review-screen-model.ts",
        'import { useInput } from "ink";\nexport const value = useInput;\n',
      ),
    );

    expect(
      messages.some((message) =>
        message.includes(
          "TUI feature code must use terminal-ui helpers instead of importing Ink runtime primitives directly.",
        ),
      ),
    ).toBe(true);
  });

  it("exempts framework modules from feature-level direct terminal routing lint", async () => {
    const messages = await lintRuleMessages(
      "src/tui/framework/input.ts",
      'if (event.systemAction === "interrupt") {\n  exitApp();\n}\n',
      "arch/no-direct-terminal-event-routing",
    );

    expect(messages).toHaveLength(0);
  });

  it("blocks tags CLI modules from bypassing the shared search-scope parser and allows the shared parser module", async () => {
    await expectRuleMessage(
      "src/tags/cli/evaluation/evaluate-movement.ts",
      'import { normalizeSearchCategory } from "../../domain/categories.js";\nexport const value = normalizeSearchCategory;\n',
      "CLI scope parsing must go through src/tags/cli/shared/search-scope-args.ts instead of ad hoc category normalization helpers.",
    );

    await expectNoRuleMessages(
      "src/tags/cli/editorial/create-derived-tag-migration-session.ts",
      'import { parseOptionalSearchCategoryArg } from "../shared/search-scope-args.js";\nexport const value = parseOptionalSearchCategoryArg;\n',
    );
  });

  it("blocks search modules from importing storage internals directly", async () => {
    const messages = lintMessageTexts(
      await lintRuleMessages(
        "src/search/ranking.ts",
        'import { rowToRecord } from "../data/rows.js";\nexport const value = rowToRecord;\n',
      ),
    );

    expect(
      messages.some((message) =>
        message.includes(
          "Search modules must depend on Pf2eDataService or another higher-level facade instead of storage rows/query/schema internals.",
        ),
      ),
    ).toBe(true);
  });

  it("blocks server modules from importing low-level search and storage internals", async () => {
    await expectRuleMessage(
      "src/server/presenters.ts",
      'import { buildSearchWhereClause } from "../search/sql.js";\nexport const value = buildSearchWhereClause;\n',
      "Server tool registration must depend on Pf2eDataService or higher-level services, not low-level SQL/query internals.",
    );
  });

  it("blocks runtime-search from importing storage internals after the search decoupling", async () => {
    const messages = lintMessageTexts(
      await lintRuleMessages(
        "src/search/runtime-search.ts",
        'import { rowToRecord } from "../data/rows.js";\nexport const value = rowToRecord;\n',
      ),
    );

    expect(
      messages.some((message) =>
        message.includes(
          "Search modules must depend on Pf2eDataService or another higher-level facade instead of storage rows/query/schema internals.",
        ),
      ),
    ).toBe(true);
  });

  it("blocks search workflows from calling terminal prompt APIs directly and allows shared prompt adapters", async () => {
    await expectRuleMessage(
      "src/tui/search-screen/workspace-actions.ts",
      'async function run() { await terminal.promptTextInput({ label: "Query" }); }\nexport { run };\n',
      "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
      "no-restricted-syntax",
    );

    await expectRuleMessage(
      "src/tui/search-screen/session-workflow.ts",
      'async function run() { await terminal.showDialog({ title: "Help" }); }\nexport { run };\n',
      "Search workflows must use the shared prompt adapter boundary instead of calling terminal.showDialog directly.",
      "no-restricted-syntax",
    );

    await expectNoRuleMessages(
      "src/tui/search-screen/workspace-actions.ts",
      'async function run() { await prompts.promptTextInput({ label: "Query" }); }\nexport { run };\n',
      "no-restricted-syntax",
    );
  });

  it("blocks editorial workbench controllers from reclaiming direct terminal prompt ownership", async () => {
    await expectRuleMessage(
      "src/tags/editorial/ui/workbench-controller.ts",
      'async function run() { await terminal.promptTextInput({ label: "Category" }); }\nexport { run };\n',
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal prompt APIs directly.",
      "no-restricted-syntax",
    );

    await expectRuleMessage(
      "src/tags/editorial/ui/workbench-controller.ts",
      'async function run() { await terminal.showDialog({ title: "Workbench" }); }\nexport { run };\n',
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal.showDialog directly.",
      "no-restricted-syntax",
    );

    await expectNoRuleMessages(
      "src/tags/editorial/ui/workbench-controller.ts",
      'async function run() { await promptDerivedTagMigrationWorkbenchSessionOptions(prompts, db, mode); }\nexport { run };\n',
      "no-restricted-syntax",
    );
  });

  it("blocks direct Ink imports in TUI feature modules that also have tactical restrictions", async () => {
    const messages = lintMessageTexts(
      await lintRuleMessages(
        "src/tui/search-screen/controller.ts",
        'import { useInput } from "ink";\nexport const value = useInput;\n',
      ),
    );

    expect(
      messages.some((message) =>
        message.includes(
          "TUI feature code must use terminal-ui helpers instead of importing Ink runtime primitives directly.",
        ),
      ),
    ).toBe(true);
  });

  it("blocks reintroducing the retired facet-picker bridge model alongside the package-level boundaries", async () => {
    const messages = lintMessageTexts(
      await lintRuleMessages(
        "src/tui/search-screen/controller.ts",
        'import { buildSearchFacetPickerModel } from "../ontology-explorer/facet-picker-model.js";\nexport const value = buildSearchFacetPickerModel;\n',
      ),
    );

    expect(
      messages.some((message) =>
        message.includes(
          "Search screen controllers must use shared ontology-to-search workflow modules instead of constructing facet-picker bridge models directly.",
        ),
      ),
    ).toBe(true);
  });

  it("blocks render-time ontology loading in the app host and legacy launch flags in TUI route flows", async () => {
    await expectRuleMessage(
      "src/tui/pf2e-app.tsx",
      'const value = services.user.ontology.loadSearchSemanticsDomain();\nexport { value };\n',
      "Pf2eTerminalApp must not load ontology route data during render. Prepare Search Semantics routes in the navigation layer before commit.",
      "arch/no-pf2e-app-render-time-ontology-load",
    );

    await expectRuleMessage(
      "src/tui/filter-explorer/controller.ts",
      'const query = { openInResults: true };\nexport { query };\n',
      "TUI ontology/search launch flows must use explicit launch intents instead of openInResults-style route flags.",
      "no-restricted-syntax",
    );
  });

  it("blocks search-screen route bootstrap patterns while allowing user-triggered execute actions", async () => {
    await expectRuleMessage(
      "src/tui/search-screen/session-workflow.ts",
      'const autoExecuteInitialQuery = true;\nexport { autoExecuteInitialQuery };\n',
      "Search session workflow must not restore route-entry auto-execute flags. Navigation owns prepared result launches.",
      "no-restricted-syntax",
    );

    await expectRuleMessage(
      "src/tui/search-screen/session-workflow.ts",
      'function run() { executeRequest(query); }\nexport { run };\n',
      "Search session workflow must not auto-execute route-entry queries locally. Prepare result-reader routes in navigation before commit.",
      "no-restricted-syntax",
    );

    await expectNoRuleMessages(
      "src/tui/search-screen/workspace-actions.ts",
      'function run() { executeRequest(query); }\nexport { run };\n',
      "no-restricted-syntax",
    );
  });
});
