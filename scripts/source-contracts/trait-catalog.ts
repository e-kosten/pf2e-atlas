import type { Catalog, CatalogDiagnostic, FamilyBinding, Location, TraitCatalog } from './contracts.js';
import { parseDiagnostics } from './compiler-types.js';

type Value = string | Map<string, Member> | Value[] | undefined;
interface Member { value: Value; node: ts.Node }
interface LoadedFile { file: ts.SourceFile; declarations: Map<string, ts.Expression>; imports: Map<string, { module: string; name: string }> }
class CatalogEvaluationError extends Error {
  constructor(readonly code: string, message: string, readonly node?: ts.Node) { super(message); }
}
function propertyText(name: ts.PropertyName): string | undefined {
  return ts.isComputedPropertyName(name) ? undefined : name.text;
}
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";

/** Extract authored metadata without starting Foundry or evaluating JavaScript.
 * Catalog membership is vocabulary evidence, not a universal runtime validity rule.
 * Missing metadata remains null. Unsupported reachable syntax makes complete false.
 */
export function extractTraitCatalog(sourceRoot: string, options: { configFile?: string; traitsFile?: string; localizationFile?: string } = {}): TraitCatalog {
  const root = path.resolve(sourceRoot);
  const diagnostics: CatalogDiagnostic[] = [];
  const files = new Map<string, LoadedFile>();
  const cache = new Map<string, Value>();
  const resolving = new Set<string>();
  const location = (node: ts.Node): Location => {
    const file = node.getSourceFile();
    const point = file.getLineAndCharacterOfPosition(node.getStart(file));
    return { file: path.relative(root, file.fileName).split(path.sep).join("/"), line: point.line + 1, column: point.character + 1 };
  };
  const diagnose = (severity: CatalogDiagnostic["severity"], code: string, message: string, node?: ts.Node) => {
    diagnostics.push({ severity, code, message, source: node ? location(node) : null });
  };
  function fail(code: string, message: string, node?: ts.Node): never {
    throw new CatalogEvaluationError(code, message, node);
  };
  const load = (relative: string): LoadedFile => {
    const filename = path.resolve(root, relative);
    if (filename !== root && !filename.startsWith(`${root}${path.sep}`)) fail("outside-source", "Import escapes the source checkout");
    if (files.has(filename)) return files.get(filename)!;
    const file = ts.createSourceFile(filename, fs.readFileSync(filename, "utf8"), ts.ScriptTarget.Latest, true);
    for (const diagnostic of parseDiagnostics(file)) {
      diagnose("error", "syntax", ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"), file);
    }
    const declarations = new Map<string, ts.Expression>();
    const imports = new Map<string, { module: string; name: string }>();
    for (const statement of file.statements) {
      if (ts.isVariableStatement(statement)) {
        for (const declaration of statement.declarationList.declarations) {
          if (ts.isIdentifier(declaration.name) && declaration.initializer) declarations.set(declaration.name.text, declaration.initializer);
        }
      }
      if (ts.isImportDeclaration(statement) && ts.isStringLiteral(statement.moduleSpecifier)) {
        const bindings = statement.importClause?.namedBindings;
        if (bindings && ts.isNamedImports(bindings)) {
          for (const binding of bindings.elements) imports.set(binding.name.text, { module: statement.moduleSpecifier.text, name: binding.propertyName?.text ?? binding.name.text });
        } else if (bindings && ts.isNamespaceImport(bindings)) {
          imports.set(bindings.name.text, { module: statement.moduleSpecifier.text, name: "*" });
        }
      }
    }
    const loaded = { file, declarations, imports };
    files.set(filename, loaded);
    return loaded;
  };
  const configFile = options.configFile ?? "src/scripts/config/index.ts";
  const traitsFile = options.traitsFile ?? "src/scripts/config/traits.ts";
  let compilerOptions: ts.CompilerOptions | undefined;
  const resolveImport = (moduleName: string, from: LoadedFile): LoadedFile => {
    if (!compilerOptions) {
      const configPath = path.join(root, "tsconfig.json");
      const read = ts.readConfigFile(configPath, ts.sys.readFile);
      if (read.error) fail("tsconfig", ts.flattenDiagnosticMessageText(read.error.messageText, "\n"));
      const parsed = ts.parseJsonConfigFileContent(read.config, ts.sys, root);
      if (parsed.errors.length) fail("tsconfig", parsed.errors.map((error) => ts.flattenDiagnosticMessageText(error.messageText, "\n")).join("\n"));
      compilerOptions = parsed.options;
    }
    const resolved = ts.resolveModuleName(moduleName, from.file.fileName, compilerOptions, ts.sys).resolvedModule;
    if (!resolved) fail("unresolved-import", `Cannot resolve ${moduleName}`, from.file);
    return load(path.relative(root, resolved.resolvedFileName));
  };
  const object = (value: Value, node?: ts.Node): Map<string, Member> => {
    if (!(value instanceof Map)) fail("expected-object", "Expected a catalog object", node);
    return value;
  };
  const array = (value: Value, node?: ts.Node): Value[] => {
    if (!Array.isArray(value)) fail("expected-array", "Expected a literal array", node);
    return value;
  };
  const strings = (value: Value, node?: ts.Node): string[] => {
    const values = array(value, node);
    if (values.some((entry) => typeof entry !== "string")) fail("expected-strings", "Expected string keys", node);
    return values as string[];
  };
  const variable = (name: string, context: LoadedFile, locals: Map<string, Value>): Value => {
    if (locals.has(name)) return locals.get(name);
    const key = `${context.file.fileName}:${name}`;
    if (cache.has(key)) return cache.get(key);
    if (resolving.has(key)) fail("cyclic-initializer", `Cyclic initializer ${name}`, context.file);
    resolving.add(key);
    try {
      let value;
      if (context.declarations.has(name)) value = evaluate(context.declarations.get(name)!, context, locals);
      else if (context.imports.has(name)) {
        const imported = context.imports.get(name)!;
        value = variable(imported.name, resolveImport(imported.module, context), new Map());
      } else fail("unresolved-initializer", `No authored initializer for ${name}`, context.file);
      cache.set(key, value);
      return value;
    } finally {
      resolving.delete(key);
    }
  };
  const evaluate = (node: ts.Node, context: LoadedFile, locals = new Map<string, Value>()): Value => {
    if (ts.isParenthesizedExpression(node) || ts.isAsExpression(node) || ts.isSatisfiesExpression(node) || ts.isTypeAssertionExpression(node)) return evaluate(node.expression, context, locals);
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return node.text;
    if (ts.isIdentifier(node)) return variable(node.text, context, locals);
    if (ts.isArrayLiteralExpression(node)) {
      return node.elements.flatMap((entry) => ts.isSpreadElement(entry) ? array(evaluate(entry.expression, context, locals), entry) : [evaluate(entry, context, locals)]);
    }
    if (ts.isObjectLiteralExpression(node)) {
      const result = new Map<string, Member>();
      for (const member of node.properties) {
        if (ts.isSpreadAssignment(member)) {
          for (const [key, value] of object(evaluate(member.expression, context, locals), member)) result.set(key, value);
        } else if (ts.isShorthandPropertyAssignment(member)) {
          result.set(member.name.text, { value: variable(member.name.text, context, locals), node: member });
        } else if (ts.isPropertyAssignment(member)) {
          const name = ts.isComputedPropertyName(member.name) ? evaluate(member.name.expression, context, locals) : ts.isComputedPropertyName(member.name) ? undefined : member.name.text;
          if (typeof name !== "string") fail("computed-key", "Computed key is not a string", member.name);
          result.set(name, { value: evaluate(member.initializer, context, locals), node: member });
        } else fail("unsupported-member", `Unsupported catalog member: ${ts.SyntaxKind[member.kind]}`, member);
      }
      return result;
    }
    if (ts.isTemplateExpression(node)) {
      return node.head.text + node.templateSpans.map((span) => {
        const value = evaluate(span.expression, context, locals);
        if (typeof value !== "string") fail("template-value", "Template substitution must resolve to a string", span.expression);
        return value + span.literal.text;
      }).join("");
    }
    if (ts.isCallExpression(node)) {
      const callee = node.expression;
      const remeda = ts.isPropertyAccessExpression(callee) && ts.isIdentifier(callee.expression) && context.imports.get(callee.expression.text)?.module === "remeda" && context.imports.get(callee.expression.text)?.name === "*";
      if (remeda) {
        const name = callee.name.text;
        if (name === "keys" && node.arguments.length === 1) return [...object(evaluate(node.arguments[0], context, locals), node).keys()];
        if (["pick", "omit"].includes(name) && node.arguments.length === 2) {
          const input = object(evaluate(node.arguments[0], context, locals), node);
          const keys = new Set(strings(evaluate(node.arguments[1], context, locals), node));
          return new Map([...input].filter(([key]) => name === "pick" ? keys.has(key) : !keys.has(key)));
        }
        if (name === "mapToObj" && node.arguments.length === 2) {
          const input = array(evaluate(node.arguments[0], context, locals), node);
          const callback = node.arguments[1];
          if (!ts.isArrowFunction(callback) || callback.parameters.length !== 1 || !ts.isIdentifier(callback.parameters[0].name) || ts.isBlock(callback.body)) fail("unsupported-callback", "mapToObj requires a one-argument expression callback", callback);
          const output = new Map<string, Member>();
          for (const value of input) {
            const environment = new Map(locals).set(callback.parameters[0].name.text, value);
            const pair = array(evaluate(callback.body, context, environment), callback.body);
            if (pair.length !== 2 || typeof pair[0] !== "string") fail("invalid-map-pair", "mapToObj must yield [string, value]", callback);
            output.set(pair[0], { value: pair[1], node });
          }
          return output;
        }
      }
      // Only the existing ASCII range-key recipe is supported; broader sluggify
      // behavior is not approximated. All other inputs/options are diagnosed.
      if (ts.isIdentifier(callee) && context.imports.get(callee.text)?.name === "sluggify" && context.imports.get(callee.text)?.module === "@util" && node.arguments.length === 2) {
        const text = evaluate(node.arguments[0], context, locals);
        const settings = object(evaluate(node.arguments[1], context, locals), node);
        if (typeof text !== "string" || !/^range-(?:increment-)?[0-9]+$/.test(text) || settings.size !== 1 || settings.get("camel")?.value !== "bactrian") fail("unsupported-sluggify", "Only range-number ASCII keys with camel: bactrian are supported", node);
        return text.split("-").map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join("");
      }
      fail("unsupported-call", `Unsupported computed catalog expression: ${callee.getText()}`, node);
    }
    fail("unsupported-expression", `Unsupported computed catalog expression: ${ts.SyntaxKind[node.kind]}`, node);
  };
  const attempt = <T>(operation: () => T): T | null => {
    try { return operation(); } catch (error) {
      if (error instanceof CatalogEvaluationError) {
        const { code, message, node } = error;
        diagnose("error", code, message, node);
      } else if (error instanceof Error && "code" in error && error.code === "ENOENT") diagnose("error", "missing-file", `Missing source file: ${path.relative(root, String("path" in error ? error.path : ""))}`);
      else throw error;
      return null;
    }
  };
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
    const values = attempt(() => object(variable(name, traits!, new Map()), initializer));
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
      if (!declaration) fail("missing-key-type", `No declaration for catalog key type ${imported.name}`, keyType ?? undefined);
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
