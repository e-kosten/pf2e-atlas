import path from 'node:path';
import ts from 'typescript';

// These select upstream registries, not a hand-maintained list of families or rules.
export const PORTFOLIO_FILE = '__atlas_source_portfolio__.ts';

export function discoverSourcePortfolio(root) {
  const diagnostics = [];
  const selections = [];
  const families = [];
  const imports = [];
  const aliases = [];
  const problem = (message) => diagnostics.push({ code: 'source-portfolio', message });
  const unique = (matches, context) => {
    if (matches.length > 1) problem(`Duplicate ${context}`);
    return matches[0];
  };
  const read = (file) => {
    const text = ts.sys.readFile(path.join(root, file));
    if (text === undefined) { problem(`Missing portfolio input ${file}`); return null; }
    const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
    for (const diagnostic of source.parseDiagnostics) problem(`Cannot parse ${file}: ${ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n')}`);
    return source;
  };
  const name = (node) => node && (ts.isIdentifier(node) || ts.isStringLiteral(node)) ? node.text : null;
  const accessPath = (node) => {
    if (ts.isIdentifier(node)) return node.text;
    if (ts.isPropertyAccessExpression(node)) return `${accessPath(node.expression)}.${node.name.text}`;
    if (ts.isElementAccessExpression(node)) return `${accessPath(node.expression)}.${name(node.argumentExpression) ?? '*'}`;
    return null;
  };
  const rejectMutations = (source, registries) => {
    const touches = (node) => {
      const target = accessPath(node);
      return target && registries.some((registry) => target === registry || target.startsWith(`${registry}.`)
        || registry.startsWith(`${target}.`));
    };
    function visit(node) {
      const write = ts.isBinaryExpression(node) && node.operatorToken.kind >= ts.SyntaxKind.FirstAssignment
        && node.operatorToken.kind <= ts.SyntaxKind.LastAssignment ? node.left
        : ts.isDeleteExpression(node) ? node.expression
          : (ts.isPrefixUnaryExpression(node) || ts.isPostfixUnaryExpression(node))
            && [ts.SyntaxKind.PlusPlusToken, ts.SyntaxKind.MinusMinusToken].includes(node.operator) ? node.operand : null;
      if (write && touches(write)) problem(`Registry mutation is outside literal discovery: ${node.getText()}`);
      if (ts.isCallExpression(node) && node.arguments.some(touches)) {
        problem(`Registry passed to a call is outside literal discovery: ${node.getText()}`);
      }
      ts.forEachChild(node, visit);
    }
    if (source) visit(source);
  };
  const entries = (object, context) => {
    if (!object || !ts.isObjectLiteralExpression(object)) {
      problem(`Expected a literal registry at ${context}`); return [];
    }
    const keys = new Set();
    return object.properties.flatMap((property) => {
      const key = name(property.name);
      if (!ts.isPropertyAssignment(property) || key === null) {
        problem(`Unsupported registry entry at ${context}: ${property.getText()}`); return [];
      }
      if (keys.has(key)) problem(`Duplicate registry key ${key} at ${context}`);
      keys.add(key);
      return [[key, property.initializer]];
    });
  };
  const property = (object, key, context) => {
    if (!object || !ts.isObjectLiteralExpression(object)) {
      problem(`Expected a literal object at ${context}`); return undefined;
    }
    if (object.properties.some((entry) => ts.isSpreadAssignment(entry) || (entry.name && ts.isComputedPropertyName(entry.name)))) {
      problem(`Dynamic properties at ${context} prevent reliable registry selection`);
    }
    const matches = object.properties.filter((entry) => ts.isPropertyAssignment(entry) && name(entry.name) === key);
    return unique(matches, `property ${key} at ${context}`)?.initializer;
  };
  const manifestText = ts.sys.readFile(path.join(root, 'static/system.json'));
  let kinds = [];
  try {
    const manifest = JSON.parse(manifestText);
    if (!Array.isArray(manifest.packs) || manifest.packs.some((pack) => typeof pack.type !== 'string')) {
      throw new Error('packs must have string document types');
    }
    kinds = [...new Set(manifest.packs.map((pack) => pack.type))].sort();
    if (!kinds.length) throw new Error('packs must not be empty');
  } catch (error) { problem(`Invalid static/system.json: ${error.message}`); }
  for (const kind of kinds) {
    const overrides = {
      Actor: ['src/module/actor/data/index.ts', 'ActorSourcePF2e'],
      Item: ['src/module/item/base/data/index.ts', 'ItemSourcePF2e'],
    };
    const [file, exportName] = overrides[kind] ?? ['types/foundry/common/documents/module.d.ts', `${kind}Source`];
    selections.push({ file, name: exportName, documentKind: kind });
  }

  const config = read('src/scripts/config/index.ts');
  rejectMutations(config, ['PF2ECONFIG.Actor.documentClasses', 'PF2ECONFIG.Item.documentClasses']);
  const declaration = unique(config?.statements.flatMap((statement) => ts.isVariableStatement(statement)
    ? [...statement.declarationList.declarations] : []).filter((entry) => name(entry.name) === 'PF2ECONFIG') ?? [],
  'PF2ECONFIG declaration');
  for (const kind of ['Actor', 'Item']) {
    const object = property(declaration?.initializer, kind, 'PF2ECONFIG');
    const registry = property(object, 'documentClasses', `PF2ECONFIG.${kind}`);
    const keys = entries(registry, `PF2ECONFIG.${kind}.documentClasses`).map(([key]) => key).sort();
    if (!keys.length) problem(`No ${kind} families registered`);
    if (!kinds.includes(kind)) problem(`${kind} registry exists but no ${kind} packs select its source root`);
    families.push({ documentKind: kind, registered: keys });
  }

  const rules = read('src/module/rules/index.ts');
  rejectMutations(rules, ['RuleElements.builtin']);
  const ruleClass = unique(rules?.statements.filter((statement) => ts.isClassDeclaration(statement)
    && name(statement.name) === 'RuleElements') ?? [], 'RuleElements declaration');
  const builtin = unique(ruleClass?.members.filter((member) => name(member.name) === 'builtin') ?? [],
    'RuleElements.builtin member');
  if (builtin && (!ts.isPropertyDeclaration(builtin) || !(ts.getCombinedModifierFlags(builtin) & ts.ModifierFlags.Static))) {
    problem('RuleElements.builtin must be a static property');
  }
  const registeredRules = entries(builtin?.initializer, 'RuleElements.builtin');
  if (!registeredRules.length) problem('No built-in rule schemas registered');
  const bindings = new Map();
  for (const statement of rules?.statements ?? []) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) continue;
    const named = statement.importClause?.namedBindings;
    if (!named || !ts.isNamedImports(named)) continue;
    for (const binding of named.elements) bindings.set(binding.name.text, {
      imported: binding.propertyName?.text ?? binding.name.text,
      file: path.posix.normalize(path.posix.join('src/module/rules', statement.moduleSpecifier.text)),
    });
  }
  for (const [index, [key, constructor]] of registeredRules.sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).entries()) {
    const binding = ts.isIdentifier(constructor) && bindings.get(constructor.text);
    if (!binding) { problem(`Cannot resolve registered rule constructor ${key}: ${constructor.getText()}`); continue; }
    // Only declarations are compiled. No upstream module is executed and no file is written to the source tree.
    imports.push(`import { ${binding.imported} as Rule${index} } from ${JSON.stringify(`./${binding.file}`)};`);
    const alias = `RuleSource_${Buffer.from(key).toString('hex')}`;
    aliases.push(`export type ${alias} = SourceFromSchema<ReturnType<typeof Rule${index}.defineSchema>>;`);
    selections.push({ file: PORTFOLIO_FILE, name: alias, ruleKey: key,
      schemaConstructor: { file: binding.file, name: binding.imported } });
  }
  return { selections, families, diagnostics, documentKinds: kinds,
    ruleKeys: registeredRules.map(([key]) => key).sort(), virtualSource: [...imports, ...aliases].join('\n') };
}
