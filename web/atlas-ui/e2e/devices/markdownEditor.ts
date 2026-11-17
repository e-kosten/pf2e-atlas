import type { Locator } from "@playwright/test";

export function markdownSource(textbox: Locator) {
  return textbox
    .locator(".cm-line")
    .allTextContents()
    .then((lines) => lines.join("\n"));
}

export async function replaceMarkdown(textbox: Locator, text: string) {
  await textbox.press("ControlOrMeta+a");
  if (text) await textbox.page().keyboard.insertText(text);
  else await textbox.press("Backspace");
}
