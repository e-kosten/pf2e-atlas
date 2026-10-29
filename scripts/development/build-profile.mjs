import { resolve } from "node:path";

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
