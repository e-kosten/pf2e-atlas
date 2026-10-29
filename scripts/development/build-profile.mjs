import { existsSync, realpathSync } from "node:fs";
import { relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

export function selectedProfile(args, defaultProfile = "dev") {
  if (!args.length) return defaultProfile;
  if (args.length !== 2 || args[0] !== "--profile")
    throw new Error("Expected --profile dev or release.");
  return args[1];
}

export function buildProfile(root, profile) {
  if (!["dev", "release"].includes(profile))
    throw new Error("Expected --profile dev or release.");
  const release = profile === "release";
  return {
    name: profile,
    cargoArgs: release ? ["--release"] : [],
    binary: resolve(
      root,
      "target",
      release ? "release" : "debug",
      process.platform === "win32" ? "atlas.exe" : "atlas",
    ),
    receipt: resolve(
      root,
      ".atlas",
      release ? "maintenance-build.json" : "preview-build.json",
    ),
  };
}

if (
  process.argv[1] &&
  existsSync(process.argv[1]) &&
  import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href
) {
  try {
    const [root, profile, ...extra] = process.argv.slice(2);
    if (!root || !profile || extra.length)
      throw new Error("Usage: build-profile.mjs CHECKOUT dev|release");
    const selected = buildProfile(root, profile);
    const local = (path) => relative(resolve(root), path).split(sep).join("/");
    console.log(`${local(selected.binary)} ${local(selected.receipt)}`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 2;
  }
}
