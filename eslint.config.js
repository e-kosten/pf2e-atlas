import js from "@eslint/js";
import prettierConfig from "eslint-config-prettier";
import { defineConfig } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";
import localRules from "./eslint-local-rules.js";

function mergeRestrictedImports(...restrictions) {
  const paths = restrictions.flatMap((restriction) => restriction.paths ?? []);
  const patterns = restrictions.flatMap((restriction) => restriction.patterns ?? []);

  return [
    "error",
    {
      ...(paths.length > 0 ? { paths } : {}),
      ...(patterns.length > 0 ? { patterns } : {}),
    },
  ];
}

function buildRelativeImportNameRestrictions(pathVariants, importNames, message) {
  return pathVariants.map((name) => ({
    name,
    importNames,
    message,
  }));
}

function buildImportSourceSyntaxRestrictions(pathVariants, message) {
  return pathVariants.map((name) => ({
    selector: `ImportDeclaration[source.value="${name}"]`,
    message,
  }));
}

const SEARCH_LIST_DETAIL_PRESENTATION_IMPORT_PATHS = [
  "../framework/line-rendering.js",
  "../framework/screen-layout.js",
  "../route-transition-status.js",
  "../interaction-context-router.js",
];
const REVIEW_LIST_DETAIL_PRESENTATION_IMPORT_PATHS = [
  "../../../tui/framework/line-rendering.js",
  "../../../tui/framework/screen-layout.js",
  "../../../tui/route-transition-status.js",
  "../../../tui/interaction-context-router.js",
];

const DOMAIN_INDEX_IMPORT_PATHS = ["./domain/index.js", "../domain/index.js", "../../domain/index.js"];
const SHARED_UTILS_IMPORT_PATHS = ["./shared/utils.js", "../shared/utils.js", "../../shared/utils.js"];
const SEARCH_CONTRACT_IMPORT_PATHS = [
  "./search/contracts.js",
  "../search/contracts.js",
  "../../search/contracts.js",
  "../../../search/contracts.js",
];
const SEARCH_REQUEST_COMPILATION_IMPORT_PATHS = [
  "./search/request-compilation.js",
  "../search/request-compilation.js",
  "../../search/request-compilation.js",
  "../../../search/request-compilation.js",
];
const SEARCH_FILTER_NORMALIZATION_IMPORT_PATHS = [
  "./search/filters/normalization.js",
  "../search/filters/normalization.js",
  "../../search/filters/normalization.js",
  "../../../search/filters/normalization.js",
];

const NON_TAG_SHARED_UTILS_OWNER_IMPORT_NAMES = [
  "bigramDice",
  "clampLimit",
  "clampOffset",
  "firstString",
  "getNested",
  "stripHtml",
  "toStringArray",
];

const NON_UI_TUI_IMPORT_RESTRICTIONS = {
  patterns: [
    {
      group: ["**/tui/**/*.js"],
      message:
        "Non-UI application, data, domain, search, server, and tag modules must not import src/tui internals directly.",
    },
  ],
};

const NON_TAG_OWNERSHIP_IMPORT_RESTRICTIONS = {
  paths: [
    ...buildRelativeImportNameRestrictions(
      SHARED_UTILS_IMPORT_PATHS,
      NON_TAG_SHARED_UTILS_OWNER_IMPORT_NAMES,
      "Non-tag code must import owner-specific helpers from their owning module instead of src/shared/utils.js. Keep shared/utils limited to true cross-layer primitives such as normalizeText and uniqueSorted.",
    ),
  ],
  patterns: [
    {
      group: DOMAIN_INDEX_IMPORT_PATHS,
      message:
        "Non-tag code must import domain contracts from explicit src/domain/* modules instead of the removed broad src/domain/index.js barrel.",
    },
  ],
};

const SEARCH_REQUEST_BOUNDARY_IMPORT_RESTRICTIONS = {
  paths: [
    ...SEARCH_CONTRACT_IMPORT_PATHS.map((name) => ({
      name,
      message:
        "App, domain, server, and TUI modules must use SearchRequest and backend facades instead of importing search execution modules directly.",
    })),
    ...SEARCH_REQUEST_COMPILATION_IMPORT_PATHS.map((name) => ({
      name,
      message:
        "App, domain, server, and TUI modules must not compile SearchRequest values directly. Route semantic queries through Pf2eDataService or the backend search service.",
    })),
    ...SEARCH_FILTER_NORMALIZATION_IMPORT_PATHS.map((name) => ({
      name,
      message:
        "App, domain, server, and TUI modules must not normalize or validate search execution filters directly. Keep that work inside search-owned execution boundaries.",
    })),
  ],
};

const SEARCH_REQUEST_BOUNDARY_SYNTAX_RESTRICTIONS = [
  ...buildImportSourceSyntaxRestrictions(
    SEARCH_CONTRACT_IMPORT_PATHS,
    "App, domain, server, and TUI modules must use SearchRequest and backend facades instead of importing search execution modules directly.",
  ),
  ...buildImportSourceSyntaxRestrictions(
    SEARCH_REQUEST_COMPILATION_IMPORT_PATHS,
    "App, domain, server, and TUI modules must not compile SearchRequest values directly. Route semantic queries through Pf2eDataService or the backend search service.",
  ),
  ...buildImportSourceSyntaxRestrictions(
    SEARCH_FILTER_NORMALIZATION_IMPORT_PATHS,
    "App, domain, server, and TUI modules must not normalize or validate search execution filters directly. Keep that work inside search-owned execution boundaries.",
  ),
];

function mergeNonTagRestrictedImports(...restrictions) {
  return mergeRestrictedImports(NON_TAG_OWNERSHIP_IMPORT_RESTRICTIONS, ...restrictions);
}

const SEARCH_STORAGE_INTERNAL_IMPORT_RESTRICTIONS = {
  patterns: [
    {
      group: ["../data/rows.js", "../data/record-queries.js", "../data/schema.js"],
      message:
        "Search modules must depend on Pf2eDataService or another higher-level facade instead of storage rows/query/schema internals.",
    },
  ],
};

const DOMAIN_SEARCH_INTERNAL_IMPORT_RESTRICTIONS = {
  patterns: [
    {
      group: ["../search/**", "../../search/**", "../../../search/**"],
      message:
        "Domain modules must not import src/search internals. Move shared search semantics into src/domain or invert the dependency through a search-owned execution boundary.",
    },
  ],
};

const DOMAIN_SEARCH_INTERNAL_SYNTAX_RESTRICTIONS = buildImportSourceSyntaxRestrictions(
  [
    "../search/filters/registry.js",
    "../../search/filters/registry.js",
    "../../../search/filters/registry.js",
    "../search/filters/metadata.js",
    "../../search/filters/metadata.js",
    "../../../search/filters/metadata.js",
  ],
  "Domain modules must not import src/search internals. Move shared search semantics into src/domain or invert the dependency through a search-owned execution boundary.",
);

const NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS = {
  paths: [
    {
      name: "ink",
      allowTypeImports: true,
      message: "TUI feature code must use terminal-ui helpers instead of importing Ink runtime primitives directly.",
    },
  ],
  patterns: [
    {
      group: ["**/keymap.js"],
      message:
        "TUI feature code must use interaction-bindings, action-target, or terminal-ui helpers instead of importing keymap directly.",
    },
  ],
};

const TAGS_CLI_SCOPE_IMPORT_RESTRICTIONS = {
  patterns: [
    {
      group: ["**/domain/categories.js", "**/data/sql-row-decoding.js"],
      message:
        "CLI scope parsing must go through src/tags/cli/shared/search-scope-args.ts instead of ad hoc category normalization helpers.",
    },
  ],
};

const NON_TAGS_DERIVED_TAG_IMPORT_RESTRICTIONS = {
  patterns: [
    {
      group: [
        "**/tags/runtime/**",
        "**/tags/rules/**",
        "**/tags/catalog/**",
        "**/tags/ontology/**",
        "**/tags/exemplars/**",
        "**/tags/editorial/**",
        "**/tags/reviews/**",
        "**/tags/legacy-rules/**",
        "**/tags/legacy-seed-migrations/**",
      ],
      message:
        "Outside src/tags, import derived-tag functionality through src/tags/runtime.js, src/tags/editorial.js, or src/tags/editorial-ui.js instead of leaf tag internals.",
    },
  ],
};

const LEGACY_ENTITY_DETAIL_PRESENTER_IMPORT_RESTRICTIONS = {
  patterns: [
    {
      group: ["**/app/ontology/presenter.js"],
      message:
        "Search preview and ontology record-page hosts must consume shared entity-page composition through services.user.entityPages and src/tui/page-document/* instead of importing the plain-line presenter directly.",
    },
  ],
};

const SERVER_STORAGE_INTERNAL_IMPORT_RESTRICTIONS = {
  patterns: [
    {
      group: ["../search/sql.js", "../data/record-queries.js", "../data/schema.js"],
      message:
        "Server tool registration must depend on Pf2eDataService or higher-level services, not low-level SQL/query internals.",
    },
  ],
};

const SEARCH_SCREEN_CONTROLLER_SYNTAX_RESTRICTIONS = [
  {
    selector: 'CallExpression[callee.type="MemberExpression"][callee.property.name="showDialog"]',
    message: "Search help dialogs must go through showTerminalReturnDialog instead of calling showDialog directly.",
  },
];

const SEARCH_WORKFLOW_PROMPT_BOUNDARY_RESTRICTIONS = [
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
];

const EDITORIAL_WORKBENCH_PROMPT_BOUNDARY_RESTRICTIONS = [
  {
    selector:
      'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptCommandPalette"]',
    message:
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal prompt APIs directly.",
  },
  {
    selector:
      'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptOptionalSelectOption"]',
    message:
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal prompt APIs directly.",
  },
  {
    selector:
      'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptPolicySelectOption"]',
    message:
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal prompt APIs directly.",
  },
  {
    selector:
      'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptMultiSelectOption"]',
    message:
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal prompt APIs directly.",
  },
  {
    selector:
      'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptSelectOption"]',
    message:
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal prompt APIs directly.",
  },
  {
    selector:
      'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="promptTextInput"]',
    message:
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal prompt APIs directly.",
  },
  {
    selector:
      'CallExpression[callee.type="MemberExpression"][callee.object.name="terminal"][callee.property.name="showDialog"]',
    message:
      "Editorial workbench controllers must route prompt ownership through workbench-session-prompts instead of calling terminal.showDialog directly.",
  },
];

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
        project: ["./tsconfig.eslint.json"],
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
    files: ["tests/**/*.{ts,tsx}"],
    rules: {
      "@typescript-eslint/require-await": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-return": "off",
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
      "arch/no-internal-tags-barrel-imports": "error",
    },
  },
  {
    files: ["src/tui/**/*.{ts,tsx}", "src/tags/editorial/ui/review-ui.tsx"],
    ignores: [
      "src/tui/framework/**/*.{ts,tsx}",
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
    files: ["src/tui/search-screen/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: 'MemberExpression[property.name="getFacetFieldOptions"]',
          message:
            "Search editor surfaces must use shared query-field selection flows instead of the legacy facet-field API.",
        },
        {
          selector: 'Literal[value="Edit Facet Filter"]',
          message: "Search editor copy must route discoverable filtering through query parts, not facet rows.",
        },
        {
          selector: 'Literal[value="Clear Facet Filter"]',
          message: "Search editor copy must route discoverable filtering through query parts, not facet rows.",
        },
        {
          selector: 'Literal[value="Facet Filters"]',
          message: "Search editor copy must not expose legacy facet-filter group labels.",
        },
        {
          selector:
            'MemberExpression[object.type="MemberExpression"][object.property.name="filters"][property.name="subcategory"]',
          message:
            "Search editor surfaces must read subcategory through shared query-part helpers instead of direct legacy filter slots.",
        },
        {
          selector:
            'MemberExpression[object.type="MemberExpression"][object.property.name="filters"][property.name="levelMin"]',
          message:
            "Search editor surfaces must read level range through shared query-part helpers instead of direct legacy filter slots.",
        },
        {
          selector:
            'MemberExpression[object.type="MemberExpression"][object.property.name="filters"][property.name="levelMax"]',
          message:
            "Search editor surfaces must read level range through shared query-part helpers instead of direct legacy filter slots.",
        },
        {
          selector:
            'MemberExpression[object.type="MemberExpression"][object.property.name="filters"][property.name="rarity"]',
          message:
            "Search editor surfaces must read rarity through shared query-part helpers instead of direct legacy filter slots.",
        },
        {
          selector:
            'MemberExpression[object.type="MemberExpression"][object.property.name="filters"][property.name="actionCost"]',
          message:
            "Search editor surfaces must read action cost through shared query-part helpers instead of direct legacy filter slots.",
        },
        {
          selector:
            'MemberExpression[object.type="MemberExpression"][object.property.name="filters"][property.name="metadata"]',
          message:
            "Search editor surfaces must read query clauses through shared query-part helpers instead of direct legacy filter slots.",
        },
        {
          selector:
            'MemberExpression[object.type="MemberExpression"][object.property.name="filters"][property.name="facets"]',
          message: "Search editor surfaces must not read legacy facet slots; use the unified query-part model instead.",
        },
      ],
    },
  },
  {
    files: ["src/tui/area-menu-screen.tsx", "src/tui/ontology-explorer/domain-picker-screen.tsx"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
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
      }),
    },
  },
  {
    files: ["src/tui/search-screen/structured-draft/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: 'CallExpression[callee.type="MemberExpression"][callee.property.name="applyFilterExplorerDraft"]',
          message:
            "Structured-draft final writeback must route through bounded structured-editor host mutations, not generic filter-explorer draft application.",
        },
      ],
    },
  },
  {
    files: ["src/tui/pf2e-app.tsx"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
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
      }),
    },
  },
  {
    files: ["src/tui/tag-refinement-menu-screen.tsx"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
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
      }),
    },
  },
  {
    files: [
      "src/tui/search-screen/controller.ts",
      "src/tui/search-screen/interactions.ts",
      "src/tui/search-screen/screen.tsx",
      "src/tui/filter-explorer/controller.ts",
      "src/tui/filter-explorer/screen-models.ts",
    ],
    rules: {
      "no-restricted-syntax": [
        "error",
        ...buildImportSourceSyntaxRestrictions(
          SEARCH_LIST_DETAIL_PRESENTATION_IMPORT_PATHS,
          "List/detail search and explorer surfaces must use src/tui/list-detail-presentation.ts instead of bypassing shared measurement, transition-footer composition, or list/detail routing directly.",
        ),
        {
          selector: 'Identifier[name="openInResults"]',
          message:
            "TUI ontology/search launch flows must use explicit launch intents instead of openInResults-style route flags.",
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
    files: ["src/tags/editorial/ui/review-ui.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "../../../tui/terminal-ui.js",
              importNames: ["useDerivedTagTerminalInput"],
              message:
                "Review render screens must use the dedicated review-ui-controller hook instead of owning raw terminal input.",
            },
            {
              name: "../../../tui/interaction-bindings.js",
              importNames: [
                "buildTerminalInteractionHelpLines",
                "formatTerminalInteractionFooter",
                "resolveTerminalInteractionAction",
              ],
              message:
                "Review render screens must use the dedicated review-ui-controller hook instead of composing help, footer, or interaction routing directly.",
            },
            {
              name: "../../../tui/action-target.js",
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
              name: "../../../tui/two-pane-state.js",
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
    files: ["src/tags/editorial/ui/review-screen-model.ts", "src/tags/editorial/ui/review-screen-state.ts"],
    rules: {
      "no-restricted-imports": mergeRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS),
    },
  },
  {
    files: ["src/tags/editorial/ui/review-screen-model.ts", "src/tags/editorial/ui/review-ui-controller.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        ...buildImportSourceSyntaxRestrictions(
          REVIEW_LIST_DETAIL_PRESENTATION_IMPORT_PATHS,
          "Review list/detail surfaces must use src/tui/list-detail-presentation.ts instead of bypassing shared measurement, transition-footer composition, or list/detail routing directly.",
        ),
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
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
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
      }),
    },
  },
  {
    files: ["src/tui/terminal-ui.tsx"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports({
        paths: [
          {
            name: "./interaction-bindings.js",
            importNames: ["getTerminalInteractionCycleDirection", "resolveTerminalInteractionAction"],
            message:
              "Terminal modal routing must go through the shared interaction-context router and prompt-context adapters instead of resolving raw actions directly.",
          },
        ],
      }),
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
    files: ["src/app/**/*.{ts,tsx}", "src/domain/**/*.{ts,tsx}", "src/server/**/*.{ts,tsx}", "src/tui/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-syntax": ["error", ...SEARCH_REQUEST_BOUNDARY_SYNTAX_RESTRICTIONS],
    },
  },
  {
    files: ["src/tui/pf2e-app.tsx"],
    rules: {
      "arch/no-pf2e-app-render-time-ontology-load": "error",
    },
  },
  {
    files: ["src/tui/search-screen/controller.ts"],
    rules: {
      "arch/no-stale-search-screen-terminology": "error",
      "no-restricted-syntax": [
        "error",
        ...SEARCH_SCREEN_CONTROLLER_SYNTAX_RESTRICTIONS,
      ],
    },
  },
  {
    files: ["src/tui/search-screen/workspace/workspace-actions.ts"],
    rules: {
      "arch/no-stale-search-screen-terminology": "error",
      "no-restricted-syntax": [
        "error",
        ...SEARCH_WORKFLOW_PROMPT_BOUNDARY_RESTRICTIONS,
      ],
    },
  },
  {
    files: ["src/tui/search-screen/session-workflow.ts"],
    rules: {
      "arch/no-stale-search-screen-terminology": "error",
      "no-restricted-syntax": [
        "error",
        ...SEARCH_WORKFLOW_PROMPT_BOUNDARY_RESTRICTIONS,
        {
          selector: 'Identifier[name="autoExecuteInitialQuery"]',
          message:
            "Search session workflow must not restore route-entry auto-execute flags. Navigation owns prepared result launches.",
        },
        {
          selector: 'Identifier[name="autoRanInitialQuery"]',
          message:
            "Search session workflow must not restore route-entry bootstrap state. Navigation owns prepared result launches.",
        },
        {
          selector: 'CallExpression[callee.name="executeRequest"]',
          message:
            "Search session workflow must not auto-execute route-entry queries locally. Prepare result-reader routes in navigation before commit.",
        },
      ],
    },
  },
  {
    files: ["src/tui/filter-explorer/**/*.{ts,tsx}", "src/tui/ontology-explorer/**/*.{ts,tsx}", "src/tui/pf2e-app.tsx", "src/tui/pf2e-navigation.ts"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: 'Identifier[name="openInResults"]',
          message:
            "TUI ontology/search launch flows must use explicit launch intents instead of openInResults-style route flags.",
        },
      ],
    },
  },
  {
    files: [
      "src/tui/search-screen/controller.ts",
      "src/tui/search-screen/interactions.ts",
      "src/tui/search-screen/screen.tsx",
      "src/tui/filter-explorer/controller.ts",
      "src/tui/filter-explorer/screen-models.ts",
    ],
    rules: {
      "no-restricted-syntax": [
        "error",
        ...buildImportSourceSyntaxRestrictions(
          SEARCH_LIST_DETAIL_PRESENTATION_IMPORT_PATHS,
          "List/detail search and explorer surfaces must use src/tui/list-detail-presentation.ts instead of bypassing shared measurement, transition-footer composition, or list/detail routing directly.",
        ),
        {
          selector: 'Identifier[name="openInResults"]',
          message:
            "TUI ontology/search launch flows must use explicit launch intents instead of openInResults-style route flags.",
        },
      ],
    },
  },
  {
    files: ["src/tui/ontology-explorer/screen.tsx", "src/tui/ontology-explorer/picker-screen.tsx"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
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
      }),
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
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS,
        SEARCH_REQUEST_BOUNDARY_IMPORT_RESTRICTIONS,
      ),
    },
  },
  {
    files: ["src/tui/**/*.{ts,tsx}"],
    ignores: [
      "src/tui/app-services.ts",
      "src/tui/keymap.ts",
      "src/tui/interaction-bindings.ts",
      "src/tui/action-target.ts",
      "src/tui/terminal-ui.tsx",
    ],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS,
        SEARCH_REQUEST_BOUNDARY_IMPORT_RESTRICTIONS,
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
                "**/tags/editorial/ui/workbench-controller.js",
                "**/tags/editorial/sessions/session-builder.js",
                "**/tags/editorial/state/runtime-state.js",
                "**/tags/editorial/sessions/session-store.js",
              ],
              message:
                "TUI modules must use app-services for tag-workbench composition instead of importing editorial service internals directly.",
            },
          ],
        },
      ),
    },
  },
  {
    files: ["src/app/**/*.{ts,tsx}", "src/domain/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_UI_TUI_IMPORT_RESTRICTIONS,
        NON_TAGS_DERIVED_TAG_IMPORT_RESTRICTIONS,
        SEARCH_REQUEST_BOUNDARY_IMPORT_RESTRICTIONS,
      ),
    },
  },
  {
    files: ["src/domain/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_UI_TUI_IMPORT_RESTRICTIONS,
        NON_TAGS_DERIVED_TAG_IMPORT_RESTRICTIONS,
        SEARCH_REQUEST_BOUNDARY_IMPORT_RESTRICTIONS,
        DOMAIN_SEARCH_INTERNAL_IMPORT_RESTRICTIONS,
      ),
      "no-restricted-syntax": [
        "error",
        ...SEARCH_REQUEST_BOUNDARY_SYNTAX_RESTRICTIONS,
        ...DOMAIN_SEARCH_INTERNAL_SYNTAX_RESTRICTIONS,
      ],
    },
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    ignores: ["src/tags/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_UI_TUI_IMPORT_RESTRICTIONS,
        NON_TAGS_DERIVED_TAG_IMPORT_RESTRICTIONS,
      ),
      "arch/no-direct-search-discovery-primitives": "error",
    },
  },
  {
    files: ["src/search/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_UI_TUI_IMPORT_RESTRICTIONS,
        NON_TAGS_DERIVED_TAG_IMPORT_RESTRICTIONS,
        SEARCH_STORAGE_INTERNAL_IMPORT_RESTRICTIONS,
      ),
    },
  },
  {
    files: ["src/tags/**/*.{ts,tsx}"],
    ignores: [
      "src/tags/cli/**/*",
      "src/tags/editorial/ui/review-detail-content.ts",
      "src/tags/editorial/ui/review-screen-model.ts",
      "src/tags/editorial/ui/review-screen-state.ts",
      "src/tags/editorial/ui/review-ui.tsx",
      "src/tags/editorial/ui/review-ui-controller.ts",
      "src/tags/editorial/sessions/record-loader.ts",
      "src/tags/editorial/types.ts",
      "src/tags/editorial/ui/workbench-controller.ts",
      "src/tags/editorial/ui/workbench-session-prompts.ts",
    ],
    rules: {
      "no-restricted-imports": mergeRestrictedImports(NON_UI_TUI_IMPORT_RESTRICTIONS),
    },
  },
  {
    files: ["src/tags/cli/**/*.{ts,tsx}"],
    ignores: [
      "src/tags/cli/shared/search-scope-args.ts",
      "src/tags/cli/editorial/derived-tag-migration-workbench.ts",
    ],
    rules: {
      "no-restricted-imports": mergeRestrictedImports(
        NON_UI_TUI_IMPORT_RESTRICTIONS,
        TAGS_CLI_SCOPE_IMPORT_RESTRICTIONS,
      ),
    },
  },
  {
    files: ["src/tags/editorial/ui/workbench-controller.ts"],
    rules: {
      "no-restricted-syntax": ["error", ...EDITORIAL_WORKBENCH_PROMPT_BOUNDARY_RESTRICTIONS],
    },
  },
  {
    files: ["src/server/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_UI_TUI_IMPORT_RESTRICTIONS,
        NON_TAGS_DERIVED_TAG_IMPORT_RESTRICTIONS,
        SEARCH_REQUEST_BOUNDARY_IMPORT_RESTRICTIONS,
        SERVER_STORAGE_INTERNAL_IMPORT_RESTRICTIONS,
      ),
    },
  },
  {
    files: ["src/tui/search-screen/controller.ts"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
        paths: [
          {
            name: "../ontology-explorer/facet-picker-model.js",
            importNames: ["buildSearchFacetPickerModel"],
            message:
              "Search screen controllers must use shared ontology-to-search workflow modules instead of constructing facet-picker bridge models directly.",
          },
          {
            name: "../terminal-ui.js",
            importNames: [
              "createDerivedTagTerminalListNavigationState",
              "resolveDerivedTagTerminalListNavigationAction",
              "useDerivedTagTerminalInput",
            ],
            message:
              "Search screen controllers must use the shared search interaction router instead of owning raw terminal event decoding.",
          },
          {
            name: "../interaction-bindings.js",
            importNames: ["TERMINAL_DIALOG_RETURN_FOOTER", "resolveTerminalInteractionAction"],
            message:
              "Search screen controllers must use shared search interaction and help-dialog helpers instead of resolving raw interaction actions or composing return footers directly.",
          },
          {
            name: "./model.js",
            importNames: [
              "SearchFacetPickerSession",
              "applyFacetPickerSelectionsToRequest",
              "buildFacetPickerInitialSelections",
            ],
            message:
              "Search screen controllers must use shared ontology-to-search workflow modules instead of owning facet-picker session bridging directly.",
          },
          {
            name: "./model.js",
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
        patterns: LEGACY_ENTITY_DETAIL_PRESENTER_IMPORT_RESTRICTIONS.patterns,
      }),
    },
  },
  {
    files: ["src/tui/search-screen/interactions.ts"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
        paths: [
          {
            name: "../terminal-ui.js",
            importNames: [
              "createDerivedTagTerminalListNavigationState",
              "resolveDerivedTagTerminalListNavigationAction",
              "useDerivedTagTerminalInput",
            ],
            message:
              "Search interaction routers must use the shared interaction-context router instead of raw list-navigation or input primitives.",
          },
          {
            name: "../interaction-bindings.js",
            importNames: ["resolveTerminalInteractionAction"],
            message:
              "Search interaction routers must use the shared interaction-context router instead of resolving raw interaction actions directly.",
          },
        ],
      }),
    },
  },
  {
    files: ["src/tui/search-screen/**/*.{ts,tsx}"],
    ignores: [
      "src/tui/search-screen/controller.ts",
      "src/tui/search-screen/session-workflow.ts",
      "src/tui/search-screen/workspace/workspace-actions.ts",
    ],
    rules: {
      "arch/no-stale-search-screen-terminology": "error",
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
        patterns: [
          {
            group: ["**/filter-explorer/search-draft-query.js"],
            message:
              "Search-screen workflow code must use the canonical search service preparation seam instead of importing filter-explorer draft-query helpers directly.",
          },
        ],
      }),
    },
  },
  {
    files: ["src/tui/ontology-explorer/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(
        NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS,
        LEGACY_ENTITY_DETAIL_PRESENTER_IMPORT_RESTRICTIONS,
      ),
    },
  },
  {
    files: ["src/tui/filter-explorer/**/*.{ts,tsx}", "src/tui/entity-page-screen.tsx"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(LEGACY_ENTITY_DETAIL_PRESENTER_IMPORT_RESTRICTIONS),
    },
  },
  {
    files: ["src/tui/shared-screens.tsx"],
    rules: {
      "no-restricted-imports": mergeNonTagRestrictedImports(NON_FRAMEWORK_TUI_IMPORT_RESTRICTIONS, {
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
      }),
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
