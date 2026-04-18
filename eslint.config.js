import js from "@eslint/js";
import prettierConfig from "eslint-config-prettier";
import { defineConfig } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";

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
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      parserOptions: {
        project: "./tsconfig.eslint.json",
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "@typescript-eslint/no-unnecessary-type-assertion": "error",
      "@typescript-eslint/restrict-template-expressions": "error",
      "@typescript-eslint/no-redundant-type-constituents": "error",
      "@typescript-eslint/unbound-method": "error",
      "@typescript-eslint/require-await": "error",
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-misused-promises": "error",
      "@typescript-eslint/await-thenable": "error",
      "@typescript-eslint/only-throw-error": "error",
      "@typescript-eslint/prefer-promise-reject-errors": "error",
      "@typescript-eslint/no-base-to-string": "error",
      "@typescript-eslint/restrict-plus-operands": "error",
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
  prettierConfig,
);
