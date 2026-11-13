#!/usr/bin/env python3
"""Exercise the installed CLI entry point against a model-free release fixture."""
import argparse
import gzip
import json
import os
from pathlib import Path
import runpy
import sqlite3
import subprocess
import tempfile
from contextlib import closing

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "crates/atlas-runtime/tests/fixtures/schema-42.sqlite.gz"
MATERIALIZE_SOURCE = runpy.run_path(str(Path(__file__).with_name("probe-embedding-upgrade.py")))["materialize_source"]


def check(candidate, root):
    index = root / "workspace.sqlite"
    source = MATERIALIZE_SOURCE(root)
    index.write_bytes(gzip.decompress(FIXTURE.read_bytes()))
    with closing(sqlite3.connect(index)) as db, db:
        db.execute("""INSERT INTO saved_lists
                      (id,owner_id,list_key,slug,name,created_at,updated_at)
                      VALUES(17,1,'kept-list','kept','Retained list','before','before')""")
    env = dict(os.environ, **dict.fromkeys(
        ("HOME", "USERPROFILE", "LOCALAPPDATA", "XDG_CACHE_HOME", "XDG_DATA_HOME"), str(root),
    ))
    for name in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"):
        env.pop(name, None)

    def invoke(*args, succeeds=True):
        result = subprocess.run(
            [candidate, *args, "--path-mode", "global", "--index", index, "--json"],
            env=env, cwd=root, capture_output=True, text=True, timeout=60,
        )
        if (result.returncode == 0) != succeeds:
            raise RuntimeError(f"{args}: {result.stdout}\n{result.stderr}")
        return json.loads(result.stdout)

    query = ("search", "Demoralize", "--retrieval", "fts")
    before = invoke(*query, succeeds=False)
    if (before["error"]["code"] != "artifact_incompatible"
            or "Run `atlas setup`" not in before["error"]["message"]):
        raise RuntimeError(f"Expected setup requirement before migration: {before}")
    setup = invoke("setup", "--offline", "--no-embeddings", "--source", source,
                   "--embedding-cache-path", root / "models")["data"]
    if not setup["ready"] or setup["readiness"]["embedding_model"]["status"] != "skipped":
        raise RuntimeError(f"Expected model-free record readiness: {setup}")
    with closing(sqlite3.connect(index)) as db:
        if db.execute("PRAGMA user_version").fetchone()[0] <= 42:
            raise RuntimeError("Setup did not advance the workspace schema")
    after = invoke(*query)["data"]["rows"]
    if not any(row["record"]["record_key"] == "actions:0000000000000001" for row in after):
        raise RuntimeError("Expected Demoralize in post-upgrade keyword search")
    saved = invoke("lists", "show", "kept")["data"]["list"]
    if saved["name"] != "Retained list" or saved["list_key"] != "kept-list":
        raise RuntimeError(f"Saved list changed during setup: {saved}")
    if (root / "models").exists():
        raise RuntimeError("Model-free setup created a model cache")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="atlas-workspace-upgrade-") as directory:
        check(args.candidate.resolve(), Path(directory))
