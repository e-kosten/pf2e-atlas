import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  lstatSync,
  readFileSync,
  readlinkSync,
  statSync,
  readdirSync,
} from "node:fs";

export function inputIdentity(scope) {
  const paths = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })
    .decode(
      execFileSync("git", [
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "--deduplicate",
        "-z",
        "--",
        ".",
        ":(top,exclude).atlas/validation",
      ]),
    )
    .split("\0")
    .filter(Boolean)
    .sort();
  const hash = createHash("sha256");
  const field = (value) => {
    const bytes = Buffer.from(value);
    hash.update(`${bytes.length}:`).update(bytes);
  };
  for (const path of paths) {
    if (
      scope === "web-format" &&
      !/^(web\/atlas-ui\/|\.prettier|\.editorconfig$|prettier\.config\.|package(?:-lock)?\.json$)/.test(
        path,
      )
    )
      continue;
    if (
      scope === "rust-format" &&
      !/(\.rs$|(^|\/)Cargo\.(toml|lock)$|^rust-toolchain|(^|\/)(\.rustfmt|rustfmt)\.toml$|^\.cargo\/)/.test(
        path,
      )
    )
      continue;
    if (
      scope === "code" &&
      path.endsWith(".md") &&
      !/^(dev-tools\/source-contracts\/|docs\/research\/evaluations\/|skills\/pf2e-atlas-cli\/)/.test(
        path,
      )
    )
      continue;
    let metadata;
    try {
      metadata = lstatSync(path);
    } catch (error) {
      if (error.code === "ENOENT") continue;
      throw error;
    }
    field(path);
    field(
      metadata.isSymbolicLink()
        ? "link"
        : metadata.isFile()
          ? metadata.mode & 0o111
            ? "executable"
            : "file"
          : "other",
    );
    if (metadata.isSymbolicLink())
      field(readlinkSync(path, { encoding: "buffer" }));
    if (
      metadata.isFile() ||
      (metadata.isSymbolicLink() && existsSync(path) && statSync(path).isFile())
    )
      field(readFileSync(path));
  }
  return hash.digest("hex");
}

const hash = (value) => createHash("sha256").update(value).digest("hex");
export const commandIdentity = (command) => hash(JSON.stringify(command));
export function runnerIdentity() {
  return hash(
    ["inputs.mjs", "run-check.mjs", "run-process.mjs"]
      .map((name) => readFileSync(new URL(name, import.meta.url)))
      .map((value) => hash(value))
      .join("\n"),
  );
}
export function environmentState() {
  const transient = new Set([
    "_",
    "SHLVL",
    "PWD",
    "OLDPWD",
    "ATLAS_VALIDATION_OUTPUT_ROOT",
    "ATLAS_VALIDATION_HEARTBEAT_SECONDS",
    "ATLAS_VALIDATION_CANCEL_FILE",
    "GITHUB_STEP_SUMMARY",
  ]);
  return Object.fromEntries(
    Object.entries(process.env)
      .filter(([name]) => !transient.has(name))
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([name, value]) => [name, hash(value)]),
  );
}
export const environmentIdentity = () =>
  hash(JSON.stringify(environmentState()));
export function artifactIdentity(command, args) {
  const files = [];
  if (command === "npm" && args.includes("--prefix")) {
    const path = `${args[args.indexOf("--prefix") + 1]}/node_modules/.package-lock.json`;
    if (existsSync(path)) files.push(path);
  }
  function collect(path) {
    if (!existsSync(path)) return;
    if (statSync(path).isDirectory())
      for (const name of readdirSync(path).sort()) collect(`${path}/${name}`);
    else if (!path.endsWith("/build-provenance.json")) files.push(path);
  }
  if (command === "cargo" && args.includes("--workspace"))
    collect("web/atlas-ui/dist");
  return hash(
    files.map((path) => `${path}\0${hash(readFileSync(path))}`).join("\n"),
  );
}

export function toolchainIdentity(command) {
  let text = `node ${process.version} ${process.execPath}\n${process.platform} ${process.arch}\n`;
  for (const tool of command === "cargo"
    ? ["rustc", "cargo"]
    : command === "npm"
      ? ["npm"]
      : []) {
    try {
      text += execFileSync(
        process.platform === "win32" ? "sh" : tool,
        process.platform === "win32"
          ? ["-c", '"$@" --version', "sh", tool]
          : ["--version"],
        {
          encoding: "utf8",
          timeout: 5000,
          env: { ...process.env, RUSTUP_AUTO_INSTALL: "0" },
        },
      );
    } catch {
      return { text: text + `${tool} unavailable\n`, identity: "unavailable" };
    }
  }
  return { text, identity: hash(text) };
}
