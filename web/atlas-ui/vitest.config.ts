import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";
import { atlasApiContract } from "./apiContract.ts";

// These cases exercise values and requests without storage, rendering or hooks.
// New tests use the DOM project until their runtime dependencies are reviewed.
const nodeTests = [
  "src/app/readingUrl.test.ts",
  "src/app/urlState.test.ts",
  "src/features/search/pageMetrics.test.ts",
  "src/shared/feedback/errorMessages.test.ts",
  "src/shared/filters/builderEdits.test.ts",
  "src/shared/filters/filterControls.test.ts",
  "src/shared/filters/filterUrl.test.ts",
  "src/shared/filters/operatorTransitions.test.ts",
  "src/shared/filters/searchState.test.ts",
  "src/shared/records/readingUrlNavigation.test.ts",
  "src/shared/records/recordDetailQuery.test.ts",
  "src/shared/records/statusReasonLabel.test.ts",
  "src/shared/search/searchIdentity.test.ts",
  "src/shared/search/similarityState.test.ts",
];

export default defineConfig({
  // npm test generates routes first; test transforms need no route watcher/splitter.
  plugins: [atlasApiContract(), react()],
  test: {
    // Match CI's bounded worker count for the Ant-heavy DOM suites.
    maxWorkers: 1,
    globals: true,
    projects: [
      {
        extends: true,
        test: { name: "node", environment: "node", include: nodeTests },
      },
      {
        extends: true,
        test: {
          name: "dom",
          environment: "jsdom",
          include: ["src/**/*.test.{ts,tsx}"],
          exclude: nodeTests,
          server: { deps: { inline: [/@react-querybuilder/, /antd/] } },
          setupFiles: ["./src/test/setup.ts"],
        },
      },
    ],
  },
});
