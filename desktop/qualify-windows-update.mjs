// Drive the real Windows confirmation owned by this disposable host. Closing it
// leaves the tab's live editor available to the next handoff attempt.
async function answerDiscard(ps, pid, env, answer) {
  await ps(`Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class AtlasDialog {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr parent, IntPtr after, string cls, string title);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint pid);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr window, uint message, UIntPtr command, IntPtr parameter);
}
'@
    $deadline = [DateTime]::UtcNow.AddSeconds(60)
    do {
      $dialog = [IntPtr]::Zero
      do {
        $dialog = [AtlasDialog]::FindWindowEx([IntPtr]::Zero, $dialog, '#32770', 'Install Atlas update')
        if ($dialog -eq [IntPtr]::Zero) { break }
        $ownerId = [uint32]0
        [void][AtlasDialog]::GetWindowThreadProcessId($dialog, [ref]$ownerId)
        if ($ownerId -eq ${pid}) {
          if (![AtlasDialog]::PostMessageW($dialog, 0x0111, [UIntPtr]${answer ? 6 : 7}, [IntPtr]::Zero)) { throw 'Could not answer update dialog' }
          return
        }
      } while ($true)
      Start-Sleep -Milliseconds 150
    } while ([DateTime]::UtcNow -lt $deadline)
    throw 'The installed host did not ask to discard changes'`, env, 70_000);
}

export async function qualifyUpdateGuards({ second, channel, expect, ps, pid, env, terminal, verifyOld, log }) {
  const install = channel.getByRole("button", { name: "Install Nightly and restart", exact: true });
  const review = async () => {
    await channel.getByRole("button", { name: "Check Nightly compatibility", exact: true }).click();
    await expect(install).toBeEnabled({ timeout: 30_000 });
  };
  const refused = async (message) => {
    await expect(channel.getByText(message)).toBeVisible({ timeout: 90_000 });
    verifyOld();
    await review();
  };

  const session = await terminal();
  try {
    await install.click();
    await refused(/Close the terminal session/);
    if (session.exitCode !== null) throw new Error("The update stopped an independent CLI");
  } finally {
    const exited = new Promise((done) => session.once("exit", done));
    if (session.kill()) await exited;
  }
  log("independent-terminal-preserved");

  // A listener that never replies must not count as a clean editor.
  await second.evaluate(() => {
    function hold(event) {
      if (event.detail.kind !== "check-exit") return;
      window.removeEventListener("atlas-windows-tab-command", hold, true);
      event.stopImmediatePropagation();
    }
    window.addEventListener("atlas-windows-tab-command", hold, true);
  });
  await install.click();
  await refused(/could not check every window/);
  log("missing-window-reply-refused");

  const newDraft = async (value) => {
    await second.getByRole("link", { name: "Parties", exact: true }).first().click();
    await second.getByRole("button", { name: "New party" }).click();
    const draft = second.getByRole("textbox", { name: "Party name" });
    await draft.fill(value);
    return draft;
  };
  const draft = await newDraft("Cancelled update retained this party");
  await install.click();
  // The embedded HTTPS server and CDP must keep running while the host
  // downloads the installer and prepares the confirmation.
  await answerDiscard(ps, pid, env, false);
  await refused(/Update cancelled/);
  await expect(draft).toHaveValue("Cancelled update retained this party");

  let waiting, release;
  const requested = new Promise((done) => { waiting = done; });
  const resume = new Promise((done) => { release = done; });
  await second.route("**/api/parties", async (route) => {
    if (route.request().method() === "POST") { waiting(); await resume; }
    await route.continue();
  });
  try {
    await second.getByRole("button", { name: "Create party" }).click();
    await requested;
    await install.click();
    await refused(/finish saving in all windows/);
  } finally {
    release();
  }
  await expect(second.getByRole("heading", { name: "Cancelled update retained this party" })).toBeVisible();
  await second.unroute("**/api/parties");
  log("pending-save-preserved");

  await newDraft("Discard this second-window draft");
  await install.click();
  await answerDiscard(ps, pid, env, true);
  log("dirty-second-window-discard-confirmed");
}
