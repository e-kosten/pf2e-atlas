import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

export function releaseConfig(tag, commit, runId, directory) {
  const match = /^v(\d+\.\d+\.\d+)(?:-[\da-zA-Z][\da-zA-Z.-]*)?$/.exec(tag);
  if (!match || match[0] !== tag || !/^[a-f0-9]{40}$/.test(commit) || !/^[1-9]\d*$/.test(runId)) {
    throw new Error("Expected an Atlas release tag, source commit and numeric build ID");
  }
  if (tag.includes("-nightly.") && !tag.endsWith(`.g${commit.slice(0, 12)}`)) {
    throw new Error("Nightly tag does not match the source commit");
  }
  return {
    version: match[1],
    bundle: {
      macOS: {
        bundleVersion: runId,
        minimumSystemVersion: "26.0",
        signingIdentity: "-",
        infoPlist: join(directory, "ReleaseInfo.plist"),
        files: { "Resources/THIRD-PARTY-NOTICES.md": "../.generated/THIRD-PARTY-NOTICES.md" },
      },
    },
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [tag, commit, runId, directory] = process.argv.slice(2);
  const config = releaseConfig(tag, commit, runId, directory);
  writeFileSync(config.bundle.macOS.infoPlist, `<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>AtlasReleaseTag</key><string>${tag}</string>
<key>AtlasSourceCommit</key><string>${commit}</string>
</dict></plist>
`);
  writeFileSync(join(directory, "config.json"), JSON.stringify(config));
}
