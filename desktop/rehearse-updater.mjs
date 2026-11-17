// Serve disposable, already signed releases over loopback for native rehearsals.
import { createReadStream, readFileSync, statSync } from "node:fs";
import { createServer } from "node:https";
import { join } from "node:path";

const [portText, key, cert, ...releases] = process.argv.slice(2);
const port = Number(portText);
if (!Number.isInteger(port) || port < 1024 || port > 65535 || !cert ||
    (releases.length !== 2 && releases.length !== 4)) {
  throw new Error("Expected PORT TLS_KEY TLS_CERT RELEASE_REF ASSEMBLED_DIRECTORY [RELEASE_REF ASSEMBLED_DIRECTORY]");
}

const root = `https://127.0.0.1:${port}/releases`;
const manifests = new Map();
const files = new Map();
let stableTag;
let nightlyTag;
for (let index = 0; index < releases.length; index += 2) {
  const tag = releases[index];
  const directory = releases[index + 1];
  if (/^release\/v\d+\.\d+\.\d+$/.test(tag) && !stableTag) stableTag = tag;
  else if (/^nightly\/\d{14}\.g[0-9a-f]{12}$/.test(tag) && !nightlyTag) nightlyTag = tag;
  else throw new Error("Expected at most one official release and one nightly");
  const download = `/releases/download/${tag}/`;
  const manifest = JSON.parse(readFileSync(join(directory, "atlas-desktop-update.json"), "utf8"));
  for (const platform of Object.values(manifest.platforms)) {
    const name = new URL(platform.url).pathname.split("/").at(-1);
    if (!/^atlas-desktop-[a-z0-9.-]+$/.test(name)) throw new Error("Invalid update package name");
    for (const asset of [name, name + ".transition.json", name + ".transition.json.sig"]) {
      const file = join(directory, asset);
      const metadata = statSync(file, { throwIfNoEntry: false });
      if (metadata?.isFile()) files.set(download + asset, { file, size: metadata.size });
    }
    platform.url = `https://127.0.0.1:${port}${download}${name}`;
  }
  manifests.set(download + "atlas-desktop-update.json", JSON.stringify(manifest));
}

createServer({ key: readFileSync(key), cert: readFileSync(cert) }, (request, response) => {
  if (request.url === "/releases/latest") {
    response.writeHead(302, { location: stableTag ? `${root}/tag/${stableTag}` : root }).end();
  } else if (nightlyTag && request.url === "/releases/download/nightly/latest/nightly-version.txt") {
    response.end(nightlyTag + "\n");
  } else if (manifests.has(request.url)) {
    response.writeHead(200, { "content-type": "application/json" }).end(manifests.get(request.url));
  } else if (files.has(request.url)) {
    const { file, size } = files.get(request.url);
    response.writeHead(200, { "content-type": "application/octet-stream", "content-length": size });
    createReadStream(file).on("error", () => response.destroy()).pipe(response);
  } else {
    response.writeHead(404).end();
  }
}).listen(port, "127.0.0.1", () => console.log(root));
