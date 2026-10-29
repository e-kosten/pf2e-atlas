import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const scripts = dirname(fileURLToPath(import.meta.url));
const quote = (value) => `'${value.replaceAll("'", "'\\''")}'`;
function fixture(t) {
  const root = mkdtempSync(resolve(tmpdir(), "atlas-preview-build-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const put = (name, value) => {
    const path = resolve(root, name);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, value);
  };
  put(".gitignore", ".atlas/\ntarget/\nweb/atlas-ui/dist/\nfake-bin/\n");
  put("Cargo.toml", "[workspace]\n");
  put(
    "scripts/validation/inputs.mjs",
    readFileSync(new URL("../validation/inputs.mjs", import.meta.url)),
  );
  put("rust-toolchain.toml", '[toolchain]\nchannel="1.95.0"\n');
  put(
    "web/atlas-ui/package.json",
    '{"type":"module","engines":{"node":"^22.18.0 || ^24.0.0"}}',
  );
  put("web/atlas-ui/node_modules/.package-lock.json", "{}");
  put(
    "scripts/development/preflight.mjs",
    readFileSync(resolve(scripts, "preflight.mjs")),
  );
  put("crates/example/src/lib.rs", "pub fn current() {}\n");
  put(
    "crates/atlas-app-model/bindings/Test.ts",
    "export type Test = string;\n",
  );
  put(
    "crates/atlas-web/api-contract.json",
    JSON.stringify({
      protocol_revision: 1,
      bindings: "crates/atlas-app-model/bindings",
    }),
  );
  put("web/atlas-ui/src/index.ts", "export const view = 'first';\n");
  put(
    "web/atlas-ui/apiContract.ts",
    readFileSync(resolve(scripts, "../../web/atlas-ui/apiContract.ts")),
  );
  put(
    "scripts/development/build-preview.mjs",
    readFileSync(resolve(scripts, "build-preview.mjs")),
  );
  put(
    "fake-bin/tool.mjs",
    `
import { appendFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const kind=process.argv[2];
if(kind==='rustc'){console.log('rustc 1.95.0 (fixture)');process.exit(0);}
const root=process.cwd();
if(kind==='cargo'&&process.argv[3]==='metadata'){console.log(JSON.stringify({target_directory:resolve(root,'target'),packages:[]}));process.exit(0);}
mkdirSync(resolve(root,'.atlas'),{recursive:true});
appendFileSync(resolve(root,'.atlas/calls'),kind+'\\n');
if(kind==='npm'){
 const {readAtlasApiContract,frontendInputIdentity}=await import(pathToFileURL(resolve(root,'web/atlas-ui/apiContract.ts')));
 const id=readAtlasApiContract(root).identifier;
 mkdirSync(resolve(root,'web/atlas-ui/dist/assets'),{recursive:true});
 writeFileSync(resolve(root,'web/atlas-ui/dist/index.html'),'<html>fixture</html>');
 writeFileSync(resolve(root,'web/atlas-ui/dist/assets/app.js'),'fixture asset');
 writeFileSync(resolve(root,'web/atlas-ui/dist/api-contract.txt'),id+'\\n');
 const sha256={};
 for(const name of ['api-contract.txt','assets/app.js','index.html'])sha256[name]=createHash('sha256').update(readFileSync(resolve(root,'web/atlas-ui/dist',name))).digest('hex');
 writeFileSync(resolve(root,'web/atlas-ui/dist/build-provenance.json'),JSON.stringify({version:1,input_sha256:frontendInputIdentity(root),api_contract:id,sha256}));
}else if(kind==='cargo'){
 mkdirSync(resolve(root,'target/debug'),{recursive:true});
 writeFileSync(resolve(root,'target/debug/atlas'),'fixture executable');
 if(process.env.ATLAS_BUILD_TEST_MUTATE==='1')appendFileSync(resolve(root,'crates/example/src/lib.rs'),'// changed while compiling\\n');
 if(process.env.ATLAS_BUILD_TEST_COMMIT==='1')execFileSync('git',['-c','core.hooksPath=/dev/null','-c','user.name=Fixture','-c','user.email=fixture@example.com','commit','--allow-empty','-qm','checkpoint']);
}
`,
  );
  for (const name of ["npm", "cargo", "rustc"]) {
    const path = resolve(root, "fake-bin", name);
    writeFileSync(
      path,
      `#!/bin/sh\nexec ${quote(process.execPath)} --experimental-strip-types ${quote(resolve(root, "fake-bin/tool.mjs"))} ${name} "$@"\n`,
      { mode: 0o755 },
    );
  }
  execFileSync("git", ["init", "--quiet", root]);
  execFileSync("git", ["-C", root, "add", "."]);
  execFileSync("git", [
    "-C",
    root,
    "-c",
    "core.hooksPath=/dev/null",
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.com",
    "commit",
    "-qm",
    "fixture",
  ]);
  return {
    root,
    put,
    build(extra = {}) {
      return spawnSync(
        process.execPath,
        [
          "--experimental-strip-types",
          resolve(root, "scripts/development/build-preview.mjs"),
          root,
        ],
        {
          cwd: root,
          encoding: "utf8",
          env: {
            ...process.env,
            PATH: `${resolve(root, "fake-bin")}:${process.env.PATH}`,
            ...extra,
          },
        },
      );
    },
    calls() {
      return readFileSync(resolve(root, ".atlas/calls"), "utf8")
        .trim()
        .split("\n");
    },
    receipt() {
      return readFileSync(resolve(root, ".atlas/preview-build.json"), "utf8");
    },
  };
}
function success(result) {
  assert.equal(result.status, 0, result.stdout + result.stderr);
}

test("Rust-only and frontend test/doc changes reuse the verified UI bundle", (t) => {
  const f = fixture(t);
  success(f.build());
  f.put("crates/example/src/lib.rs", "pub fn changed() {}\n");
  f.put("web/atlas-ui/e2e/example.spec.ts", "// browser-only change\n");
  f.put("web/atlas-ui/docs/example.md", "Documentation only\n");
  f.put("web/atlas-ui/src/example.test.ts", "// unit-test-only change\n");
  const second = f.build();
  success(second);
  assert.deepEqual(f.calls(), ["npm", "cargo", "cargo"]);
  const receipt = JSON.parse(f.receipt());
  assert.equal(
    receipt.binary_sha256,
    createHash("sha256").update("fixture executable").digest("hex"),
  );
  assert.equal(
    receipt.frontend_sha256,
    createHash("sha256").update("<html>fixture</html>").digest("hex"),
  );
  assert.equal(receipt.source.dirty, true);
  assert.match(receipt.api_contract, /^atlas-web-v2-/);
});

test("UI source and API contract changes rebuild the bundle", (t) => {
  const f = fixture(t);
  success(f.build());
  f.put("web/atlas-ui/src/index.ts", "export const view = 'second';\n");
  success(f.build());
  f.put(
    "crates/atlas-app-model/bindings/Test.ts",
    "export type Test = number;\n",
  );
  success(f.build());
  assert.deepEqual(f.calls(), ["npm", "cargo", "npm", "cargo", "npm", "cargo"]);
});

test("tampered bundle assets invalidate frontend reuse", (t) => {
  const f = fixture(t);
  success(f.build());
  f.put("web/atlas-ui/dist/assets/app.js", "different bundle asset");
  success(f.build());
  assert.deepEqual(f.calls(), ["npm", "cargo", "npm", "cargo"]);
});

test("source mutation during a build refuses a new receipt before refresh", (t) => {
  const f = fixture(t);
  success(f.build());
  const receipt = f.receipt();
  const changed = f.build({ ATLAS_BUILD_TEST_MUTATE: "1" });
  assert.notEqual(changed.status, 0);
  assert.match(changed.stderr, /changed during the build/);
  assert.equal(f.receipt(), receipt);
});

test("a logical commit with unchanged inputs does not invalidate compilation", (t) => {
  const f = fixture(t);
  success(f.build({ ATLAS_BUILD_TEST_COMMIT: "1" }));
  const source = JSON.parse(f.receipt()).source;
  assert.notEqual(source.observed_start.commit, source.observed_finish.commit);
  assert.equal(
    source.observed_start.input_sha256,
    source.observed_finish.input_sha256,
  );
});
