import { existsSync, readdirSync, realpathSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

export function toolingChecks(root = process.cwd()) {
  const tests = [
    "scripts",
    "scripts/development",
    "scripts/validation",
  ].flatMap((directory) =>
    readdirSync(resolve(root, directory))
      .filter((name) => name.endsWith(".test.mjs"))
      .sort()
      .map((name) => `${directory}/${name}`),
  );
  if (!tests.length) throw new Error("Tooling regression tests are missing");
  return [
    ["hook routing", "sh", "scripts/git-hooks/test-common.sh"],
    ["tooling regressions", "node", "--test", ...tests],
  ];
}
if (
  process.argv[1] &&
  existsSync(process.argv[1]) &&
  import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href
) {
  try {
    for (const [label, command, ...args] of toolingChecks()) {
      console.error(`Running ${label}...`);
      const result =
        process.platform === "win32"
          ? spawnSync("sh", ["-c", 'exec "$@"', "sh", command, ...args], {
              stdio: "inherit",
            })
          : spawnSync(command, args, { stdio: "inherit" });
      if (result.error) throw result.error;
      if (result.status !== 0) {
        process.exitCode = result.status ?? 1;
        break;
      }
    }
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
