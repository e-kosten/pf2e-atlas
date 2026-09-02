#!/bin/sh
set -eu

repo_root=$(git rev-parse --show-toplevel)
evidence_path="$repo_root/crates/atlas-cli/tests/goldens/record_text/snapshot-evidence.json"
output_path=${1:-"$repo_root/scratch/b5-cli-b-final/manifest.json"}
expected_parent=d595966a11502f6eca0d4580a500e003619f1613

candidate_commit=$(git rev-parse HEAD)
candidate_tree=$(git rev-parse HEAD^{tree})
candidate_parent=$(git rev-parse HEAD^)
if [ "$candidate_parent" != "$expected_parent" ]; then
    echo "candidate parent mismatch: expected $expected_parent, got $candidate_parent" >&2
    exit 1
fi
if [ "$(jq -r '.remediation_base' "$evidence_path")" != "$candidate_parent" ]; then
    echo "snapshot evidence does not bind the candidate parent" >&2
    exit 1
fi
git verify-commit "$candidate_commit" >/dev/null 2>&1

evidence_sha256=$(shasum -a 256 "$evidence_path" | awk '{print $1}')
candidate_signer_fingerprint=$(git log -1 --format=%GF "$candidate_commit")
mkdir -p "$(dirname "$output_path")"
temporary_path="$output_path.tmp"
jq -n \
    --arg format "pf2e-atlas-cli-text-snapshot-manifest/v1" \
    --arg candidate_commit "$candidate_commit" \
    --arg candidate_tree "$candidate_tree" \
    --arg candidate_parent "$candidate_parent" \
    --arg candidate_signature "verified_good" \
    --arg candidate_signer_fingerprint "$candidate_signer_fingerprint" \
    --arg evidence_path "crates/atlas-cli/tests/goldens/record_text/snapshot-evidence.json" \
    --arg evidence_sha256 "$evidence_sha256" \
    --slurpfile evidence "$evidence_path" \
    '{
        format: $format,
        candidate: {
            commit: $candidate_commit,
            tree: $candidate_tree,
            parent: $candidate_parent,
            signature: $candidate_signature,
            signer_fingerprint: $candidate_signer_fingerprint
        },
        committed_evidence: {
            path: $evidence_path,
            sha256: $evidence_sha256,
            value: $evidence[0]
        }
    }' >"$temporary_path"
mv "$temporary_path" "$output_path"
shasum -a 256 "$output_path"
