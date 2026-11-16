// Serve a disposable, already signed release over loopback for native rehearsal.
import { createReadStream, readFileSync, statSync } from "node:fs";
import { createServer } from "node:https";
import { join } from "node:path";

const [portText, key, cert, tag, directory] = process.argv.slice(2);
const port = Number(portText);
const stable = /^release\/v\d+\.\d+\.\d+$/.test(tag);
if (!Number.isInteger(port) || port < 1024 || port > 65535 || !directory ||
    (!stable && !/^nightly\/\d{14}\.g[0-9a-f]{12}$/.test(tag))) {
  throw new Error("Expected PORT TLS_KEY TLS_CERT RELEASE_REF ASSEMBLED_DIRECTORY");
}
const root = `https://127.0.0.1:${port}/releases`;
const downloadPath = `/releases/download/${tag}/`;
const manifest = JSON.parse(readFileSync(join(directory, "atlas-desktop-update.json"), "utf8"));
const packages = new Map();
for (const platform of Object.values(manifest.platforms)) {
  const name = new URL(platform.url).pathname.split("/").at(-1);
  if (!/^atlas-desktop-[a-z0-9.-]+$/.test(name)) throw new Error("Invalid update package name");
  const file = join(directory, name);
  packages.set(downloadPath + name, { file, size: statSync(file).size });
  platform.url = `${root}/download/${tag}/${name}`;
}

createServer({ key: readFileSync(key), cert: readFileSync(cert) }, (request, response) => {
  if (request.url === "/releases/latest") {
    response.writeHead(302, { location: stable ? `${root}/tag/${tag}` : root }).end();
  } else if (!stable && request.url === "/releases/download/nightly/latest/nightly-version.txt") {
    response.end(tag + "\n");
  } else if (request.url === downloadPath + "atlas-desktop-update.json") {
    response.writeHead(200, { "content-type": "application/json" }).end(JSON.stringify(manifest));
  } else if (packages.has(request.url)) {
    const { file, size } = packages.get(request.url);
    response.writeHead(200, { "content-type": "application/octet-stream", "content-length": size });
    createReadStream(file).on("error", () => response.destroy()).pipe(response);
  } else {
    response.writeHead(404).end();
  }
}).listen(port, "127.0.0.1", () => console.log(root));
