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
  prettierConfig,
);
