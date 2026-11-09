import { fileURLToPath } from "node:url";

const schema = new URL("../../crates/atlas-web/openapi.json", import.meta.url);
const client = new URL(
  "../../web/atlas-ui/src/generated/http.ts",
  import.meta.url,
);
const transport = new URL(
  "../../web/atlas-ui/src/api/atlasTransport.ts",
  import.meta.url,
);

const input = process.env.ATLAS_OPENAPI_SCHEMA ?? fileURLToPath(schema);

export default {
  atlas: {
    input,
    output: {
      target: process.env.ATLAS_OPENAPI_CLIENT ?? fileURLToPath(client),
      client: "fetch",
      urlEncodeParameters: true,
      override: {
        mutator: { path: fileURLToPath(transport), name: "fetchGenerated" },
        fetch: { includeHttpResponseReturnType: false },
      },
    },
  },
};
