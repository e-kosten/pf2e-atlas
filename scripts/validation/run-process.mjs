import { spawn, spawnSync } from "node:child_process";
import { constants } from "node:os";
import { readFileSync } from "node:fs";

// This supervisor owns the process group it creates. It never discovers or
// signals processes through saved PID files or a system-wide process scan.
let argv = process.argv.slice(2);
let cancelFile;
if (argv[0] === "--cancel-file" && argv[2] === "--") {
  cancelFile = argv[1];
  argv = argv.slice(3);
}
const [command, ...args] = argv;
if (!command) {
  console.error("Usage: node run-process.mjs COMMAND [ARG...]");
  process.exit(2);
}
const windows = process.platform === "win32";
let child;
let interrupted;
let commandStatus;
let cleaning = false;
let poll;
let escalation;
let deadline;
let cancellationPoll;

function signalGroup(signal) {
  if (!child?.pid) return false;
  try {
    process.kill(-child.pid, signal);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}
function finish() {
  clearInterval(poll);
  clearInterval(cancellationPoll);
  clearTimeout(escalation);
  clearTimeout(deadline);
  process.exit(interrupted ?? commandStatus ?? 125);
}
function cleanup() {
  if (cleaning || !child?.pid) return;
  cleaning = true;
  if (windows) {
    // taskkill acts only on the live Child we own. Windows has no POSIX group
    // signal; never reinterpret a saved PID or scan for unrelated processes.
    if (child.exitCode === null && child.signalCode === null) {
      const stopped = spawnSync(
        "taskkill",
        ["/PID", String(child.pid), "/T", "/F"],
        { stdio: "ignore", windowsHide: true, timeout: 1500 },
      );
      if (stopped.error)
        console.error(`Validation process cleanup: ${stopped.error.message}`);
    }
    return finish();
  }
  if (!signalGroup("SIGTERM")) return finish();
  poll = setInterval(() => {
    if (!signalGroup(0)) finish();
  }, 25);
  escalation = setTimeout(() => signalGroup("SIGKILL"), 500);
  deadline = setTimeout(() => {
    signalGroup("SIGKILL");
    finish();
  }, 1500);
}
function checkCancellation() {
  if (!cancelFile) return;
  let requested;
  try {
    requested = Number(readFileSync(cancelFile, "utf8").trim());
  } catch (error) {
    if (error.code === "ENOENT") return;
    console.error(error.message);
    requested = 125;
  }
  interrupted ??= [129, 130, 143].includes(requested) ? requested : 125;
  cleanup();
}
for (const signal of ["SIGHUP", "SIGINT", "SIGTERM"]) {
  process.on(signal, () => {
    interrupted ??= 128 + constants.signals[signal];
    cleanup();
  });
}
// Git Bash owns extensionless shebang fixtures and npm.cmd dispatch on Windows.
// Pass argv through unchanged, without interpolating a command string.
child = windows
  ? spawn("sh", ["-c", 'exec "$@"', "sh", command, ...args], {
      stdio: "inherit",
      windowsHide: true,
    })
  : spawn(command, args, { detached: true, stdio: "inherit" });
child.on("error", (error) => {
  console.error(error.message);
  commandStatus = error.code === "ENOENT" ? 127 : 126;
  finish();
});
child.on("exit", (code, signal) => {
  commandStatus = code ?? 128 + constants.signals[signal];
  cleanup();
});
if (cancelFile) {
  cancellationPoll = setInterval(checkCancellation, 100);
  checkCancellation();
}
if (interrupted !== undefined) cleanup();
