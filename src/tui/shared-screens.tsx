import React from "react";

import { TerminalTextScreen } from "./terminal-ui.js";

export function TerminalBusyScreen({
  title = "PF2E Terminal",
  message,
}: {
  title?: string;
  message: string;
}): React.JSX.Element {
  return (
    <TerminalTextScreen
      title={title}
      body={[{ text: message, tone: "section" }]}
      footer={[{ text: "Working...", tone: "dim" }]}
    />
  );
}
