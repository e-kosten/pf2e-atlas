import { readFile } from 'node:fs/promises';
import path from 'node:path';
import ts from 'typescript';

export interface ActionVariantDefinition { slug: string; name: string | null; cost: string | null }
export interface ActionDefinition extends ActionVariantDefinition { file: string; line: number; variants: ActionVariantDefinition[] }

/** Read only the declared PF2e registry and literal constructor metadata. Never execute upstream code. */
export async function actionDefinitions(source: string): Promise<ActionDefinition[]> {
  const registryPath = 'src/module/system/action-macros/index.ts';
  const parse = async (file: string) => ts.createSourceFile(file, await readFile(path.join(source, file), 'utf8'), ts.ScriptTarget.Latest, true);
  const registry = await parse(registryPath);
  const variables = (file: ts.SourceFile) => {
    const result = new Map<string, ts.Expression>();
    for (const statement of file.statements) if (ts.isVariableStatement(statement))
      for (const declaration of statement.declarationList.declarations)
        if (ts.isIdentifier(declaration.name) && declaration.initializer) result.set(declaration.name.text, declaration.initializer);
    return result;
  };
  const imports = new Map<string, { file: string; name?: string }>();
  for (const statement of registry.statements) if (ts.isImportDeclaration(statement) && ts.isStringLiteral(statement.moduleSpecifier)) {
    const binding = statement.importClause?.namedBindings;
    const file = path.posix.join(path.posix.dirname(registryPath), statement.moduleSpecifier.text);
    if (binding && ts.isNamespaceImport(binding)) imports.set(binding.name.text, { file });
    else if (binding && ts.isNamedImports(binding)) for (const entry of binding.elements)
      imports.set(entry.name.text, { file, name: entry.propertyName?.text ?? entry.name.text });
  }
  const declared = variables(registry).get('SystemActions');
  if (!declared || !ts.isArrayLiteralExpression(declared)) throw new Error('Expected literal SystemActions registry');
  const actions = new Map<string, ActionDefinition>();
  for (const entry of declared.elements) {
    const local = ts.isIdentifier(entry) ? entry.text : ts.isPropertyAccessExpression(entry) && ts.isIdentifier(entry.expression) ? entry.expression.text : '';
    let imported = imports.get(local);
    if (!imported) throw new Error(`Unresolved registry entry ${entry.getText(registry)}`);
    let file = await parse(imported.file);
    let name = ts.isPropertyAccessExpression(entry) ? entry.name.text : imported.name;
    const visited = new Set<string>();
    while (!variables(file).has(name ?? '')) {
      const reexport = file.statements.flatMap(statement => ts.isExportDeclaration(statement) && statement.moduleSpecifier
        && ts.isStringLiteral(statement.moduleSpecifier) && statement.exportClause && ts.isNamedExports(statement.exportClause)
        ? statement.exportClause.elements.filter(element => element.name.text === name).map(element => ({
          file: path.posix.join(path.posix.dirname(imported!.file), (statement.moduleSpecifier as ts.StringLiteral).text),
          name: element.propertyName?.text ?? element.name.text,
        })) : [])[0];
      if (!reexport) break;
      if (visited.has(reexport.file)) throw new Error('Cyclic action reexport');
      visited.add(reexport.file);
      imported = reexport; name = reexport.name; file = await parse(imported.file);
    }
    const values = variables(file);
    let initializer = name && values.get(name);
    // Namespace exports may rename their local action variable.
    if (!initializer) for (const statement of file.statements) if (ts.isExportDeclaration(statement)
      && statement.exportClause && ts.isNamedExports(statement.exportClause)) {
      const exported = statement.exportClause.elements.find(element => element.name.text === name);
      if (exported) initializer = values.get(exported.propertyName?.text ?? exported.name.text);
    }
    if (!initializer || !ts.isNewExpression(initializer)) throw new Error(`Expected action constructor: ${imported.file}:${name}`);
    let metadata = initializer.arguments?.[0];
    if (!metadata) {
      const className = initializer.expression.getText(file);
      const declaration = file.statements.find((statement): statement is ts.ClassDeclaration => ts.isClassDeclaration(statement) && statement.name?.text === className);
      const calls: ts.CallExpression[] = [];
      const visit = (node: ts.Node) => { if (ts.isCallExpression(node) && node.expression.kind === ts.SyntaxKind.SuperKeyword) calls.push(node); ts.forEachChild(node, visit); };
      if (declaration) ts.forEachChild(declaration, visit);
      if (calls.length !== 1) throw new Error(`Expected one literal super constructor: ${imported.file}`);
      metadata = calls[0].arguments[0];
    }
    const literal = (node: ts.Expression): string => {
      if (ts.isStringLiteralLike(node) || ts.isNumericLiteral(node)) return node.text;
      if (ts.isIdentifier(node) && values.has(node.text)) return literal(values.get(node.text)!);
      if (ts.isTemplateExpression(node)) return node.head.text + node.templateSpans.map(span => literal(span.expression) + span.literal.text).join('');
      throw new Error(`Nonliteral action metadata: ${imported.file}:${node.getText(file)}`);
    };
    const fields = (node: ts.Expression | undefined) => {
      if (!node || !ts.isObjectLiteralExpression(node)) throw new Error(`Expected literal action data: ${imported.file}`);
      if (node.properties.some(property => !ts.isPropertyAssignment(property)))
        throw new Error(`Nonliteral action data properties: ${imported.file}`);
      return new Map(node.properties.flatMap(property => ts.isPropertyAssignment(property) ? [[property.name.getText(file).replace(/^['"]|['"]$/g, ''), property.initializer] as const] : []));
    };
    const definition = (node: ts.Expression): ActionVariantDefinition => {
      const data = fields(node);
      const get = (key: string) => data.has(key) ? literal(data.get(key)!) : null;
      const cost = get('cost');
      if (cost !== null && !['0', '1', '2', '3', 'free', 'reaction'].includes(cost)) throw new Error(`Invalid action cost ${cost}`);
      return { slug: get('slug') ?? '', name: get('name'), cost };
    };
    const action = definition(metadata!);
    if (!action.slug || !action.name) throw new Error(`Missing explicit action identity: ${imported.file}`);
    const variantNode = fields(metadata).get('variants');
    const variants = variantNode ? (ts.isArrayLiteralExpression(variantNode) ? [...variantNode.elements] : [variantNode]).map(node => definition(node)) : [];
    for (const variant of variants) if (!variant.slug) throw new Error(`Missing explicit variant slug: ${imported.file}`);
    const line = file.getLineAndCharacterOfPosition(metadata!.getStart(file)).line + 1;
    const result = { ...action, file: imported.file, line, variants };
    const previous = actions.get(action.slug);
    if (previous && JSON.stringify(previous) !== JSON.stringify(result)) throw new Error(`Conflicting action ${action.slug}`);
    actions.set(action.slug, result);
  }
  return [...actions.values()].sort((a, b) => a.slug < b.slug ? -1 : a.slug > b.slug ? 1 : 0);
}

export function generateActionModule(actions: ActionDefinition[], header: string, source: { repository: string; commit: string }): string {
  const quote = JSON.stringify;
  const optional = (value: string | null) => value === null ? 'None' : `Some(${quote(value)})`;
  return `${header}
// Literal metadata from the PF2e SystemActions registry; no upstream code is executed.
// ${source.repository.replace(/\.git$/, '')}/blob/${source.commit}/src/module/system/action-macros/index.ts
// Upstream copyright: Copyright 2019 Hooking. License: Apache-2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundryActionVariant {
    pub slug: &'static str,
    pub name_key: Option<&'static str>,
    pub cost: Option<&'static str>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundryActionDefinition {
    pub slug: &'static str,
    pub name_key: &'static str,
    pub cost: Option<&'static str>,
    pub file: &'static str,
    pub line: u32,
    pub variants: &'static [FoundryActionVariant],
}
static ACTIONS: &[FoundryActionDefinition] = &[
${actions.map(action => `    FoundryActionDefinition { slug: ${quote(action.slug)}, name_key: ${quote(action.name)}, cost: ${optional(action.cost)}, file: ${quote(action.file)}, line: ${action.line}, variants: &[${action.variants.map(variant => `FoundryActionVariant { slug: ${quote(variant.slug)}, name_key: ${optional(variant.name)}, cost: ${optional(variant.cost)} }`).join(', ')}] },`).join('\n')}
];
pub fn action_definition(slug: &str) -> Option<&'static FoundryActionDefinition> {
    ACTIONS.binary_search_by_key(&slug, |action| action.slug).ok().map(|index| &ACTIONS[index])
}
pub fn action_definitions() -> &'static [FoundryActionDefinition] { ACTIONS }
`;
}
