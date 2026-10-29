#!/bin/sh
# Sourced by verification entrypoints. Logs survive callers' temporary-directory traps.
# Child-shell fragments deliberately expand in their own process.
# shellcheck disable=SC2016

atlas_validation_inputs() {
  atlas_inputs_tmp=$(mktemp -d "$ATLAS_VALIDATION_RUN_DIR/.inputs.XXXXXX")
  git ls-files --cached --others --exclude-standard --deduplicate -z -- . ":(top,exclude).atlas/validation" >"$atlas_inputs_tmp/list" || return
  LC_ALL=C sort -zu "$atlas_inputs_tmp/list" >"$atlas_inputs_tmp/sorted" || return
  # Ignore unrelated prose for code gates. Package fixture/evidence/skill Markdown
  # is executable input; frontend formatting also consumes its package Markdown.
  xargs -0 sh -c '
    scope=$1; shift
    for path do
      if [ ! -e "$path" ] && [ ! -L "$path" ]; then continue; fi
      case "$scope" in
        web-format)
          case "$path" in web/atlas-ui/* | .prettier* | .editorconfig | prettier.config.* | package.json | package-lock.json) ;; *) continue ;; esac ;;
        rust-format)
          case "$path" in *.rs | Cargo.toml | Cargo.lock | */Cargo.toml | rust-toolchain* | .rustfmt.toml | rustfmt.toml | */.rustfmt.toml | */rustfmt.toml | .cargo/*) ;; *) continue ;; esac ;;
        code)
          case "$path" in
            dev-tools/source-contracts/* | docs/research/evaluations/* | skills/pf2e-atlas-cli/*) ;;
            *.md) continue ;;
          esac ;;
      esac
      printf "%s\000" "$path"
    done
  ' sh "$1" <"$atlas_inputs_tmp/sorted" >"$atlas_inputs_tmp/paths" || return
  xargs -0 sh -c '
    for path do
      if [ -L "$path" ]; then printf "link:%s\000" "$(readlink "$path")"
      elif [ -f "$path" ]; then
        if [ -x "$path" ]; then printf "executable\000"; else printf "file\000"; fi
      elif [ -e "$path" ]; then printf "other\000"
      else printf "deleted\000"; fi
    done
  ' sh <"$atlas_inputs_tmp/paths" >"$atlas_inputs_tmp/modes" || return
  xargs -0 sh -c '
    for path do [ ! -f "$path" ] || printf "%s\000" "$path"; done
  ' sh <"$atlas_inputs_tmp/paths" >"$atlas_inputs_tmp/files" || return
  xargs -0 git hash-object -- <"$atlas_inputs_tmp/files" >"$atlas_inputs_tmp/hashes" || return
  cat "$atlas_inputs_tmp/paths" "$atlas_inputs_tmp/modes" "$atlas_inputs_tmp/hashes" >"$atlas_inputs_tmp/content" || return
  git hash-object --stdin <"$atlas_inputs_tmp/content"
  atlas_inputs_status=$?
  rm -rf "$atlas_inputs_tmp"
  return "$atlas_inputs_status"
}

atlas_validation_init() {
  atlas_validation_kind=${atlas_validation_kind:-final}
  if [ -n "${ATLAS_VALIDATION_RUN_DIR:-}" ] && [ -f "$ATLAS_VALIDATION_RUN_DIR/candidate.tsv" ]; then
    atlas_validation_checkout=$(awk -F '\t' '$1 == "checkout" { print $2 }' "$ATLAS_VALIDATION_RUN_DIR/candidate.tsv")
    atlas_validation_previous_kind=$(awk -F '\t' '$1 == "kind" { print $2 }' "$ATLAS_VALIDATION_RUN_DIR/candidate.tsv")
    if [ "$atlas_validation_checkout" = "$(git rev-parse --show-toplevel)" ] && [ "$atlas_validation_previous_kind" = "$atlas_validation_kind" ]; then return 0; fi
  fi
  command -v node >/dev/null 2>&1 || { echo "Node.js is required for validation process cleanup." >&2; return 127; }
  atlas_validation_root=${ATLAS_VALIDATION_OUTPUT_ROOT:-$(git rev-parse --show-toplevel)/.atlas/validation}
  mkdir -p "$atlas_validation_root"
  ATLAS_VALIDATION_RUN_DIR=$(mktemp -d "$atlas_validation_root/run.XXXXXX")
  ATLAS_VALIDATION_RUN_DIR=$(CDPATH='' cd -- "$ATLAS_VALIDATION_RUN_DIR" && pwd -P)
  export ATLAS_VALIDATION_RUN_DIR
  atlas_validation_start_inputs=per-check
  if [ "$atlas_validation_kind" = final ]; then atlas_validation_start_inputs=$(atlas_validation_inputs code) || return; fi
  {
    printf 'kind\t%s\n' "$atlas_validation_kind"
    printf 'checkout\t%s\n' "$(git rev-parse --show-toplevel)"
    printf 'start_commit\t%s\n' "$(git rev-parse HEAD)"
    printf 'start_inputs\t%s\n' "$atlas_validation_start_inputs"
    printf 'started_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  } >"$ATLAS_VALIDATION_RUN_DIR/candidate.tsv"
  printf 'Validation evidence: %s\n' "$ATLAS_VALIDATION_RUN_DIR" >&2
}

atlas_validation_heartbeat() {
  atlas_sleep_pid=
  trap '[ -z "$atlas_sleep_pid" ] || kill "$atlas_sleep_pid" 2>/dev/null || true; exit 0' HUP INT TERM
  while :; do
    sleep "$atlas_check_heartbeat" & atlas_sleep_pid=$!
    wait "$atlas_sleep_pid" || exit 0
    printf 'Still running %s (%ss); log: %s\n' "$atlas_check_label" "$(($(date +%s) - atlas_check_started))" "$atlas_check_log" >&2
  done
}

atlas_validation_check() {
  atlas_check_label=$1; shift
  atlas_check_started=$(date +%s)
  atlas_check_dir=$(mktemp -d "$ATLAS_VALIDATION_RUN_DIR/check.XXXXXX")
  atlas_check_log="$atlas_check_dir/output.log"
  : >"$atlas_check_log"
  atlas_check_status=0
  atlas_check_state=running
  atlas_check_pid=
  atlas_check_tee_pid=
  atlas_check_heartbeat_pid=
  atlas_check_kind=${atlas_validation_kind:-final}
  atlas_check_scope=code
  if [ "$atlas_check_kind" = iteration ]; then
    case "$atlas_check_label" in
      'web UI format') atlas_check_scope=web-format ;;
      'cargo fmt') atlas_check_scope=rust-format ;;
    esac
  fi
  atlas_check_start_commit=$(git rev-parse HEAD)
  atlas_check_start_inputs=$(atlas_validation_inputs "$atlas_check_scope") || return
  atlas_check_candidate_inputs=$(awk -F '\t' '$1 == "start_inputs" { print $2 }' "$ATLAS_VALIDATION_RUN_DIR/candidate.tsv")
  atlas_check_current_inputs=$atlas_check_start_inputs
  if [ "$atlas_check_kind" = final ] && [ "$atlas_check_scope" != code ]; then atlas_check_current_inputs=$(atlas_validation_inputs code); fi
  atlas_check_heartbeat=${ATLAS_VALIDATION_HEARTBEAT_SECONDS:-20}
  case "$atlas_check_heartbeat" in '' | *[!0-9]*) atlas_check_heartbeat=20 ;; esac
  if [ "$atlas_check_heartbeat" -lt 1 ] || [ "$atlas_check_heartbeat" -gt 30 ]; then atlas_check_heartbeat=20; fi
  printf '%s\n' "$@" >"$atlas_check_dir/command.args"
  sh -c 'printf "%s\n" "$PPID"' >"$atlas_check_dir/phase.pid"
  node --version >"$atlas_check_dir/toolchain.txt" 2>&1
  case "$1" in
    cargo | */verify.sh) { rustc --version; cargo --version; } >>"$atlas_check_dir/toolchain.txt" 2>&1 || true ;;
    npm) npm --version >>"$atlas_check_dir/toolchain.txt" 2>&1 || true ;;
  esac
  # shellcheck disable=SC2329 # Called by the EXIT trap.
  atlas_validation_finish() {
    atlas_finish_status=$?
    if [ "$atlas_check_status" -eq 0 ] && [ "$atlas_finish_status" -ne 0 ]; then atlas_check_status=$atlas_finish_status; fi
    trap '' HUP INT TERM USR1
    [ -z "$atlas_check_heartbeat_pid" ] || { kill "$atlas_check_heartbeat_pid" 2>/dev/null || true; wait "$atlas_check_heartbeat_pid" 2>/dev/null || true; }
    if [ -n "$atlas_check_pid" ]; then
      # A native Windows SIGTERM can bypass Node's handlers. Request cancellation
      # through this owned check's channel and wait for descendant cleanup.
      printf '%s\n' "$atlas_check_status" >"$atlas_check_dir/cancel.tmp"
      mv "$atlas_check_dir/cancel.tmp" "$atlas_check_dir/cancel"
      wait "$atlas_check_pid" 2>/dev/null || true
    fi
    [ -z "$atlas_check_tee_pid" ] || { kill "$atlas_check_tee_pid" 2>/dev/null || true; wait "$atlas_check_tee_pid" 2>/dev/null || true; }
    rm -f "$atlas_check_dir/output.pipe"
    atlas_check_end_commit=$(git rev-parse HEAD)
    atlas_check_end_inputs=$(atlas_validation_inputs "$atlas_check_scope") || atlas_check_end_inputs=unavailable
    atlas_check_run_end_inputs=$atlas_check_end_inputs
    if [ "$atlas_check_kind" = iteration ]; then atlas_check_run_end_inputs=per-check
    elif [ "$atlas_check_scope" != code ]; then atlas_check_run_end_inputs=$(atlas_validation_inputs code) || atlas_check_run_end_inputs=unavailable; fi
    atlas_check_elapsed=$(($(date +%s) - atlas_check_started))
    if [ "$atlas_check_state" != interrupted ] && [ "$atlas_check_state" != stale ]; then
      atlas_check_state=passed
      if [ "$atlas_check_status" -ne 0 ]; then atlas_check_state=failed
      elif [ "$atlas_check_start_inputs" != "$atlas_check_end_inputs" ] || { [ "$atlas_check_kind" = final ] && [ "$atlas_check_candidate_inputs" != "$atlas_check_run_end_inputs" ]; }; then atlas_check_state=stale; atlas_check_status=78; fi
    fi
    {
      printf 'kind\t%s\n' "$atlas_check_kind"
      printf 'label\t%s\nstatus\t%s\nexit\t%s\nduration_seconds\t%s\n' "$atlas_check_label" "$atlas_check_state" "$atlas_check_status" "$atlas_check_elapsed"
      printf 'start_commit\t%s\nend_commit\t%s\n' "$atlas_check_start_commit" "$atlas_check_end_commit"
      printf 'input_scope\t%s\nstart_inputs\t%s\nend_inputs\t%s\n' "$atlas_check_scope" "$atlas_check_start_inputs" "$atlas_check_end_inputs"
      printf 'candidate_start_inputs\t%s\ncandidate_end_inputs\t%s\n' "$atlas_check_candidate_inputs" "$atlas_check_run_end_inputs"
      printf 'log\t%s\ncommand\t%s\nheartbeat_pid\t%s\n' "$atlas_check_log" "$atlas_check_dir/command.args" "$atlas_check_heartbeat_pid"
    } >"$atlas_check_dir/result.tsv"
    printf '%s\t%s\t%s\t%s\n' "$atlas_check_state" "$atlas_check_label" "$atlas_check_elapsed" "$atlas_check_dir/result.tsv" >>"$ATLAS_VALIDATION_RUN_DIR/checks.tsv"
    if [ "$atlas_check_status" -eq 0 ]; then
      printf 'Passed %s (%ss); log: %s\n' "$atlas_check_label" "$atlas_check_elapsed" "$atlas_check_log" >&2
    else
      printf '\nFailed %s (%ss, %s); evidence: %s\n' "$atlas_check_label" "$atlas_check_elapsed" "$atlas_check_state" "$atlas_check_dir/result.tsv" >&2
      [ "${verbose:-0}" -ne 0 ] || cat "$atlas_check_log" >&2
    fi
    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
      printf -- '- %s: %ss (exit %s), %s; log: %s\n' "$atlas_check_label" "$atlas_check_elapsed" "$atlas_check_status" "$atlas_check_state" "$atlas_check_log" >>"$GITHUB_STEP_SUMMARY"
    fi
    exit "$atlas_check_status"
  }
  trap atlas_validation_finish EXIT
  trap 'atlas_check_status=129; atlas_check_state=interrupted; exit 129' HUP
  trap 'atlas_check_status=130; atlas_check_state=interrupted; exit 130' INT USR1
  trap 'atlas_check_status=143; atlas_check_state=interrupted; exit 143' TERM
  if [ "$atlas_check_kind" = final ] && [ "$atlas_check_candidate_inputs" != "$atlas_check_current_inputs" ]; then
    printf 'Candidate inputs changed between checks; start a fresh validation run.\n' >"$atlas_check_log"
    atlas_check_state=stale
    atlas_check_status=78
    exit 78
  fi
  printf 'Running %s...\n' "$atlas_check_label" >&2
  if [ "${verbose:-0}" -eq 1 ]; then
    mkfifo "$atlas_check_dir/output.pipe"
    tee "$atlas_check_log" <"$atlas_check_dir/output.pipe" >&2 & atlas_check_tee_pid=$!
    node "$(git rev-parse --show-toplevel)/scripts/validation/run-process.mjs" --cancel-file "$atlas_check_dir/cancel" -- "$@" >"$atlas_check_dir/output.pipe" 2>&1 & atlas_check_pid=$!
  else
    node "$(git rev-parse --show-toplevel)/scripts/validation/run-process.mjs" --cancel-file "$atlas_check_dir/cancel" -- "$@" >"$atlas_check_log" 2>&1 & atlas_check_pid=$!
  fi
  atlas_validation_heartbeat & atlas_check_heartbeat_pid=$!
  wait "$atlas_check_pid" || atlas_check_status=$?
  atlas_check_pid=
  if [ -n "$atlas_check_tee_pid" ]; then wait "$atlas_check_tee_pid" || atlas_check_status=$?; atlas_check_tee_pid=; fi
  exit "$atlas_check_status"
}

# Forward signals while waiting for an owned phase or self-reporting composite.
# A composite keeps its own phase supervisors, rather than another detached group.
atlas_validation_run_owned() {
  atlas_validation_interrupt_signal=$1; shift
  # dash resets traps in command substitutions. Capture them in the caller's
  # shell instead, then source the builtin's shell-quoted restoration commands.
  atlas_validation_saved_traps_file=$(mktemp "${TMPDIR:-/tmp}/atlas-caller-traps.XXXXXX")
  trap >"$atlas_validation_saved_traps_file"
  atlas_validation_phase_pid=
  atlas_validation_interrupted=0
  trap 'atlas_validation_interrupted=129; [ -z "$atlas_validation_phase_pid" ] || kill -HUP "$atlas_validation_phase_pid" 2>/dev/null || true' HUP
  # POSIX asynchronous shell jobs inherit ignored SIGINT. An owned phase handles
  # USR1 as Ctrl-C; a composite shell forwards TERM through its active phase.
  trap 'atlas_validation_interrupted=130; [ -z "$atlas_validation_phase_pid" ] || kill -"$atlas_validation_interrupt_signal" "$atlas_validation_phase_pid" 2>/dev/null || true' INT
  trap 'atlas_validation_interrupted=143; [ -z "$atlas_validation_phase_pid" ] || kill -TERM "$atlas_validation_phase_pid" 2>/dev/null || true' TERM
  "$@" & atlas_validation_phase_pid=$!
  while :; do
    atlas_validation_phase_status=0
    wait "$atlas_validation_phase_pid" || atlas_validation_phase_status=$?
    kill -0 "$atlas_validation_phase_pid" 2>/dev/null || break
  done
  atlas_validation_phase_pid=
  trap - HUP INT TERM
  # The private file contains only freshly captured builtin trap output.
  # shellcheck disable=SC1090
  . "$atlas_validation_saved_traps_file"
  rm -f "$atlas_validation_saved_traps_file"
  if [ "$atlas_validation_interrupted" -ne 0 ]; then return "$atlas_validation_interrupted"; fi
  return "$atlas_validation_phase_status"
}

run_check() {
  atlas_validation_init || return
  atlas_validation_run_owned USR1 atlas_validation_check "$@"
}

run_composite() {
  atlas_validation_run_owned TERM "$@"
}

# The same owner can wrap frontend/other explicit gates without another runner.
if [ "${0##*/}" = run-check.sh ]; then
  set -eu
  verbose=0
  if [ "${1:-}" = --verbose ]; then verbose=1; shift; fi
  if [ "$#" -lt 2 ]; then
    printf 'Usage: sh scripts/validation/run-check.sh [--verbose] LABEL COMMAND [ARG...]\n' >&2
    exit 2
  fi
  cd "$(git rev-parse --show-toplevel)"
  run_check "$@"
fi
