"""Filesystem publication and opaque SQLite snapshots for contributor workflows.

No schema migrations or reference writes belong here. Atlas validates compatibility.
The metadata below is recorded verbatim for provenance, never used for admission.
"""

import contextlib
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


@contextlib.contextmanager
def lock(path, exclusive=True):
    # Rust std::fs::File locks use flock on the supported macOS/Linux hosts.
    import fcntl

    with open(path, "a+b") as stream:
        try:
            fcntl.flock(stream, (fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH) | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise RuntimeError(f"Busy: {path}; retry after the active operation finishes") from error
        yield


def read_database(path):
    return sqlite3.connect(Path(path).resolve().as_uri() + "?mode=ro", uri=True)


def snapshot(source, destination):
    source = source.resolve(strict=True)
    # Do not race reference preparation, publication or structural migration.
    # Saved-state writes may continue: SQLite backup supplies a consistent view.
    with lock(str(source) + ".update.lock", exclusive=False), lock(str(source) + ".access.lock", exclusive=False):
        with contextlib.closing(read_database(source)) as reader:
            with contextlib.closing(sqlite3.connect(destination)) as writer:
                reader.backup(writer, pages=4096)
                # Publish a closed standalone file, never a main-file-only WAL copy.
                writer.execute("PRAGMA journal_mode=DELETE").fetchone()


def metadata(path):
    with contextlib.closing(read_database(path)) as connection:
        connection.execute("BEGIN")
        workspace = connection.execute(
            "SELECT workspace_key, reference_revision, maintenance FROM workspace WHERE id=1"
        ).fetchone()
        if not workspace or workspace[2] is not None:
            raise RuntimeError("Workspace is missing or in maintenance; finish setup before publishing")
        context = connection.execute("SELECT context_json FROM artifact_context WHERE singleton=1").fetchone()
        if not context:
            raise RuntimeError("Workspace has no published reference data")
        return {
            "schema_version": connection.execute("PRAGMA user_version").fetchone()[0],
            "workspace_key": workspace[0],
            "reference_revision": workspace[1],
            "context": json.loads(context[0]),
        }


def clone_file(source, destination):
    """Clone a closed seed when supported; ordinary copy is the portable fallback."""
    if sys.platform == "darwin":
        result = subprocess.run(["/bin/cp", "-c", str(source), str(destination)], capture_output=True)
        if result.returncode == 0:
            return "filesystem-clone"
        # A failed clone can leave an incomplete destination in our private staging dir.
        destination.unlink(missing_ok=True)
    shutil.copyfile(source, destination)
    return "copy"


def write_json(path, value):
    with open(path, "w") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())


def replace_json(path, value):
    fd, name = tempfile.mkstemp(prefix=".pointer-", dir=path.parent)
    os.close(fd)
    temporary = Path(name)
    try:
        write_json(temporary, value)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)
