import js from "@eslint/js";
import prettierConfig from "eslint-config-prettier";
import { defineConfig } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";
import localRules from "./eslint-local-rules.js";

export default defineConfig(
  {
    ignores: [".cache/**", ".codex/**", "coverage/**", "dist/**", "node_modules/**", "scratch/**", "vendor/pf2e/**"],
  },
  {
    files: ["**/*.{js,mjs,cjs,ts,tsx}"],
    languageOptions: {
      globals: {
        ...globals.node,
      },
    },
    plugins: {
      arch: localRules,
    },
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      parserOptions: {
        projectService: {
          allowDefaultProject: ["vitest.config.ts"],
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "@typescript-eslint/no-unnecessary-type-assertion": "error",
      "@typescript-eslint/restrict-template-expressions": "error",
      "@typescript-eslint/no-redundant-type-constituents": "error",
      "@typescript-eslint/unbound-method": "error",
      "@typescript-eslint/switch-exhaustiveness-check": "error",
      "@typescript-eslint/require-await": "error",
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-misused-promises": "error",
      "@typescript-eslint/await-thenable": "error",
      "@typescript-eslint/only-throw-error": "error",
      "@typescript-eslint/prefer-promise-reject-errors": "error",
      "@typescript-eslint/no-base-to-string": "error",
      "@typescript-eslint/restrict-plus-operands": "error",
      "@typescript-eslint/no-unsafe-assignment": "error",
      "@typescript-eslint/no-unsafe-member-access": "error",
      "@typescript-eslint/no-unsafe-return": "error",
      "@typescript-eslint/no-unnecessary-condition": "error",
    },
  },
  {
    files: ["**/*.{ts,tsx}"],
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    rules: {
      "no-useless-assignment": "error",
      "preserve-caught-error": "error",
    },
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    rules: {
      "arch/no-direct-json-parse": "error",
      "arch/no-direct-database-sync-construction": "error",
      "arch/no-search-category-assertion": "error",
    },
  },
  {
    files: ["src/tui/**/*.{ts,tsx}", "src/tags/migration/review-ui.tsx"],
    ignores: [
      "src/tui/keymap.ts",
      "src/tui/interaction-bindings.ts",
      "src/tui/action-target.ts",
      "src/tui/terminal-ui.tsx",
    ],
    rules: {
      "arch/no-direct-terminal-event-routing": "error",
    },
  },
  {
    files: ["src/tui/area-menu-screen.tsx", "src/tui/ontology-explorer/domain-picker-screen.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "./terminal-ui.js",
              importNames: [
                "createDerivedTagTerminalListNavigationState",
                "getTerminalPaneBodyHeight",
                "resolveDerivedTagTerminalListNavigationAction",
                "useDerivedTagTerminalApp",
                "useDerivedTagTerminalInput",
                "useDerivedTagTerminalSize",
              ],
              message:
                "Simple chooser screens must use the shared TerminalMenuScreen abstraction instead of owning input/navigation loops.",
            },
            {
              name: "../terminal-ui.js",
              importNames: [
                "createDerivedTagTerminalListNavigationState",
                "getTerminalPaneBodyHeight",
                "resolveDerivedTagTerminalListNavigationAction",
                "useDerivedTagTerminalApp",
                "useDerivedTagTerminalInput",
                "useDerivedTagTerminalSize",
              ],
              message:
                "Simple chooser screens must use the shared TerminalMenuScreen abstraction instead of owning input/navigation loops.",
            },
            {
              name: "./interaction-bindings.js",
              importNames: ["resolveTerminalInteractionAction"],
              message:
                "Simple chooser screens must use the shared TerminalMenuScreen abstraction instead of resolving interaction actions directly.",
            },
            {
              name: "../interaction-bindings.js",
              importNames: ["resolveTerminalInteractionAction"],
              message:
                "Simple chooser screens must use the shared TerminalMenuScreen abstraction instead of resolving interaction actions directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/pf2e-app.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "./terminal-ui.js",
              importNames: ["TerminalTextScreen", "useDerivedTagTerminalInput"],
              message:
                "Static informational TUI screens must use the shared TerminalMessageScreen abstraction instead of owning input or TerminalTextScreen composition directly.",
            },
            {
              name: "./interaction-bindings.js",
              importNames: ["formatTerminalInteractionFooter", "resolveTerminalInteractionAction"],
              message:
                "Static informational TUI screens must use the shared TerminalMessageScreen abstraction instead of resolving their own footer or interaction actions.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/tag-refinement-menu-screen.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "./terminal-ui.js",
              importNames: [
                "TerminalTwoPaneScreen",
                "createDerivedTagTerminalListNavigationState",
                "getTerminalPaneBodyHeight",
                "resolveDerivedTagTerminalListNavigationAction",
                "useDerivedTagTerminalApp",
                "useDerivedTagTerminalInput",
                "useDerivedTagTerminalSize",
              ],
              message:
                "Tag refinement menus must use the shared TerminalActionMenuScreen abstraction instead of owning list/input plumbing.",
            },
            {
              name: "./interaction-bindings.js",
              importNames: ["formatTerminalInteractionFooter", "resolveTerminalInteractionAction"],
              message:
                "Tag refinement menus must use the shared TerminalActionMenuScreen abstraction instead of resolving footer or interaction actions directly.",
            },
            {
              name: "./action-target.js",
              importNames: [
                "buildDerivedTagTerminalActionTargetLine",
                "createDerivedTagTerminalActionTargetState",
                "getDerivedTagTerminalActionTargetInteractionActions",
                "reduceDerivedTagTerminalActionTargetState",
                "resolveDerivedTagTerminalActionTargetIntent",
                "shouldRenderDerivedTagTerminalActionTarget",
              ],
              message:
                "Tag refinement menus must use the shared TerminalActionMenuScreen abstraction instead of owning action-target state and rendering directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/app/ontology-service.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector:
            'ObjectExpression:has(Property[key.name="kind"][value.value="example"]):has(Property[key.name="detailTitle"][value.value="Example Predicate"])',
          message:
            "Search semantics must not emit curated standalone example nodes. Fold that support into final shared semantics surfaces instead.",
        },
        {
          selector:
            'ObjectExpression:has(Property[key.name="kind"][value.value="group"]):has(Property[key.name="label"][value.value="Examples"]):has(Property[key.name="detailTitle"][value.value="Category Examples"])',
          message: "Search semantics must not emit an Examples browse group.",
        },
      ],
    },
  },
  {
    files: ["src/tags/migration/review-ui.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "../../tui/terminal-ui.js",
              importNames: ["useDerivedTagTerminalInput"],
              message:
                "Review render screens must use the dedicated review-ui-controller hook instead of owning raw terminal input.",
            },
            {
              name: "../../tui/interaction-bindings.js",
              importNames: [
                "buildTerminalInteractionHelpLines",
                "formatTerminalInteractionFooter",
                "resolveTerminalInteractionAction",
              ],
              message:
                "Review render screens must use the dedicated review-ui-controller hook instead of composing help, footer, or interaction routing directly.",
            },
            {
              name: "../../tui/action-target.js",
              importNames: [
                "buildDerivedTagTerminalActionTargetLine",
                "buildDerivedTagTerminalActionTargetHelpLines",
                "createDerivedTagTerminalActionTargetState",
                "getDerivedTagTerminalActionTargetInteractionActions",
                "reduceDerivedTagTerminalActionTargetState",
                "resolveDerivedTagTerminalActionTargetIntent",
              ],
              message:
                "Review render screens must use the dedicated review-ui-controller hook instead of owning action-target state or rendering directly.",
            },
            {
              name: "../../tui/two-pane-state.js",
              importNames: ["getDerivedTagTerminalTwoPaneLayoutMode", "reduceDerivedTagTerminalTwoPaneState"],
              message:
                "Review render screens must use the dedicated review-ui-controller hook instead of owning pane-state transitions directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/ontology-explorer/screen.tsx", "src/tui/ontology-explorer/picker-screen.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "../interaction-bindings.js",
              importNames: [
                "buildTerminalInteractionHelpLines",
                "formatTerminalInteractionFooter",
                "TerminalInteractionAction",
              ],
              message:
                "Ontology render screens must use shared screen-model helpers instead of composing local help/footer/action tables.",
            },
            {
              name: "../terminal-ui.js",
              importNames: ["DerivedTagTerminalCommandOption"],
              message:
                "Ontology render screens must use shared screen-model helpers instead of defining command-palette models locally.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/ontology-explorer/controller.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector:
            'ImportDeclaration[source.value="../terminal-ui.js"] ImportSpecifier[imported.name="createDerivedTagTerminalListNavigationState"]',
          message:
            "Ontology explorer controllers must use a shared ontology interaction helper instead of owning terminal input or list-navigation routing directly.",
        },
        {
          selector:
            'ImportDeclaration[source.value="../terminal-ui.js"] ImportSpecifier[imported.name="resolveDerivedTagTerminalListNavigationAction"]',
          message:
            "Ontology explorer controllers must use a shared ontology interaction helper instead of owning terminal input or list-navigation routing directly.",
        },
        {
          selector:
            'ImportDeclaration[source.value="../terminal-ui.js"] ImportSpecifier[imported.name="useDerivedTagTerminalInput"]',
          message:
            "Ontology explorer controllers must use a shared ontology interaction helper instead of owning terminal input or list-navigation routing directly.",
        },
        {
          selector:
            'ImportDeclaration[source.value="../interaction-bindings.js"] ImportSpecifier[imported.name="resolveTerminalInteractionAction"]',
          message:
            "Ontology explorer controllers must use a shared ontology interaction helper instead of resolving raw interaction or text-entry intents directly.",
        },
        {
          selector:
            'ImportDeclaration[source.value="../interaction-bindings.js"] ImportSpecifier[imported.name="resolveTerminalTextEntryIntent"]',
          message:
            "Ontology explorer controllers must use a shared ontology interaction helper instead of resolving raw interaction or text-entry intents directly.",
        },
      ],
    },
  },
  {
    files: ["src/tui/ontology-explorer/interactions.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "../terminal-ui.js",
              importNames: [
                "createDerivedTagTerminalListNavigationState",
                "resolveDerivedTagTerminalListNavigationAction",
                "useDerivedTagTerminalInput",
              ],
              message:
                "Ontology interaction routers must use the shared interaction-context router instead of raw list-navigation or input primitives.",
            },
            {
              name: "../interaction-bindings.js",
              importNames: ["resolveTerminalInteractionAction", "resolveTerminalTextEntryIntent"],
              message:
                "Ontology interaction routers must use the shared interaction-context router instead of resolving raw interaction or text-entry intents directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/terminal-ui.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "./interaction-bindings.js",
              importNames: ["getTerminalInteractionCycleDirection", "resolveTerminalInteractionAction"],
              message:
                "Terminal modal routing must go through the shared interaction-context router and prompt-context adapters instead of resolving raw actions directly.",
            },
          ],
        },
      ],
      "no-restricted-syntax": [
        "error",
        {
          selector: 'Literal[value="Type to filter  Backspace edit  Esc cancel"]',
          message:
            "Do not hardcode command-palette filter footer bindings in terminal-ui. Derive prompt footer text from shared interaction helpers or exported footer constants instead.",
        },
        {
          selector: 'Literal[value="Esc/backspace/left/q cancel"]',
          message:
            "Do not hardcode prompt cancel footer bindings in terminal-ui. Derive prompt footer text from shared interaction helpers or exported footer constants instead.",
        },
      ],
    },
  },
  {
    files: ["src/tui/search-screen-controller.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: 'CallExpression[callee.type="MemberExpression"][callee.property.name="showDialog"]',
          message:
            "Search help dialogs must go through showTerminalReturnDialog instead of calling showDialog directly.",
        },
      ],
    },
  },
  {
    files: ["src/tui/search-screen-session-workflow.ts", "src/tui/search-screen-workspace-actions.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptCommandPalette"]',
          message:
            "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
        },
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptOptionalSelectOption"]',
          message:
            "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
        },
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptPolicySelectOption"]',
          message:
            "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
        },
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptMultiSelectOption"]',
          message:
            "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
        },
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptSelectOption"]',
          message:
            "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
        },
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptTextInput"]',
          message:
            "Search workflows must use the shared prompt adapter boundary instead of calling terminal prompt APIs directly.",
        },
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="showDialog"]',
          message:
            "Search workflows must use the shared prompt adapter boundary instead of calling terminal.showDialog directly.",
        },
      ],
    },
  },
  {
    files: ["src/tui/ontology-explorer/screen.tsx", "src/tui/ontology-explorer/picker-screen.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "../terminal-ui.js",
              importNames: ["useDerivedTagTerminalApp"],
              message:
                "Ontology screens must use the shared prompt adapter hook instead of reaching directly into the terminal app for prompts or dialogs.",
            },
            {
              name: "../interaction-bindings.js",
              importNames: ["TERMINAL_DIALOG_RETURN_FOOTER"],
              message:
                "Ontology screens must use the shared return-dialog helper instead of composing the return footer locally.",
            },
          ],
        },
      ],
      "no-restricted-syntax": [
        "error",
        {
          selector: 'CallExpression[callee.type="MemberExpression"][callee.property.name="showDialog"]',
          message: "Ontology screens must use showTerminalReturnDialog instead of calling showDialog directly.",
        },
      ],
    },
  },
  {
    files: ["src/tui/**/*.{ts,tsx}"],
    ignores: [
      "src/tui/keymap.ts",
      "src/tui/interaction-bindings.ts",
      "src/tui/action-target.ts",
      "src/tui/terminal-ui.tsx",
    ],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "ink",
              allowTypeImports: true,
              message:
                "TUI feature code must use terminal-ui helpers instead of importing Ink runtime primitives directly.",
            },
          ],
          patterns: [
            {
              group: ["**/keymap.js"],
              message:
                "TUI feature code must use interaction-bindings, action-target, or terminal-ui helpers instead of importing keymap directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/**/*.{ts,tsx}"],
    ignores: ["src/tui/app-services.ts", "src/tui/ontology-explorer/data.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "node:sqlite",
              message:
                "TUI modules must not open SQLite directly. Go through app-services or approved cache/composition modules.",
            },
          ],
          patterns: [
            {
              group: ["**/data/service.js", "**/app/runtime.js", "**/app/ontology-service.js"],
              message:
                "TUI modules must consume composed app services instead of constructing runtime or data services directly.",
            },
            {
              group: [
                "**/tags/migration/workbench-controller.js",
                "**/tags/migration/session-builder.js",
                "**/tags/migration/runtime-state.js",
                "**/tags/migration/session-store.js",
                "**/tags/migration/cli-utils.js",
              ],
              message:
                "TUI modules must use app-services for tag-workbench composition instead of importing migration service internals directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    ignores: ["src/tags/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: [
                "**/tags/runtime/**",
                "**/tags/authored-rules/**",
                "**/tags/catalog/**",
                "**/tags/ontology/**",
                "**/tags/exemplars/**",
                "**/tags/legacy-rules/**",
                "**/tags/legacy-seed-migrations/**",
              ],
              message:
                "Outside src/tags, import derived-tag functionality through src/tags/index.js or another approved facade instead of leaf tag internals.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tags/cli/**/*.{ts,tsx}"],
    ignores: ["src/tags/cli/search-scope-args.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["**/domain/categories.js", "**/data/sql-row-decoding.js"],
              message:
                "CLI scope parsing must go through src/tags/cli/search-scope-args.ts instead of ad hoc category normalization helpers.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/server/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["**/search/sql.js", "**/data/record-queries.js", "**/data/schema.js"],
              message:
                "Server tool registration must depend on Pf2eDataService or higher-level services, not low-level SQL/query internals.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/search-screen-controller.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "./ontology-explorer/facet-picker-model.js",
              importNames: ["buildSearchFacetPickerModel"],
              message:
                "Search screen controllers must use shared ontology-to-search workflow modules instead of constructing facet-picker bridge models directly.",
            },
            {
              name: "./terminal-ui.js",
              importNames: [
                "createDerivedTagTerminalListNavigationState",
                "resolveDerivedTagTerminalListNavigationAction",
                "useDerivedTagTerminalInput",
              ],
              message:
                "Search screen controllers must use the shared search interaction router instead of owning raw terminal event decoding.",
            },
            {
              name: "./interaction-bindings.js",
              importNames: ["TERMINAL_DIALOG_RETURN_FOOTER", "resolveTerminalInteractionAction"],
              message:
                "Search screen controllers must use shared search interaction and help-dialog helpers instead of resolving raw interaction actions or composing return footers directly.",
            },
            {
              name: "./search-screen-model.js",
              importNames: [
                "SearchFacetPickerSession",
                "applyFacetPickerSelectionsToRequest",
                "buildFacetPickerInitialSelections",
              ],
              message:
                "Search screen controllers must use shared ontology-to-search workflow modules instead of owning facet-picker session bridging directly.",
            },
            {
              name: "./search-screen-model.js",
              importNames: [
                "LIVE_COUNT_DEBOUNCE_MS",
                "RESULT_WINDOW_FETCH_DEBOUNCE_MS",
                "getSearchResultWindowMetrics",
                "getSearchResultWindowTarget",
                "getSessionBufferRange",
                "getSessionRecordAtIndex",
              ],
              message:
                "Search screen controllers must use shared result-window/session workflow modules instead of orchestrating buffer windows or session reads directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/search-screen-interactions.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "./terminal-ui.js",
              importNames: [
                "createDerivedTagTerminalListNavigationState",
                "resolveDerivedTagTerminalListNavigationAction",
                "useDerivedTagTerminalInput",
              ],
              message:
                "Search interaction routers must use the shared interaction-context router instead of raw list-navigation or input primitives.",
            },
            {
              name: "./interaction-bindings.js",
              importNames: ["resolveTerminalInteractionAction"],
              message:
                "Search interaction routers must use the shared interaction-context router instead of resolving raw interaction actions directly.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/tui/search-screen*.ts", "src/tui/search-screen*.tsx"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: 'Literal[value="draft"]',
          message:
            "Search screen terminology is query/editor, not draft. Keep the final editor/query naming consistent in this feature.",
        },
        {
          selector: 'Literal[value="setup"]',
          message:
            "Search screen terminology is query/editor, not setup. Keep the final editor/query naming consistent in this feature.",
        },
        {
          selector: 'Literal[value="openSetup"]',
          message:
            "Search result commands must route back through the query editor using final editor/query naming, not setup wording.",
        },
      ],
    },
  },
  {
    files: ["src/tui/shared-screens.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "./terminal-ui.js",
              importNames: [
                "createDerivedTagTerminalListNavigationState",
                "resolveDerivedTagTerminalListNavigationAction",
                "useDerivedTagTerminalApp",
                "useDerivedTagTerminalInput",
              ],
              message:
                "Shared screen primitives must use the shared interaction-context router and prompt adapter hook instead of owning raw list-navigation, input routing, or direct dialog access.",
            },
            {
              name: "./interaction-bindings.js",
              importNames: ["TERMINAL_DIALOG_RETURN_FOOTER", "resolveTerminalInteractionAction"],
              message:
                "Shared screen primitives must use shared interaction and return-dialog helpers instead of resolving raw interaction actions or composing dialog footers directly.",
            },
          ],
        },
      ],
      "no-restricted-syntax": [
        "error",
        {
          selector: 'CallExpression[callee.type="MemberExpression"][callee.property.name="showDialog"]',
          message: "Shared screens must use showTerminalReturnDialog instead of calling showDialog directly.",
        },
      ],
    },
  },
  prettierConfig,
);
