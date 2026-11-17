import type { Locator } from "@playwright/test";

export function markdownSource(textbox: Locator) {
  return textbox
    .locator(".cm-line")
    .allTextContents()
    .then((lines) => lines.join("\n"));
}

export async function pressMarkdownShortcut(
  textbox: Locator,
  key: string,
  nonAppleKey = key,
) {
  // CodeMirror uses the browser platform, which can differ from Playwright's host OS.
  const appleKeyboard = await textbox
    .page()
    .evaluate(
      () =>
        /Mac/.test(navigator.platform) ||
        (/Apple Computer/.test(navigator.vendor) &&
          (/Mobile\/\w+/.test(navigator.userAgent) || navigator.maxTouchPoints > 2)),
    );
  await textbox.press(appleKeyboard ? `Meta+${key}` : `Control+${nonAppleKey}`);
}

export async function replaceMarkdown(textbox: Locator, text: string) {
  await pressMarkdownShortcut(textbox, "a");
  if (text) await textbox.page().keyboard.insertText(text);
  else await textbox.press("Backspace");
}
