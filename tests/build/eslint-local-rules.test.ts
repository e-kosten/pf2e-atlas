import { RuleTester } from "eslint";
import { describe, it } from "vitest";

import localRules from "../../eslint-local-rules.js";

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
});
