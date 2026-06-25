/* global URL, console, process */

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const sourceRoot = join(root, "src");
const allowedFiles = new Set(["src/styles/theme.css"]);
const checkedExtensions = new Set([".css", ".ts", ".tsx"]);
const rawColorPattern = /#[0-9a-fA-F]{3,8}\b|rgba?\(|hsla?\(/g;

const violations = [];

for (const file of walk(sourceRoot)) {
  const relativePath = relative(root, file);
  if (allowedFiles.has(relativePath)) {
    continue;
  }
  const contents = readFileSync(file, "utf8");
  const lines = contents.split(/\r?\n/);
  lines.forEach((line, index) => {
    if (rawColorPattern.test(line)) {
      violations.push(`${relativePath}:${index + 1}: ${line.trim()}`);
    }
    rawColorPattern.lastIndex = 0;
  });
}

if (violations.length > 0) {
  console.error("Raw color literals are only allowed in src/styles/theme.css.");
  console.error(violations.join("\n"));
  process.exit(1);
}

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
