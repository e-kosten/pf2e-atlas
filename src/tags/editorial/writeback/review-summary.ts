import { writeFile } from "node:fs/promises";
import path from "node:path";

import { reviewSessionDirectory } from "../sessions/session-store.js";

export async function writeDerivedTagReviewSummary(
  rootPath: string,
  sessionId: string,
  summary: string,
): Promise<void> {
  const directory = reviewSessionDirectory(rootPath, sessionId);
  await writeFile(path.join(directory, "summary.md"), `${summary}\n`, "utf8");
}
