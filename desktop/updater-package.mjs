import { createHash } from "node:crypto";
import { basename } from "node:path";
import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const tauri = fileURLToPath(new URL("./node_modules/@tauri-apps/cli/tauri.js", import.meta.url));
const [action, tag, input, output] = process.argv.slice(2);
const version = tag?.match(/^v(\d+\.\d+\.\d+(?:-nightly\.\d{14}\.g[0-9a-f]{12})?)$/)?.[1];
const publicKey = process.env.ATLAS_UPDATER_PUBLIC_KEY;

function keyId(encoded, length) {
  const line = Buffer.from(encoded.trim(), "base64").toString("utf8").split(/\r?\n/)[1];
  const bytes = Buffer.from(line ?? "", "base64");
  if (bytes.length !== length || !["Ed", "ED"].includes(bytes.subarray(0, 2).toString())) {
    throw new Error("Invalid Tauri updater signing material");
  }
  return bytes.subarray(2, 10).toString("hex");
}

if (action === "prepare") {
  if (!version) {
    appendFileSync(output, "ATLAS_UPDATER_PUBLIC_KEY=\n");
  } else {
    let key = publicKey;
    if (process.env.ATLAS_UPDATER_SIGNING_MODE === "rehearsal") {
      const keyPath = join(input, "atlas-update-rehearsal.key");
      execFileSync(process.execPath, [tauri, "signer", "generate", "--ci", "--write-keys", keyPath], {
        stdio: "ignore",
      });
      key = readFileSync(keyPath + ".pub", "utf8").trim();
      appendFileSync(output, `ATLAS_UPDATER_REHEARSAL_KEY=${keyPath}\n`);
    }
    if (!key) throw new Error("Configure ATLAS_UPDATER_PUBLIC_KEY before packaging a release");
    keyId(key, 42);
    appendFileSync(output, `ATLAS_UPDATER_PUBLIC_KEY=${key}\n`);
  }
} else if (action === "compatibility") {
  if (!version || !publicKey) throw new Error("Expected an updater-enabled Atlas release");
  const info = JSON.parse(execFileSync(input, ["desktop-probe", "--update-compatibility"], { encoding: "utf8" }));
  if (info.status !== "ok" || !Number.isSafeInteger(info.data?.workspaceSchema)
      || !/^[0-9a-f]{40}$/.test(info.data?.foundryCommit)
      || !/^\d+\.\d+\.\d+$/.test(info.data?.foundryVersion)) {
    throw new Error("Invalid compatibility metadata from the bundled Atlas CLI");
  }
  const commit = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  writeFileSync(output, JSON.stringify({ version, commit, ...info.data }) + "\n");
  process.stdout.write(String(info.data.workspaceSchema));
} else if (action === "sign") {
  if (version) {
    if (!publicKey) throw new Error("Expected the packaged Atlas updater public key");
    const rehearsalKey = process.env.ATLAS_UPDATER_REHEARSAL_KEY;
    const env = { ...process.env };
    if (rehearsalKey) {
      delete env.TAURI_SIGNING_PRIVATE_KEY;
      env.TAURI_SIGNING_PRIVATE_KEY_PATH = rehearsalKey;
      env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "";
    } else if (!env.TAURI_SIGNING_PRIVATE_KEY && !env.TAURI_SIGNING_PRIVATE_KEY_PATH) {
      throw new Error("Configure the Atlas updater signing secret before packaging a release");
    }
    // Bind every signed file to the full version even when the bundle version is shorter.
    const sign = (file) => {
      execFileSync(process.execPath, [tauri, "signer", "sign", "--app-version", version, file], {
        env,
        stdio: ["ignore", "inherit", "inherit"],
      });
      if (keyId(publicKey, 42) !== keyId(readFileSync(file + ".sig", "utf8"), 74)) {
        throw new Error("Updater signing secret does not match the packaged Atlas public key");
      }
    };
    const info = JSON.parse(readFileSync(input + ".compatibility.json", "utf8"));
    const nightly = version.match(/-nightly\.(\d{14}\.g[0-9a-f]{12})$/)?.[1];
    if (info.version !== version || !/^[a-f0-9]{40}$/.test(info.commit)
      || (nightly && !nightly.endsWith(".g" + info.commit.slice(0, 12)))) {
      throw new Error("Atlas transition metadata does not match the packaged build");
    }
    const descriptor = input + ".transition.json";
    writeFileSync(descriptor, JSON.stringify({
      protocol: 2,
      channel: nightly ? "nightly" : "stable",
      tag: nightly ? "nightly/" + nightly : "release/" + tag,
      ...info,
      asset: basename(input),
      sha256: createHash("sha256").update(readFileSync(input)).digest("hex"),
    }) + "\n");
    sign(input);
    sign(descriptor);
  }
} else {
  throw new Error("Expected prepare, compatibility, or sign");
}
