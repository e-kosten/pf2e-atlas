import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";

const web = new URL("../web/atlas-ui/", import.meta.url);
const requireWeb = createRequire(new URL("package.json", web));
const ts = requireWeb("typescript");
const { createElement } = requireWeb("react");
const { renderToStaticMarkup } = requireWeb("react-dom/server");

function loadBrandModule(path) {
  const source = readFileSync(new URL(path, web), "utf8");
  const { outputText } = ts.transpileModule(source, {
    compilerOptions: {
      jsx: ts.JsxEmit.ReactJSX,
      module: ts.ModuleKind.CommonJS,
    },
  });
  const exports = {};
  new Function("require", "exports", outputText)(requireWeb, exports);
  return exports;
}

const { AtlasMark } = loadBrandModule("src/shared/branding/AtlasMark.tsx");
const { atlasPalettes } = loadBrandModule(
  "src/shared/theme/themeDefinitions.ts",
);
const target = new URL(".generated/bootstrap/", import.meta.url);
const bootstrap = new URL("bootstrap/", import.meta.url);
mkdirSync(target, { recursive: true });

function brandColors(scheme) {
  const { text, mark } = atlasPalettes[scheme].brand;
  const { dieFaces, ...colors } = mark;
  const variables = Object.entries(colors).map(
    ([key, value]) =>
      `--brand-mark-${key.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}: ${value};`,
  );
  dieFaces.forEach((value, index) =>
    variables.push(`--brand-mark-die-face-${index}: ${value};`),
  );
  return `:root { --brand-text: ${text}; ${variables.join(" ")} }`;
}

const font = "@fontsource-variable/lora/files/lora-latin-wght-normal.woff2";
const styles = readFileSync(
  new URL("src/styles/ui/atlas-mark.css", web),
  "utf8",
).replace(font, "lora.woff2");
writeFileSync(
  new URL("branding.css", target),
  [
    brandColors("light"),
    `@media (prefers-color-scheme: dark) { ${brandColors("dark")} }`,
    styles,
  ].join("\n"),
);
copyFileSync(requireWeb.resolve(font), new URL("lora.woff2", target));
copyFileSync(
  new URL("node_modules/@fontsource-variable/lora/LICENSE", web),
  new URL("lora-license.txt", target),
);
for (const file of ["setup.css", "setup.js"]) {
  copyFileSync(new URL(file, bootstrap), new URL(file, target));
}
const html = readFileSync(new URL("index.html", bootstrap), "utf8").replace(
  "<!-- atlas-mark -->",
  renderToStaticMarkup(createElement(AtlasMark)),
);
writeFileSync(new URL("index.html", target), html);
