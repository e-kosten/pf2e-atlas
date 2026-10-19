"""Publish reusable Atlas builds/data and seed independent development workspaces."""

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import uuid

import atlas_checks
from snapshots import clone_file, lock, metadata, replace_json, sha256, snapshot, write_json


def git(checkout, *arguments):
    return subprocess.check_output(["git", "-C", str(checkout), *arguments], text=True).strip()


def locations(checkout, store=None):
    root = Path(git(checkout, "rev-parse", "--show-toplevel")).resolve()
    common = Path(git(root, "rev-parse", "--path-format=absolute", "--git-common-dir"))
    return root, (root / Path(store).expanduser()).resolve() if store else common.parent / ".atlas/dev-baselines"


def load_baseline(store, selected=None, verify_data=False):
    if selected is None:
        pointer = store / "current.json"
        if not pointer.is_file():
            raise RuntimeError("No development baseline published; run dev-baseline-publish first")
        current = json.loads(pointer.read_text())
        if current.get("version") != 1:
            raise RuntimeError("Unsupported baseline pointer version")
        selected = current["baseline"]
    if not isinstance(selected, str) or not re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9._-]*", selected):
        raise RuntimeError("Invalid baseline identity")
    directory = store / "versions" / selected
    manifest = json.loads((directory / "manifest.json").read_text())
    if manifest.get("version") != 1 or manifest.get("id") != selected:
        raise RuntimeError("Unsupported or mismatched baseline manifest")
    if sha256(directory / "atlas") != manifest["build"]["sha256"]:
        raise RuntimeError("Baseline executable hash mismatch")
    if verify_data and sha256(directory / "workspace.sqlite") != manifest["database"]["sha256"]:
        raise RuntimeError("Baseline database hash mismatch")
    return directory, manifest


def publish(args, checkout, store):
    binary = args.binary.expanduser().resolve(strict=True)
    source = args.index.expanduser().resolve(strict=True)
    commit = git(checkout, "rev-parse", "--verify", args.build_commit + "^{commit}")
    store.mkdir(parents=True, exist_ok=True)
    (store / "versions").mkdir(exist_ok=True)
    with lock(store / "publish.lock"):
        identifier = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + commit[:12] + "-" + uuid.uuid4().hex[:8]
        with tempfile.TemporaryDirectory(prefix=".publish-", dir=store) as temporary:
            stage = Path(temporary)
            print("Capturing executable and consistent workspace snapshot…", file=sys.stderr, flush=True)
            original_hash = sha256(binary)
            shutil.copy2(binary, stage / "atlas")
            if sha256(stage / "atlas") != original_hash:
                raise RuntimeError("Executable changed while copying; retry with a frozen build")
            snapshot(source, stage / "workspace.sqlite")
            observed = metadata(stage / "workspace.sqlite")
            print("Validating copied data and every configured search profile…", file=sys.stderr, flush=True)
            checks = atlas_checks.validate(stage / "atlas", stage / "workspace.sqlite", {
                "source": str(args.source.resolve()) if args.source else None,
                "embedding_cache": str(args.embedding_cache_path.resolve()) if args.embedding_cache_path else None,
            }, stage)
            version = subprocess.check_output([str(stage / "atlas"), "--version"], text=True).strip()
            paths = checks["readiness"]["paths"]
            # Published resource paths must survive retirement of task checkouts.
            primary = Path(git(checkout, "rev-parse", "--path-format=absolute", "--git-common-dir")).parent.resolve()
            worktrees = [Path(line[9:]).resolve() for line in git(checkout, "worktree", "list", "--porcelain").splitlines() if line.startswith("worktree ")]
            for key in ("source", "embedding_cache"):
                resource = Path(paths[key]).resolve(strict=True)
                if any(worktree != primary and (resource == worktree or worktree in resource.parents) for worktree in worktrees):
                    raise RuntimeError(f"Shared {key} must live outside disposable worktrees: {resource}")
                paths[key] = str(resource)
            manifest = {
                "version": 1, "id": identifier,
                "created_at": datetime.now(timezone.utc).isoformat(),
                "build": {
                    "commit": commit, "profile": args.build_profile, "toolchain": args.toolchain,
                    "provenance": "publisher-declared; existing executable, no compilation performed",
                    "sha256": original_hash, "version": version,
                },
                "database": {**observed, "sha256": sha256(stage / "workspace.sqlite")},
                "assets": {key: paths[key] for key in ("source", "embedding_cache")},
                "captured_from": str(source), "validation": checks,
            }
            write_json(stage / "manifest.json", manifest)
            for filename, mode in (("atlas", 0o555), ("workspace.sqlite", 0o444), ("manifest.json", 0o444)):
                (stage / filename).chmod(mode)
            destination = store / "versions" / identifier
            # Publish the complete directory first. A failed pointer update leaves
            # the old default intact, with the new version available by identity.
            os.rename(stage, destination)
            replace_json(store / "current.json", {"version": 1, "baseline": identifier})
    return {"published": identifier, "directory": str(destination), "build": manifest["build"], "assets": manifest["assets"]}


def workspace_paths(checkout):
    directory = checkout / ".atlas"
    if directory.is_symlink():
        raise RuntimeError("Checkout .atlas must be a private directory, not a symlink")
    return directory / "workspace.sqlite", directory / "dev-workspace.json"


def initialize(args, checkout, store):
    index, receipt_path = workspace_paths(checkout)
    index.parent.mkdir(exist_ok=True)
    # Same lock order as Atlas publication, and never unlink stable lock files.
    with lock(str(index) + ".update.lock"), lock(str(index) + ".access.lock"):
        if os.path.lexists(index) or os.path.lexists(receipt_path):
            return {"initialized": False, "index": str(index), "reason": "Existing workspace or receipt preserved; no files replaced"}
        if any(os.path.lexists(str(index) + suffix) for suffix in ("-wal", "-shm", "-journal")):
            raise RuntimeError("SQLite sidecars already exist; inspect/recover this workspace before initialization")
        directory, manifest = load_baseline(store, args.baseline, verify_data=True)
        with tempfile.TemporaryDirectory(prefix=".seed-", dir=index.parent) as temporary:
            stage = Path(temporary)
            method = clone_file(directory / "workspace.sqlite", stage / "workspace.sqlite")
            (stage / "workspace.sqlite").chmod(0o600)
            if sha256(stage / "workspace.sqlite") != manifest["database"]["sha256"]:
                raise RuntimeError("Workspace copy verification failed")
            # Do not migrate with a branch binary here. Initialization is a copy;
            # status assesses the candidate, and setup is an explicit next step.
            receipt = {"version": 1, "baseline": manifest["id"], "store": str(store),
                       "seed_sha256": manifest["database"]["sha256"], "copy_method": method}
            # Exclusive destination creation also protects against non-cooperating
            # initializers. Rename only over our own empty reservation under locks.
            fd = os.open(index, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
            os.close(fd)
            try:
                os.replace(stage / "workspace.sqlite", index)
            except BaseException:
                index.unlink()
                raise
            replace_json(receipt_path, receipt)
    return {"initialized": True, "index": str(index), "baseline": manifest["id"], "copy_method": method}


def workspace_baseline(checkout, store, selected=None):
    index, receipt_path = workspace_paths(checkout)
    if receipt_path.is_file():
        receipt = json.loads(receipt_path.read_text())
        if receipt.get("version") != 1:
            raise RuntimeError("Unsupported development workspace receipt")
        store = Path(receipt["store"])
        selected = receipt["baseline"]
    directory, manifest = load_baseline(store, selected)
    return index, directory, manifest


def candidate_binary(directory, explicit=None):
    if explicit:
        return explicit.expanduser().resolve(strict=True), "explicit candidate"
    return directory / "atlas", "frozen baseline build (use --binary for a candidate)"


def web_command(binary, index, assets, port):
    return [str(binary), "web", "--path-mode", "global", "--index", str(index),
            "--source", assets["source"], "--embedding-cache-path", assets["embedding_cache"], "--port", str(port)]


def status(args, checkout, store):
    index, directory, manifest = workspace_baseline(checkout, store, args.baseline)
    binary, selection = candidate_binary(directory, args.binary)
    result = {"baseline": manifest["id"], "baseline_directory": str(directory),
              "build": manifest["build"], "seed": manifest["database"], "assets": manifest["assets"],
              "checkout": str(checkout), "checkout_commit": git(checkout, "rev-parse", "HEAD"),
              "binary": str(binary), "binary_selection": selection, "index": str(index), "exists": index.is_file()}
    if index.is_file():
        result["readiness"] = atlas_checks.readiness(binary, index, manifest["assets"], checkout)
        result["compatibility"] = atlas_checks.invoke(binary, ["index", "check", "--path-mode", "global", "--index", index, "--json"], checkout)
    else:
        result["next_step"] = "Run just dev-workspace-init to create a private working copy"
    result["web_command"] = shlex.join(web_command(binary, index, manifest["assets"], args.port))
    result["setup_command"] = shlex.join([str(binary), "setup", "--offline", "--path-mode", "global", "--index", str(index),
                                         "--source", manifest["assets"]["source"], "--embedding-cache-path", manifest["assets"]["embedding_cache"]])
    return result


def web(args, checkout, store):
    index, directory, manifest = workspace_baseline(checkout, store, args.baseline)
    if not index.is_file():
        raise RuntimeError("No private workspace; run just dev-workspace-init first")
    binary, selection = candidate_binary(directory, args.binary)
    ready = atlas_checks.require_ok(atlas_checks.readiness(binary, index, manifest["assets"], checkout))
    if not ready["ready"]:
        raise RuntimeError("Candidate requires setup; run just dev-status to see the explicit setup command")
    command = web_command(binary, index, manifest["assets"], args.port)
    print(selection + "\n" + shlex.join(command), file=sys.stderr, flush=True)
    return subprocess.call(command, cwd=checkout)


def print_result(command, result):
    if command == "publish":
        print(f"Published {result['published']}\n{result['directory']}")
    elif command == "init":
        if result["initialized"]:
            print(f"Initialized {result['index']}\nBaseline: {result['baseline']}\nCopy: {result['copy_method']}")
        else:
            print(f"{result['reason']}\n{result['index']}")
    else:
        print(f"Baseline: {result['baseline']}\nBuild: {result['build']['commit'][:12]} ({result['build']['profile']}, Rust {result['build']['toolchain']})")
        print(f"Executable: {result['binary']}\nSelection: {result['binary_selection']}\nWorkspace: {result['index']}")
        if not result["exists"]:
            print(result["next_step"])
            return
        ready = result["readiness"]["response"].get("data", {})
        compatible = result["compatibility"]
        good = not compatible["exit_code"] and compatible["response"].get("status") == "ok"
        print(f"Compatible: {'yes' if good else 'no'}; ready: {'yes' if ready.get('ready') else 'no'}")
        for profile in ready.get("search_profiles", []):
            print(f"  {profile['profile_id']}: {'ready' if profile['ready'] else 'not ready'}")
        if not good or not ready.get("ready"):
            print("Inspect --json for check details. Explicit setup command:\n" + result["setup_command"])
        else:
            print("Backend command:\n" + result["web_command"])


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--checkout", type=Path, default=Path.cwd(), help="Checkout to seed/run; defaults to cwd")
    common.add_argument("--store", type=Path, help="Shared store; default: primary checkout .atlas/dev-baselines")
    common.add_argument("--json", action="store_true", help="Emit complete command report (except web)")
    commands = result.add_subparsers(dest="command", required=True)
    publish_parser = commands.add_parser("publish", parents=[common], help="Validate and publish an existing build and consistent database snapshot")
    publish_parser.add_argument("--binary", type=Path, required=True)
    publish_parser.add_argument("--index", type=Path, required=True, help="Explicit seed workspace; all saved state is copied")
    publish_parser.add_argument("--build-commit", required=True, help="Commit that produced the supplied binary; publisher attests provenance")
    publish_parser.add_argument("--build-profile", required=True, choices=["dev", "debug-full", "release", "dist"])
    publish_parser.add_argument("--toolchain", required=True, help="Rust toolchain used for this binary")
    publish_parser.add_argument("--source", type=Path)
    publish_parser.add_argument("--embedding-cache-path", type=Path)
    for name, help_text in (("init", "Initialize only a missing private workspace"), ("status", "Show baseline provenance and candidate compatibility; never migrate"), ("web", "Run a backend with explicit private workspace and shared asset paths")):
        command = commands.add_parser(name, parents=[common], help=help_text)
        command.add_argument("--baseline", help="Published identity; default: current (existing workspaces retain their seed)")
        if name != "init":
            command.add_argument("--binary", type=Path, help="Explicit candidate executable; default: frozen seed binary")
            command.add_argument("--port", type=int, default=4727)
    return result


def main():
    args = parser().parse_args()
    try:
        if os.name != "posix":
            raise RuntimeError("Development baseline commands currently support macOS and Linux")
        checkout, store = locations(args.checkout, args.store)
        for key in ("binary", "index", "source", "embedding_cache_path"):
            value = getattr(args, key, None)
            if value is not None:
                setattr(args, key, checkout / value.expanduser())
        if args.command == "web":
            return web(args, checkout, store)
        result = {"publish": publish, "init": initialize, "status": status}[args.command](args, checkout, store)
        if args.json:
            print(json.dumps(result, indent=2, sort_keys=True))
        else:
            print_result(args.command, result)
        if args.command == "status" and result["exists"]:
            ready = result["readiness"]
            compatible = result["compatibility"]
            if (ready["exit_code"] or not ready["response"].get("data", {}).get("ready", False)
                    or compatible["exit_code"] or compatible["response"].get("status") != "ok"):
                return 1
        return 0
    except (OSError, ValueError, KeyError, RuntimeError, sqlite3.Error, subprocess.CalledProcessError) as error:
        print("Development baseline: " + str(error), file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    sys.exit(main())
