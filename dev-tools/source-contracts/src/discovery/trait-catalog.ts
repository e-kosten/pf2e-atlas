import type { Catalog, FamilyBinding, TraitCatalog } from '../contracts.js';
import { createCatalogReader, type Member, type Value } from './catalog-values.js';
import fs from 'node:fs';
import path from 'node:path';
import ts from 'typescript';

function propertyText(name: ts.PropertyName): string | undefined {
  return ts.isComputedPropertyName(name) ? undefined : name.text;
}
/** Exact trait catalogs and bindings, using the shared bounded AST reader. */
export function extractTraitCatalog(sourceRoot: string, options: { configFile?: string; traitsFile?: string; localizationFile?: string } = {}): TraitCatalog {
  const { root, diagnostics, location, diagnose, fail, load, resolveImport, object, variable, evaluate, attempt } = createCatalogReader(sourceRoot);
  const configFile = options.configFile ?? 'src/scripts/config/index.ts';
  const traitsFile = options.traitsFile ?? 'src/scripts/config/traits.ts';
  const localization = attempt(() => {
    const authored = fs.readFileSync(path.join(root, options.localizationFile ?? "static/lang/en.json"), "utf8");
    try { return JSON.parse(authored) as unknown; } catch (error) {
      if (error instanceof SyntaxError) fail("invalid-localization-json", "Localization input is not valid JSON");
      throw error;
    }
  });
  const localized = (key: Value, node: ts.Node | undefined, kind: string): string | null => {
    if (typeof key !== "string") { diagnose("error", "invalid-localization-key", `${kind} key is not a string`, node); return null; }
    let value = localization;
    for (const part of key.split(".")) value = value && typeof value === "object" && Object.hasOwn(value, part) ? (value as Record<string, unknown>)[part] : undefined;
    if (typeof value !== "string") {
      diagnose("warning", "missing-localization", `No authored ${kind} text for ${key}`, node);
      return null;
    }
    return value;
  };
  const traits = attempt(() => load(traitsFile));
  const config = attempt(() => load(configFile));
  const descriptionsMap = traits ? attempt(() => object(variable("traitDescriptions", traits, new Map()), traits.file)) : null;
  const descriptions = [...(descriptionsMap ?? [])].sort(([a], [b]) => a.localeCompare(b, "en")).map(([identifier, member]) => ({ identifier, key: typeof member.value === "string" ? member.value : null, text: localized(member.value, member.node, "description"), source: location(member.node) }));
  const byDescription = new Map(descriptions.map((entry) => [entry.identifier, entry]));
  const catalogs: Catalog[] = [];
  const exportedNames = new Set<string>();
  for (const statement of traits?.file.statements ?? []) {
    if (ts.isExportDeclaration(statement) && !statement.isTypeOnly && statement.exportClause && ts.isNamedExports(statement.exportClause)) {
      for (const specifier of statement.exportClause.elements) if (!specifier.isTypeOnly) exportedNames.add(specifier.propertyName?.text ?? specifier.name.text);
    }
    if (ts.isVariableStatement(statement) && statement.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword)) {
      for (const declaration of statement.declarationList.declarations) if (ts.isIdentifier(declaration.name)) exportedNames.add(declaration.name.text);
    }
  }
  const addCatalog = (name: string, namespace: Catalog["namespace"], values: Map<string, Member> | null, member: ts.Node, exposedInConfig: boolean) => {
    const entries = [...(values ?? [])].sort(([a], [b]) => a.localeCompare(b, "en")).map(([identifier, entry]) => {
      // otherTags do not implicitly inherit a same-named trait description.
      const description = namespace === "otherTag" ? null : byDescription.get(identifier);
      return { identifier, labelKey: typeof entry.value === "string" ? entry.value : null, label: localized(entry.value, entry.node, "label"), descriptionKey: description?.key ?? null, description: description?.text ?? null, source: location(entry.node) };
    });
    catalogs.push({ name, namespace, complete: values !== null && [...values.values()].every((entry) => typeof entry.value === "string"), exposedInConfig, exportedFromTraits: exportedNames.has(name), source: location(member), entries, familyBindings: [] });
  };
  if (config) {
    // Select the configuration object structurally rather than evaluating unrelated
    // startup values, which include runtime helpers and document constructors.
    const candidates: ts.ObjectLiteralExpression[] = [];
    const visit = (node: ts.Node): void => {
      if (ts.isObjectLiteralExpression(node) && node.properties.some((member) => (ts.isShorthandPropertyAssignment(member) || ts.isPropertyAssignment(member)) && /^(?:traitsDescriptions|traitDescriptions)$/.test((propertyText(member.name) ?? "")))) candidates.push(node);
      ts.forEachChild(node, visit);
    };
    visit(config.file);
    if (candidates.length !== 1) diagnose("error", "config-object", `Expected one trait-bearing config object, found ${candidates.length}`, config.file);
    else for (const member of candidates[0].properties) {
      if (ts.isSpreadAssignment(member) || member.name && ts.isComputedPropertyName(member.name)) {
        diagnose("error", "unsupported-config-member", "Cannot prove trait catalog selection through a computed config key or spread", member);
        continue;
      }
      const name = member.name && (ts.isIdentifier(member.name) || ts.isStringLiteral(member.name)) ? member.name.text : null;
      if (!name || !(/Traits$/.test(name) || /^other.*Tags$/.test(name))) continue;
      if (!ts.isShorthandPropertyAssignment(member) && !ts.isPropertyAssignment(member)) {
        diagnose("error", "unsupported-config-member", `Catalog ${name} is not an authored property initializer`, member);
        continue;
      }
      const namespace = name === "rarityTraits" ? "rarity" : /^other.*Tags$/.test(name) ? "otherTag" : "trait";
      const values = attempt(() => object(ts.isShorthandPropertyAssignment(member) ? variable(name, config, new Map()) : evaluate(member.initializer, config), member));
      addCatalog(name, namespace, values, member, true);
    }
  }
  // Source-only helper/exported catalogs remain useful vocabulary evidence even
  // when they are not currently exposed on CONFIG.PF2E (e.g. backgroundTraits).
  for (const [name, initializer] of traits?.declarations ?? []) {
    if (!(/Traits$/.test(name) || /^other.*Tags$/.test(name)) || catalogs.some((catalog) => catalog.name === name)) continue;
    const value = attempt(() => variable(name, traits!, new Map()));
    // Authored key lists used by pick/omit are selectors, not label catalogs.
    if (Array.isArray(value) && value.every(entry => typeof entry === "string")) continue;
    const values = value === null ? null : attempt(() => object(value, initializer));
    addCatalog(name, /^other.*Tags$/.test(name) ? "otherTag" : "trait", values, initializer.parent, false);
  }
  const byCatalog = new Map(catalogs.map((entry) => [entry.name, entry]));
  // Some catalogs name their key types explicitly. Follow that authored
  // type import rather than guessing a family from the catalog's English name.
  for (const catalog of catalogs) {
    const initializer = traits?.declarations.get(catalog.name);
    const parent = initializer?.parent;
    const type = parent && ts.isVariableDeclaration(parent) ? parent.type : undefined;
    const keyType = type && ts.isTypeReferenceNode(type) && type.typeName.getText() === "Record" ? type.typeArguments?.[0] : null;
    const imported = keyType && ts.isTypeReferenceNode(keyType) ? traits?.imports.get(keyType.typeName.getText()) : null;
    if (imported) attempt(() => {
      const context = resolveImport(imported.module, traits!);
      const declaration = context.file.statements.find((statement) => ts.isTypeAliasDeclaration(statement) && statement.name.text === imported.name);
      const relative = path.relative(root, context.file.fileName).split(path.sep).join("/");
      const parts = relative.split("/");
      if (!declaration) return fail("missing-key-type", `No declaration for catalog key type ${imported.name}`, keyType ?? undefined);
      if (parts[0] === "src" && parts[1] === "module" && (parts[2] === "item" || parts[2] === "actor")) catalog.familyBindings.push({ scope: parts[2], family: parts[3], basis: "catalog-key-type", declaration: imported.name, source: location(declaration) });
    });
  }
  const scanBindings = (directory: string, scope: FamilyBinding["scope"]): void => {
    const absolute = path.join(root, directory);
    if (!fs.existsSync(absolute)) return;
    for (const entry of fs.readdirSync(absolute, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name, "en"))) {
      const relative = `${directory}/${entry.name}`;
      if (entry.isDirectory()) scanBindings(relative, scope);
      else if (entry.isFile() && entry.name.endsWith(".ts")) {
        const context = attempt(() => load(relative));
        if (!context) continue;
        const family = relative.split("/")[3];
        const add = (catalog: string, basis: FamilyBinding["basis"], node: ts.Node, declaration: string | null = null) => {
          if (!byCatalog.has(catalog)) {
            if (basis === "runtime-validTraits") diagnose("error", "unresolved-family-catalog", `validTraits references an unextracted catalog ${catalog}`, node);
            return;
          }
          byCatalog.get(catalog)!.familyBindings.push({ scope, family, basis, declaration, source: location(node) });
        };
        const visit = (node: ts.Node): void => {
          if (ts.isGetAccessorDeclaration(node) && node.name.getText() === "validTraits" && node.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.StaticKeyword)) {
            const returns = node.body?.statements.filter(ts.isReturnStatement) ?? [];
            const expression = returns.length === 1 ? returns[0].expression : null;
            if (expression && ts.isPropertyAccessExpression(expression) && expression.expression.getText() === "CONFIG.PF2E") add(expression.name.text, "runtime-validTraits", node);
            // A literal empty getter is not a catalog binding; complex getters
            // remain explicitly outside the evidence extracted here.
            else if (!expression || !ts.isObjectLiteralExpression(expression) || expression.properties.length) diagnose("error", "unsupported-family-binding", "validTraits is not an empty literal or direct CONFIG.PF2E catalog reference", node);
          }
          if (ts.isTypeAliasDeclaration(node)) {
            const inspect = (type: ts.Node): void => {
              if (ts.isTypeOperatorNode(type) && type.operator === ts.SyntaxKind.KeyOfKeyword && ts.isTypeQueryNode(type.type)) {
                const parts = type.type.exprName.getText().split(".");
                if (parts.length === 3 && parts[0] === "CONFIG" && parts[1] === "PF2E") add(parts[2], "declared-keyof-vocabulary", type, node.name.text);
              }
              ts.forEachChild(type, inspect);
            };
            inspect(node.type);
          }
          ts.forEachChild(node, visit);
        };
        visit(context.file);
      }
    }
  };
  scanBindings("src/module/item", "item");
  scanBindings("src/module/actor", "actor");
  catalogs.sort((a, b) => a.name.localeCompare(b.name, "en"));
  diagnostics.sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b), "en"));
  return { complete: !diagnostics.some((diagnostic) => diagnostic.severity === "error"), catalogs, descriptions, diagnostics };
}
