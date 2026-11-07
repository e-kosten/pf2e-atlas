import type { CatalogDiagnostic, Location } from '../contracts.js';
import { parseDiagnostics } from './compiler-types.js';
import fs from 'node:fs';
import path from 'node:path';
import ts from 'typescript';

export type Value = string | Map<string, Member> | Value[] | undefined;
export interface Member { value: Value; node: ts.Node }
export interface LoadedFile { file: ts.SourceFile; declarations: Map<string, ts.Expression>; imports: Map<string, { module: string; name: string }> }
class CatalogEvaluationError extends Error {
  constructor(readonly code: string, message: string, readonly node?: ts.Node) { super(message); }
}
/** Bounded authored catalog expressions shared by trait and reference extraction.
 * This reads a finite AST vocabulary; it never imports or executes upstream code.
 */
export function createCatalogReader(sourceRoot: string,
  options: { systemId?: 'pf2e' | 'sf2e'; labelKeyPolicy?: 'ranges' | 'ascii_words' } = {}) {
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
    if (ts.isConditionalExpression(node) && options.systemId && ts.isBinaryExpression(node.condition)
      && ts.isIdentifier(node.condition.left) && node.condition.left.text === 'SYSTEM_ID'
      && !context.declarations.has('SYSTEM_ID') && !context.imports.has('SYSTEM_ID')
      && ts.isStringLiteral(node.condition.right) && ['pf2e', 'sf2e'].includes(node.condition.right.text)
      && [ts.SyntaxKind.EqualsEqualsEqualsToken, ts.SyntaxKind.ExclamationEqualsEqualsToken].includes(node.condition.operatorToken.kind)) {
      const equal = options.systemId === node.condition.right.text;
      const selected = node.condition.operatorToken.kind === ts.SyntaxKind.EqualsEqualsEqualsToken ? equal : !equal;
      return evaluate(selected ? node.whenTrue : node.whenFalse, context, locals);
    }
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return node.text;
    if (ts.isIdentifier(node)) return variable(node.text, context, locals);
    if (ts.isNewExpression(node) && ts.isIdentifier(node.expression) && node.expression.text === 'Set'
      && !context.declarations.has('Set') && !context.imports.has('Set') && node.arguments?.length === 1) {
      const values = strings(evaluate(node.arguments[0], context, locals), node);
      if (new Set(values).size !== values.length) fail('duplicate-set-value', 'Catalog Set has repeated keys', node);
      return values;
    }
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
          const name = ts.isComputedPropertyName(member.name) ? evaluate(member.name.expression, context, locals) : member.name.text;
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
      if (ts.isPropertyAccessExpression(callee) && ts.isIdentifier(callee.expression)
        && callee.expression.text === 'Array' && callee.name.text === 'from'
        && !context.declarations.has('Array') && !context.imports.has('Array') && node.arguments.length === 1)
        return array(evaluate(node.arguments[0], context, locals), node);
      const remeda = ts.isPropertyAccessExpression(callee) && ts.isIdentifier(callee.expression) && context.imports.get(callee.expression.text)?.module === "remeda" && context.imports.get(callee.expression.text)?.name === "*";
      if (remeda) {
        const name = callee.name.text;
        if (name === "keys" && node.arguments.length === 1) return [...object(evaluate(node.arguments[0], context, locals), node).keys()];
        if (["pick", "omit"].includes(name) && node.arguments.length === 2) {
          const input = object(evaluate(node.arguments[0], context, locals), node);
          const keys = new Set(strings(evaluate(node.arguments[1], context, locals), node));
          return new Map([...input].filter(([key]) => name === "pick" ? keys.has(key) : !keys.has(key)));
        }
        if (name === 'mapValues' && node.arguments.length === 2) {
          const input = object(evaluate(node.arguments[0], context, locals), node);
          const callback = node.arguments[1];
          if (!ts.isArrowFunction(callback) || callback.parameters.length !== 2
            || !callback.parameters.every(parameter => ts.isIdentifier(parameter.name)) || ts.isBlock(callback.body))
            fail('unsupported-callback', 'mapValues requires a two-argument expression callback', callback);
          const [valueParameter, keyParameter] = callback.parameters.map(parameter => (parameter.name as ts.Identifier).text);
          return new Map([...input].map(([key, member]) => [key, {
            value: evaluate(callback.body, context, new Map(locals).set(valueParameter, member.value).set(keyParameter, key)), node,
          }]));
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
      // Only lowercase ASCII hyphen-separated keys with this exact option are
      // supported. This label-key transform never changes a semantic identifier.
      if (ts.isIdentifier(callee) && context.imports.get(callee.text)?.name === "sluggify" && context.imports.get(callee.text)?.module === "@util" && node.arguments.length === 2) {
        const text = evaluate(node.arguments[0], context, locals);
        const settings = object(evaluate(node.arguments[1], context, locals), node);
        const supported = options.labelKeyPolicy === 'ascii_words' ? /^[a-z0-9]+(?:-[a-z0-9]+)*$/ : /^range-(?:increment-)?[0-9]+$/;
        if (typeof text !== "string" || !supported.test(text) || settings.size !== 1 || settings.get("camel")?.value !== "bactrian") fail("unsupported-sluggify", "Unsupported ASCII label-key recipe or sluggify options", node);
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
  return { root, diagnostics, location, diagnose, fail, load, resolveImport, object, array, strings, variable, evaluate, attempt };
}
