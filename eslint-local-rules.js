import path from "node:path";

function toRepoRelativePath(filename) {
  return path.relative(process.cwd(), filename).split(path.sep).join("/");
}

function matchesAllowedPath(filename, allowed) {
  return allowed.some((entry) => (entry instanceof RegExp ? entry.test(filename) : filename === entry));
}

function isJsonParseCall(node) {
  return (
    node.callee?.type === "MemberExpression" &&
    !node.callee.computed &&
    node.callee.object?.type === "Identifier" &&
    node.callee.object.name === "JSON" &&
    node.callee.property?.type === "Identifier" &&
    node.callee.property.name === "parse"
  );
}

function containsRestrictedSearchType(typeAnnotation) {
  if (!typeAnnotation) {
    return false;
  }

  switch (typeAnnotation.type) {
    case "TSTypeReference":
      return (
        typeAnnotation.typeName?.type === "Identifier" &&
        (typeAnnotation.typeName.name === "SearchCategory" || typeAnnotation.typeName.name === "SearchSubcategory")
      );
    case "TSArrayType":
      return containsRestrictedSearchType(typeAnnotation.elementType);
    case "TSUnionType":
      return typeAnnotation.types.some((entry) => containsRestrictedSearchType(entry));
    case "TSParenthesizedType":
      return containsRestrictedSearchType(typeAnnotation.typeAnnotation);
    case "TSTypeOperator":
      return containsRestrictedSearchType(typeAnnotation.typeAnnotation);
    default:
      return false;
  }
}

function isMemberProperty(node, propertyName) {
  return (
    node?.type === "MemberExpression" &&
    !node.computed &&
    node.property?.type === "Identifier" &&
    node.property.name === propertyName
  );
}

function isLiteralString(node, value) {
  return node?.type === "Literal" && node.value === value;
}

function isTerminalInputActionComparison(node, propertyName, literalValue) {
  if (!["==", "===", "!=", "!=="].includes(node.operator)) {
    return false;
  }

  return (
    (isMemberProperty(node.left, propertyName) && isLiteralString(node.right, literalValue)) ||
    (isMemberProperty(node.right, propertyName) && isLiteralString(node.left, literalValue))
  );
}

const JSON_PARSE_ALLOWED_PATHS = [
  "src/data/metadata-glossary.ts",
  "src/data/references.ts",
  "src/data/sql-row-decoding.ts",
  "src/data/indexer.ts",
  "src/data/rows.ts",
  "src/search/ranking-config.ts",
  "src/tags/migration/session-store.ts",
  "src/tags/discovery/row-decoding.ts",
  "src/tags/discovery/discovery-records.ts",
  "src/tui/ontology-explorer/data.ts",
];

const DATABASE_SYNC_ALLOWED_PATHS = [
  "src/data/schema.ts",
  "src/tags/migration/cli-utils.ts",
  "src/refresh-index.ts",
  "src/app/storage-service.ts",
  /^src\/tags\/cli\/.+\.ts$/,
];

const localRules = {
  "no-direct-json-parse": {
    meta: {
      type: "problem",
      docs: {
        description: "Require JSON.parse to stay inside explicit decoder or boundary modules.",
      },
      schema: [],
      messages: {
        noDirectJsonParse:
          "Do not call JSON.parse here. Route JSON decoding through an approved decoder or boundary module instead.",
      },
    },
    create(context) {
      const filename = toRepoRelativePath(context.filename);
      if (matchesAllowedPath(filename, JSON_PARSE_ALLOWED_PATHS)) {
        return {};
      }

      return {
        CallExpression(node) {
          if (isJsonParseCall(node)) {
            context.report({ node, messageId: "noDirectJsonParse" });
          }
        },
      };
    },
  },
  "no-direct-database-sync-construction": {
    meta: {
      type: "problem",
      docs: {
        description: "Limit DatabaseSync construction to composition roots and approved entry modules.",
      },
      schema: [],
      messages: {
        noDatabaseSync:
          "Do not construct DatabaseSync here. Open SQLite connections only in approved composition or entry modules.",
      },
    },
    create(context) {
      const filename = toRepoRelativePath(context.filename);
      if (matchesAllowedPath(filename, DATABASE_SYNC_ALLOWED_PATHS)) {
        return {};
      }

      return {
        NewExpression(node) {
          if (node.callee?.type === "Identifier" && node.callee.name === "DatabaseSync") {
            context.report({ node, messageId: "noDatabaseSync" });
          }
        },
      };
    },
  },
  "no-search-category-assertion": {
    meta: {
      type: "problem",
      docs: {
        description: "Ban raw SearchCategory/SearchSubcategory assertions in favor of parser helpers.",
      },
      schema: [],
      messages: {
        noSearchCategoryAssertion:
          "Do not assert SearchCategory/SearchSubcategory values. Use normalization or parser helpers instead.",
      },
    },
    create(context) {
      const reportIfRestrictedAssertion = (node) => {
        if (containsRestrictedSearchType(node.typeAnnotation)) {
          context.report({ node, messageId: "noSearchCategoryAssertion" });
        }
      };

      return {
        TSAsExpression: reportIfRestrictedAssertion,
        TSTypeAssertion: reportIfRestrictedAssertion,
      };
    },
  },
  "no-direct-terminal-event-routing": {
    meta: {
      type: "problem",
      docs: {
        description:
          "Require TUI feature code to route cancel/back/quit/interrupt behavior through shared interaction helpers.",
      },
      schema: [],
      messages: {
        noDirectInterruptHandling:
          "Do not branch on event.systemAction in feature code. Interrupt handling must stay centralized in terminal-ui.",
        noDirectCancelHandling:
          'Do not branch on event.textInputAction === "cancel" in feature code. Resolve a shared terminal interaction action instead.',
        noDirectSubmitHandling:
          'Do not branch on event.textInputAction === "submit" in feature code. Resolve a shared text-entry intent instead.',
        noDirectDeleteBackwardHandling:
          'Do not branch on event.textInputAction === "deleteBackward" in feature code. Resolve a shared text-entry intent instead.',
        noDirectPrintableHandling:
          "Do not read event.printable in feature code. Resolve shared text-entry intent instead.",
        noDirectBackHandling:
          "Do not call event.isBackNavigationKey() in feature code. Resolve back/return through shared interaction helpers instead.",
        noDirectQuitHandling:
          "Do not call event.isTerminalQuitKey() in feature code. Resolve quit through shared interaction helpers instead.",
      },
    },
    create(context) {
      return {
        MemberExpression(node) {
          if (isMemberProperty(node, "systemAction")) {
            context.report({ node, messageId: "noDirectInterruptHandling" });
            return;
          }
          if (isMemberProperty(node, "printable")) {
            context.report({ node, messageId: "noDirectPrintableHandling" });
          }
        },
        BinaryExpression(node) {
          if (isTerminalInputActionComparison(node, "textInputAction", "cancel")) {
            context.report({ node, messageId: "noDirectCancelHandling" });
            return;
          }
          if (isTerminalInputActionComparison(node, "textInputAction", "submit")) {
            context.report({ node, messageId: "noDirectSubmitHandling" });
            return;
          }
          if (isTerminalInputActionComparison(node, "textInputAction", "deleteBackward")) {
            context.report({ node, messageId: "noDirectDeleteBackwardHandling" });
          }
        },
        SwitchStatement(node) {
          if (!isMemberProperty(node.discriminant, "textInputAction")) {
            return;
          }

          for (const switchCase of node.cases) {
            if (isLiteralString(switchCase.test, "cancel")) {
              context.report({ node: switchCase, messageId: "noDirectCancelHandling" });
              continue;
            }
            if (isLiteralString(switchCase.test, "submit")) {
              context.report({ node: switchCase, messageId: "noDirectSubmitHandling" });
              continue;
            }
            if (isLiteralString(switchCase.test, "deleteBackward")) {
              context.report({ node: switchCase, messageId: "noDirectDeleteBackwardHandling" });
            }
          }
        },
        CallExpression(node) {
          if (isMemberProperty(node.callee, "isBackNavigationKey")) {
            context.report({ node, messageId: "noDirectBackHandling" });
            return;
          }
          if (isMemberProperty(node.callee, "isTerminalQuitKey")) {
            context.report({ node, messageId: "noDirectQuitHandling" });
          }
        },
      };
    },
  },
};

export default {
  rules: localRules,
};
