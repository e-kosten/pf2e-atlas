import path from "node:path";
import { fileURLToPath } from "node:url";

import { ESLint, type Linter } from "eslint";

const testsDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(testsDir, "../..");

let eslint: ESLint | null = null;

function getEslint(): ESLint {
  if (eslint) {
    return eslint;
  }

  eslint = new ESLint({
    cwd: repoRoot,
    overrideConfigFile: path.join(repoRoot, "eslint.config.js"),
  });

  return eslint;
}

export async function lintRuleMessages(relativePath: string, code: string, ruleId = "no-restricted-imports") {
  const engine = getEslint();
  const [result] = await engine.lintText(code, {
    filePath: path.join(repoRoot, relativePath),
  });
  if (!result) {
    throw new Error(`ESLint returned no lint result for ${relativePath}.`);
  }

  const fatalMessages = result.messages.filter((message) => message.fatal);
  if (fatalMessages.length > 0) {
    throw new Error(fatalMessages.map((message) => message.message).join("\n"));
  }

  return result.messages.filter((message) => message.ruleId === ruleId);
}

export function lintMessageTexts(messages: Linter.LintMessage[]): string[] {
  return messages.map((message) => message.message);
}
