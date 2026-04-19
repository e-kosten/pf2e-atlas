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
  "src/app/ontology-service.ts",
  "src/tui/app-services.ts",
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
};

export default {
  rules: localRules,
};
