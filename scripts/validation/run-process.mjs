import { spawn, spawnSync } from "node:child_process";
import { constants } from "node:os";

export function runProcess(command, args, { signal, onOutput }) {
  const interruptedStatus = () =>
    [129, 130, 143].includes(signal.reason) ? signal.reason : 125;
  if (signal.aborted) return Promise.resolve(interruptedStatus());
  return new Promise((resolve) => {
    const windows = process.platform === "win32";
    let child;
    let interrupted;
    let commandStatus;
    let outputFailed = false;
    let cleaning = false;
    let cleanupComplete = false;
    let childClosed = false;
    let finished = false;
    let poll;
    let escalation;
    let deadline;

    function output(chunk) {
      if (outputFailed) return;
      try {
        onOutput(chunk);
      } catch (error) {
        outputFailed = true;
        commandStatus = 125;
        console.error(`Validation output: ${error.message}`);
        if (child) cleanup();
      }
    }
    function diagnostic(message) {
      if (!outputFailed) output(Buffer.from(`${message}\n`));
      else console.error(message);
    }
    function signalGroup(requested) {
      try {
        process.kill(-child.pid, requested);
        return true;
      } catch (error) {
        if (error.code !== "ESRCH") {
          diagnostic(`Validation process cleanup: ${error.message}`);
          commandStatus = 125;
        }
        return false;
      }
    }
    function finish() {
      if (finished || !cleanupComplete || !childClosed) return;
      finished = true;
      clearInterval(poll);
      clearTimeout(escalation);
      clearTimeout(deadline);
      signal.removeEventListener("abort", abort);
      resolve(interrupted ?? commandStatus ?? 125);
    }
    function cleanup() {
      if (cleaning) return;
      cleaning = true;
      deadline = setTimeout(() => {
        if (!windows && child.pid) signalGroup("SIGKILL");
        child.stdout?.destroy();
        child.stderr?.destroy();
        cleanupComplete = childClosed = true;
        finish();
      }, 1500);
      if (!child.pid) {
        cleanupComplete = true;
      } else if (windows) {
        if (child.exitCode === null && child.signalCode === null) {
          const stopped = spawnSync(
            "taskkill",
            ["/PID", String(child.pid), "/T", "/F"],
            { stdio: "ignore", windowsHide: true, timeout: 1500 },
          );
          if (stopped.error || stopped.status !== 0)
            diagnostic(
              `Validation process cleanup: ${stopped.error?.message ?? `taskkill exited ${stopped.status}`}`,
            );
        }
        cleanupComplete = true;
      } else if (!signalGroup("SIGTERM")) {
        cleanupComplete = true;
      } else {
        poll = setInterval(() => {
          if (!signalGroup(0)) {
            cleanupComplete = true;
            clearInterval(poll);
            clearTimeout(escalation);
            finish();
          }
        }, 25);
        escalation = setTimeout(() => signalGroup("SIGKILL"), 500);
      }
      finish();
    }
    function abort() {
      interrupted ??= interruptedStatus();
      cleanup();
    }
    const options = {
      stdio: ["ignore", "pipe", "pipe"],
      windowsHide: true,
    };
    try {
      child = windows
        ? spawn("sh", ["-c", 'exec "$@"', "sh", command, ...args], options)
        : spawn(command, args, { ...options, detached: true });
    } catch (error) {
      commandStatus = error.code === "ENOENT" ? 127 : 126;
      diagnostic(error.message);
      resolve(commandStatus);
      return;
    }
    child.stdout.on("data", output);
    child.stderr.on("data", output);
    child.once("error", (error) => {
      commandStatus = error.code === "ENOENT" ? 127 : 126;
      diagnostic(error.message);
      cleanup();
    });
    child.once("exit", (code, endedSignal) => {
      commandStatus ??= code ?? 128 + constants.signals[endedSignal];
      cleanup();
    });
    child.once("close", () => {
      childClosed = true;
      finish();
    });
    signal.addEventListener("abort", abort, { once: true });
    if (signal.aborted) abort();
  });
}
