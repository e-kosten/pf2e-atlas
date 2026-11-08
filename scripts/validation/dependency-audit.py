#!/usr/bin/env python3
"""Audit Atlas's committed lockfiles without installing packages."""
from datetime import UTC, datetime, date
import json
from pathlib import Path
import subprocess
import sys
import tomllib
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[2]
NPM_PROJECTS = (
    "web/atlas-ui",
    "dev-tools/source-contracts",
    "dev-tools/release",
    "dev-tools/openapi",
    "dev-tools/dice-prototype/renderer-deps",
)
CARGO_LOCKFILES = ("Cargo.lock", "dev-tools/dice-prototype/server/Cargo.lock")
OSV_API = "https://api.osv.dev/v1/"
MINIMATCH_REVIEW = date(2026, 12, 4)


def osv(path, payload=None):
    req = Request(
        OSV_API + path,
        data=json.dumps(payload).encode() if payload is not None else None,
        headers={"Content-Type": "application/json", "User-Agent": "PF2e Atlas dependency audit"},
    )
    with urlopen(req, timeout=30) as response:
        return json.load(response)


def audit_cargo():
    entries = {}
    for path in CARGO_LOCKFILES:
        lock = tomllib.loads((ROOT / path).read_text())
        for entry in lock["package"]:
            if entry.get("source", "").startswith("registry+"):
                entries.setdefault((entry["name"], entry["version"]), []).append(path)
    queries = [{"package": {"name": name, "ecosystem": "crates.io"}, "version": version}
               for name, version in entries]
    result = osv("querybatch", {"queries": queries})["results"]
    if len(result) != len(entries) or any(item.get("next_page_token") for item in result):
        raise ValueError("OSV returned incomplete Cargo audit results")
    active = 0
    for (name, version), item in zip(entries, result, strict=True):
        for match in item.get("vulns", []):
            identifier = match["id"]
            advisory = osv("vulns/" + identifier)
            affected = [entry for entry in advisory["affected"]
                        if entry["package"]["name"] == name]
            unmaintained = bool(affected) and all(
                (entry.get("database_specific") or {}).get("informational") == "unmaintained"
                for entry in affected
            )
            label = "maintenance warning" if unmaintained else "advisory"
            print(f"{label}: {identifier} {name}@{version} ({', '.join(entries[name, version])})")
            active += not unmaintained
    print(f"OSV scanned {len(entries)} distinct locked Rust versions.")
    return active


def audit_npm():
    active = 0
    for project in NPM_PROJECTS:
        scan = subprocess.run(
            ["npm", "audit", "--package-lock-only", "--ignore-scripts", "--include=dev", "--json"],
            cwd=ROOT / project, text=True, capture_output=True, timeout=90, check=False,
        )
        try:
            report = json.loads(scan.stdout)
            vulnerabilities = report["vulnerabilities"]
            total = report["metadata"]["vulnerabilities"]["total"]
        except (ValueError, KeyError) as exc:
            raise ValueError(f"npm advisory scan failed for {project}: {scan.stderr}") from exc
        if scan.returncode not in (0, 1) or total != len(vulnerabilities):
            raise ValueError(f"npm advisory scan returned incomplete results for {project}")
        packages = json.loads((ROOT / project / "package-lock.json").read_text())["packages"]
        direct = 0
        for name, entry in vulnerabilities.items():
            for advisory in (a for a in entry["via"] if isinstance(a, dict)):
                direct += 1
                nodes = entry["nodes"]
                if not nodes:
                    raise ValueError(f"npm audit cannot locate {name} in {project}")
                for node in nodes:
                    package = packages[node]
                    accepted = (
                        project == "web/atlas-ui"
                        and node == "node_modules/@typescript-eslint/typescript-estree/node_modules/minimatch"
                        and package["version"] == "10.2.6"
                        and package.get("dev") is True
                        and advisory["url"] == "https://github.com/advisories/GHSA-g3hv-75wg-m249"
                        and advisory["severity"] == "moderate"
                        and datetime.now(UTC).date() < MINIMATCH_REVIEW
                    )
                    label = "scoped exception" if accepted else "advisory"
                    print(f"{label}: {project}/{node}@{package['version']} {advisory['url']}")
                    active += not accepted
        if total and not direct:
            raise ValueError(f"npm audit did not identify the underlying advisory in {project}")
        print(f"npm scanned {project}.")
    return active


if __name__ == "__main__":
    try:
        findings = audit_cargo() + audit_npm()
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        sys.exit(f"Dependency audit could not complete: {error}")
    sys.exit(1 if findings else 0)
