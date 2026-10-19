"""Delegate all executable/data compatibility decisions to Atlas itself."""

import json
import subprocess


def invoke(binary, arguments, cwd):
    result = subprocess.run([str(binary), *map(str, arguments)], cwd=cwd, capture_output=True, text=True)
    try:
        envelope = json.loads(result.stdout)
    except ValueError:
        envelope = {"status": "error", "message": result.stderr.strip() or result.stdout.strip()}
    return {"exit_code": result.returncode, "response": envelope}


def require_ok(result):
    if result["exit_code"] or result["response"].get("status") != "ok":
        raise RuntimeError("Atlas check failed: " + json.dumps(result))
    return result["response"]["data"]


def readiness(binary, index, assets, cwd):
    arguments = ["setup", "--check", "--offline", "--path-mode", "global", "--index", index, "--json"]
    for key, option in (("source", "--source"), ("embedding_cache", "--embedding-cache-path")):
        if assets.get(key):
            arguments.extend([option, assets[key]])
    return invoke(binary, arguments, cwd)


def validate(binary, index, assets, cwd):
    ready = require_ok(readiness(binary, index, assets, cwd))
    if not ready["ready"] or not ready["search_profiles"] or not all(p["ready"] for p in ready["search_profiles"]):
        raise RuntimeError("Baseline needs ready records and every configured search profile: " + json.dumps(ready))
    deep = require_ok(invoke(binary, ["index", "validate", "--path-mode", "global", "--index", index, "--json"], cwd))
    searches = []
    for profile in ready["search_profiles"]:
        report = require_ok(invoke(binary, [
            "search", "healing", "--retrieval", "vector", "--limit", "1", "--search-profile", profile["profile_id"],
            "--path-mode", "global", "--index", index,
            "--embedding-cache-path", ready["paths"]["embedding_cache"], "--json",
        ], cwd))
        if not report["results"]:
            raise RuntimeError("Semantic smoke query returned no results for " + profile["profile_id"])
        searches.append({"profile": profile["profile_id"], "result_count": len(report["results"])})
    return {"readiness": ready, "deep_validation": deep, "searches": searches}
