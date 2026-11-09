"""Generate the verified managed-source tree digest for the pinned Git revision."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("checkout", type=Path)
    args = parser.parse_args()
    pin_path = Path(__file__).with_name("source-pin.json")
    pin = json.loads(pin_path.read_text())
    with subprocess.Popen(
        ["git", "-C", str(args.checkout), "archive", pin["commit"]],
        stdout=subprocess.PIPE,
    ) as process:
        hashes = {}
        with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
            for member in archive:
                if member.isdir():
                    continue
                if not member.isfile():
                    raise ValueError(f"Unsupported entry: {member.name}")
                with archive.extractfile(member) as source:
                    hashes[member.name.encode()] = hashlib.file_digest(source, "sha256").digest()
        if process.wait() != 0:
            raise SystemExit("Could not read the pinned Git tree")
    digest = hashlib.sha256(b"atlas-managed-source-v1\0")
    for name, file_hash in sorted(hashes.items()):
        digest.update(len(name).to_bytes(8, "big"))
        digest.update(name)
        digest.update(file_hash)
    print(digest.hexdigest())


if __name__ == "__main__":
    main()
