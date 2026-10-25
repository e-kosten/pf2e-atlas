/* global URL, console, process */

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const sourceRoot = join(root, "src");
const allowedFiles = new Set([
  "src/shared/theme/themeDefinitions.ts",
  "src/styles/theme.css",
]);
const checkedExtensions = new Set([".css", ".ts", ".tsx"]);
const rawColorPattern =
  /#[0-9a-fA-F]{3,8}\b|(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\(/g;
const semanticKeywords = new Set([
  "transparent",
  "currentcolor",
  "inherit",
  "initial",
  "unset",
  "revert",
  "none",
  "default",
  "primary",
  "danger",
  "success",
  "warning",
  "error",
  "info",
]);
const cssColorProperty =
  /(?:^|[;{])\s*(?:color|background(?:-color)?|border-color|fill|stroke)\s*:\s*([a-z]+)\s*[;}]/gi;
const quotedColorProperty =
  /\b(?:color|backgroundColor|background|borderColor|fill|stroke)\s*[:=]\s*["']([a-z]+)["']/gi;
const cssVariableUse = /var\((--[a-z0-9-]+)/g;
const cssVariableDefinition = /(--[a-z0-9-]+)\s*:/g;
const objectVariableDefinition = /["'](--[a-z0-9-]+)["']\s*:/g;
const setPropertyVariableDefinition = /setProperty\(["'](--[a-z0-9-]+)["']/g;

const violations = [];
const variableDefinitions = new Set();
const variableUses = [];

for (const file of walk(sourceRoot)) {
  const relativePath = relative(root, file);
  const contents = readFileSync(file, "utf8");
  const definitionPattern = file.endsWith(".css")
    ? cssVariableDefinition
    : objectVariableDefinition;
  for (const match of contents.matchAll(definitionPattern)) {
    variableDefinitions.add(match[1]);
  }
  if (!file.endsWith(".css")) {
    for (const match of contents.matchAll(setPropertyVariableDefinition)) {
      variableDefinitions.add(match[1]);
    }
  }
  for (const match of contents.matchAll(cssVariableUse)) {
    variableUses.push({ name: match[1], file: relativePath });
  }
  if (allowedFiles.has(relativePath)) {
    continue;
  }
  const lines = contents.split(/\r?\n/);
  lines.forEach((line, index) => {
    const namedPattern = file.endsWith(".css") ? cssColorProperty : quotedColorProperty;
    const namedColors = Array.from(line.matchAll(namedPattern));
    if (
      rawColorPattern.test(line) ||
      namedColors.some((match) => !semanticKeywords.has(match[1].toLowerCase()))
    ) {
      violations.push(`${relativePath}:${index + 1}: ${line.trim()}`);
    }
    rawColorPattern.lastIndex = 0;
  });
}

const undefinedVariables = variableUses.filter(
  ({ name }) => !variableDefinitions.has(name) && !name.startsWith("--rarity-"),
);

if (violations.length > 0) {
  console.error("Raw color literals belong in src/shared/theme/themeDefinitions.ts.");
  console.error(violations.join("\n"));
}
if (undefinedVariables.length > 0) {
  console.error("CSS variables must have a theme, stylesheet, or inline definition.");
  console.error(
    undefinedVariables.map(({ name, file }) => `${file}: ${name}`).join("\n"),
  );
}
if (violations.length > 0 || undefinedVariables.length > 0) process.exit(1);

function* walk(directory) {
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      yield* walk(path);
      continue;
    }
    if (checkedExtensions.has(extension(path))) {
      yield path;
    }
  }
}

function extension(path) {
  const index = path.lastIndexOf(".");
  return index === -1 ? "" : path.slice(index);
}
