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
  });

  it("blocks DatabaseSync construction outside approved entry modules and allows composition roots", async () => {
    await expectRuleMessage(
      "src/data/service.ts",
      "const db = new DatabaseSync(path);\nexport { db };\n",
      "Open SQLite connections only in approved composition or entry modules.",
      "arch/no-direct-database-sync-construction",
    );

    await expectNoRuleMessages(
      "src/tags/cli/evaluate-movement.ts",
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
        filePath: "src/tags/runtime/index.ts",
        code: 'import { useDerivedTagTerminalInput } from "../../tui/terminal-ui.js";\nexport const value = useDerivedTagTerminalInput;\n',
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

    await expectNoRuleMessages(
      "src/app/runtime.ts",
      'import { normalizeDerivedTag } from "../tags/index.js";\nexport const value = normalizeDerivedTag;\n',
    );
  });

  it("allows designated tag UI entrypoints to import tui modules", async () => {
    const reviewUiMessages = await lintRuleMessages(
      "src/tags/migration/review-ui.tsx",
      'import { TerminalPaneScreen } from "../../tui/terminal-ui.js";\nexport const Screen = TerminalPaneScreen;\n',
    );
    const reviewModelMessages = await lintRuleMessages(
      "src/tags/migration/review-screen-model.ts",
      'import { getTerminalPaneBodyHeight } from "../../tui/terminal-ui.js";\nexport const value = getTerminalPaneBodyHeight;\n',
    );
    const reviewStateMessages = await lintRuleMessages(
      "src/tags/migration/review-screen-state.ts",
      'import { reduceDerivedTagTerminalTwoPaneState } from "../../tui/two-pane-state.js";\nexport const value = reduceDerivedTagTerminalTwoPaneState;\n',
    );
    const workbenchMessages = await lintRuleMessages(
      "src/tags/cli/derived-tag-migration-workbench.ts",
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
        "src/tags/migration/review-screen-model.ts",
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
      "src/tags/cli/evaluate-movement.ts",
      'import { normalizeSearchCategory } from "../../domain/categories.js";\nexport const value = normalizeSearchCategory;\n',
      "CLI scope parsing must go through src/tags/cli/search-scope-args.ts instead of ad hoc category normalization helpers.",
    );

    await expectNoRuleMessages(
      "src/tags/cli/create-derived-tag-migration-session.ts",
      'import { parseOptionalSearchCategoryArg } from "./search-scope-args.js";\nexport const value = parseOptionalSearchCategoryArg;\n',
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
      "src/tui/search-screen-workspace-actions.ts",
      'async function run() { await terminal.promptTextInput({ label: "Query" }); }\nexport { run };\n',
      "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
      "no-restricted-syntax",
    );

    await expectRuleMessage(
      "src/tui/search-screen-session-workflow.ts",
      'async function run() { await terminal.showDialog({ title: "Help" }); }\nexport { run };\n',
      "Search workflows must use the shared prompt adapter boundary instead of calling terminal.showDialog directly.",
      "no-restricted-syntax",
    );

    await expectNoRuleMessages(
      "src/tui/search-screen-workspace-actions.ts",
      'async function run() { await prompts.promptTextInput({ label: "Query" }); }\nexport { run };\n',
      "no-restricted-syntax",
    );
  });

  it("blocks direct Ink imports in TUI feature modules that also have tactical restrictions", async () => {
    const messages = lintMessageTexts(
      await lintRuleMessages(
        "src/tui/search-screen-controller.ts",
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

  it("retains existing tactical search-screen controller restrictions alongside the package-level boundaries", async () => {
    const messages = lintMessageTexts(
      await lintRuleMessages(
        "src/tui/search-screen-controller.ts",
        'import { buildSearchFacetPickerModel } from "./ontology-explorer/facet-picker-model.js";\nexport const value = buildSearchFacetPickerModel;\n',
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
});
