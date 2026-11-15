import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

function unsignedPayload(path) {
  const bytes = readFileSync(path);
  assert.equal(bytes.readUInt32LE(0), 0xfeedfacf, "Expected a thin 64-bit Mach-O");
  const end = 32 + bytes.readUInt32LE(20);
  assert.ok(end <= bytes.length);
  let linkedit;
  for (let offset = 32; offset < end;) {
    const command = bytes.readUInt32LE(offset);
    const size = bytes.readUInt32LE(offset + 4);
    assert.ok(size >= 8 && offset + size <= end);
    assert.notEqual(command, 0x1d, "Remove the signature before comparing");
    if (command === 0x19 && size >= 72 && bytes.toString("ascii", offset + 8, offset + 24).split("\0")[0] === "__LINKEDIT") {
      assert.equal(linkedit, undefined, "Expected one __LINKEDIT segment");
      linkedit = offset;
    }
    offset += size;
  }
  assert.ok(linkedit !== undefined, "Missing __LINKEDIT segment");
  // codesign can leave a different virtual allocation after stripping its blob.
  // The file offset, file size and all actual payload bytes must still match.
  const virtualSize = bytes.readBigUInt64LE(linkedit + 32);
  const fileSize = bytes.readBigUInt64LE(linkedit + 48);
  assert.ok(virtualSize >= fileSize);
  bytes.writeBigUInt64LE(fileSize, linkedit + 32);
  return bytes;
}

const [archive, bundle] = process.argv.slice(2);
assert.ok(unsignedPayload(archive).equals(unsignedPayload(bundle)), "Bundled CLI differs from the packaged executable");
