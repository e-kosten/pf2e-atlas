import ts from 'typescript';
export type ImmunityPredicate = { kind: 'trait' | 'actorKind' | 'authored'; value: string }
  | { kind: 'all' | 'any'; terms: ImmunityPredicate[] } | { kind: 'not'; term: ImmunityPredicate };
export interface TraitImmunity { type: string; predicate: ImmunityPredicate; labelKey: string; line: number }
type Binding = { kind: 'traits' | 'immunities' | 'authored' }
  | { kind: 'values'; values: string[] } | { kind: 'literal'; value: string };
/** Every statement is accounted for; unknown semantics stop generation. */
export function traitImmunities(text: string, knownTraits: ReadonlySet<string>): TraitImmunity[] {
  const file = ts.createSourceFile('helpers.ts', text, ts.ScriptTarget.Latest, true);
  const functions = file.statements.filter(ts.isFunctionDeclaration).filter(n => n.name?.text === 'setImmunitiesFromTraits');
  const fn = functions[0];
  if (functions.length !== 1 || !fn.body || fn.parameters.length !== 1 || !ts.isIdentifier(fn.parameters[0].name)
    || fn.parameters[0].type?.getText(file) !== 'CreaturePF2e') throw new Error('Unsupported immunity helper signature');
  const actor = fn.parameters[0].name.text, result: TraitImmunity[] = [];
  function fail(node: ts.Node): never { throw new Error('Unsupported immunity syntax: ' + node.getText(file)); }
  const unwrap = (node: ts.Expression): ts.Expression => ts.isAsExpression(node) || ts.isParenthesizedExpression(node) ? unwrap(node.expression) : node;
  const path = (node: ts.Expression): string => {
    node = unwrap(node);
    return ts.isIdentifier(node) ? node.text : ts.isPropertyAccessExpression(node) ? path(node.expression) + '.' + node.name.text : '';
  };
  const literal = (node: ts.Expression, env: Map<string, Binding>): string => {
    node = unwrap(node);
    if (ts.isStringLiteral(node)) return node.text;
    const binding = ts.isIdentifier(node) ? env.get(node.text) : undefined;
    return binding?.kind === 'literal' ? binding.value : fail(node);
  };
  const values = (node: ts.Expression, env: Map<string, Binding>): string[] => {
    node = unwrap(node);
    if (ts.isArrayLiteralExpression(node)) return node.elements.map(n => literal(n as ts.Expression, env));
    const binding = ts.isIdentifier(node) ? env.get(node.text) : undefined;
    return binding?.kind === 'values' ? binding.values : fail(node);
  };
  const predicate = (node: ts.Expression, env: Map<string, Binding>): ImmunityPredicate => {
    node = unwrap(node);
    if (ts.isPrefixUnaryExpression(node) && node.operator === ts.SyntaxKind.ExclamationToken) return { kind: 'not', term: predicate(node.operand, env) };
    if (ts.isBinaryExpression(node) && [ts.SyntaxKind.AmpersandAmpersandToken, ts.SyntaxKind.BarBarToken].includes(node.operatorToken.kind))
      return { kind: node.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken ? 'all' : 'any', terms: [predicate(node.left, env), predicate(node.right, env)] };
    if (ts.isCallExpression(node) && node.arguments.length === 1 && ts.isPropertyAccessExpression(node.expression)) {
      const target = node.expression.expression, method = node.expression.name.text;
      if (path(target) === actor && method === 'isOfType') return { kind: 'actorKind', value: literal(node.arguments[0], env) };
      const binding = ts.isIdentifier(target) ? env.get(target.text) : undefined;
      if (method === 'includes' && (binding?.kind === 'traits' || binding?.kind === 'authored')) {
        const value = literal(node.arguments[0], env);
        if (binding.kind === 'traits' && !knownTraits.has(value)) throw new Error('Unknown immunity trait: ' + value);
        return { kind: binding.kind === 'traits' ? 'trait' : 'authored', value };
      }
    }
    return fail(node);
  };
  const list = (node: ts.Statement): readonly ts.Statement[] => ts.isBlock(node) ? node.statements : [node];
  const walk = (statements: readonly ts.Statement[], initialEnv: Map<string, Binding>, initialGuards: ImmunityPredicate[], topLevel = false): void => {
    const env = new Map(initialEnv), guards = [...initialGuards];
    for (const statement of statements) {
      if (ts.isIfStatement(statement)) {
        const guard = predicate(statement.expression, env), body = list(statement.thenStatement);
        if (body.length === 1 && ts.isReturnStatement(body[0]) && !body[0].expression && !statement.elseStatement) {
          if (!topLevel) fail(statement);
          guards.push({ kind: 'not', term: guard });
        }
        else {
          walk(body, env, [...guards, guard]);
          if (statement.elseStatement) walk(list(statement.elseStatement), env, [...guards, { kind: 'not', term: guard }]);
        }
      } else if (ts.isVariableStatement(statement)) {
        if (!(statement.declarationList.flags & ts.NodeFlags.Const)) fail(statement);
        for (const declaration of statement.declarationList.declarations) {
          if (!ts.isIdentifier(declaration.name) || !declaration.initializer) fail(declaration);
          const init = unwrap(declaration.initializer!), name = declaration.name.getText(file), source = path(init);
          if (source === actor + '.system.traits.value') env.set(name, { kind: 'traits' });
          else if (source === actor + '.attributes.immunities') env.set(name, { kind: 'immunities' });
          else if (ts.isCallExpression(init) && ts.isPropertyAccessExpression(init.expression) && init.expression.name.text === 'map'
            && ts.isIdentifier(init.expression.expression) && env.get(init.expression.expression.text)?.kind === 'immunities' && init.arguments.length === 1) {
            const callback = init.arguments[0];
            if (!ts.isArrowFunction(callback) || callback.parameters.length !== 1 || !ts.isIdentifier(callback.parameters[0].name)
              || !ts.isPropertyAccessExpression(callback.body) || path(callback.body) !== callback.parameters[0].name.text + '.type') fail(init);
            env.set(name, { kind: 'authored' });
          } else env.set(name, { kind: 'values', values: values(init, env) });
        }
      } else if (ts.isForOfStatement(statement)) {
        if (statement.awaitModifier || !ts.isVariableDeclarationList(statement.initializer) || !(statement.initializer.flags & ts.NodeFlags.Const)
          || statement.initializer.declarations.length !== 1 || !ts.isIdentifier(statement.initializer.declarations[0].name)) fail(statement);
        const name = statement.initializer.declarations[0].name.getText(file);
        for (const value of values(statement.expression, env)) {
          const loop = new Map(env); loop.set(name, { kind: 'literal', value }); walk(list(statement.statement), loop, guards);
        }
      } else if (ts.isExpressionStatement(statement) && ts.isCallExpression(statement.expression)) {
        const call = statement.expression;
        if (!ts.isPropertyAccessExpression(call.expression) || call.expression.name.text !== 'push' || !ts.isIdentifier(call.expression.expression)
          || env.get(call.expression.expression.text)?.kind !== 'immunities' || call.arguments.length !== 1 || !ts.isNewExpression(call.arguments[0])) fail(statement);
        const create = call.arguments[0] as ts.NewExpression;
        if (path(create.expression) !== 'Immunity' || create.arguments?.length !== 1 || !ts.isObjectLiteralExpression(create.arguments[0])) fail(statement);
        const fields = new Map<string, ts.Expression>();
        for (const member of (create.arguments![0] as ts.ObjectLiteralExpression).properties) {
          if (!ts.isPropertyAssignment(member) || !ts.isIdentifier(member.name) || fields.has(member.name.text)) fail(member);
          fields.set(member.name.getText(file), (member as ts.PropertyAssignment).initializer);
        }
        const source = fields.get('source'), type = fields.get('type');
        if (fields.size !== 2 || !source || !type || !ts.isCallExpression(source) || path(source.expression) !== '_loc' || source.arguments.length !== 1) fail(statement);
        result.push({ type: literal(type!, env), labelKey: literal((source as ts.CallExpression).arguments[0], env),
          predicate: { kind: 'all', terms: [...guards] }, line: file.getLineAndCharacterOfPosition(statement.getStart(file)).line + 1 });
      } else fail(statement);
    }
  };
  walk(fn.body.statements, new Map(), [], true);
  if (!result.length) throw new Error('No trait immunities extracted');
  return result;
}
