import { RuleTester } from "eslint";
import { describe, expect, it } from "vitest";

import localRules from "../../eslint-local-rules.js";
import { lintMessageTexts, lintRuleMessages } from "./eslint-config-test-helpers.js";

type TestedRuleModule = Parameters<RuleTester["run"]>[1];

const typedLocalRules = localRules as unknown as { rules: Record<string, TestedRuleModule> };
const directTerminalEventRoutingRule = typedLocalRules.rules["no-direct-terminal-event-routing"];

const ruleTester = new RuleTester({
  languageOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
  },
});

describe("eslint local architecture rules", () => {
  it("blocks direct terminal event routing in feature code", () => {
    ruleTester.run("no-direct-terminal-event-routing", directTerminalEventRoutingRule, {
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
