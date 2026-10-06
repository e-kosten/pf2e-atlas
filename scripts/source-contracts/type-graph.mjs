import { createHash } from 'node:crypto';
import path from 'node:path';
import ts from 'typescript';
import { discoverSourcePortfolio, PORTFOLIO_FILE } from './source-portfolio.mjs';

const digest = (text) => createHash('sha256').update(text).digest('hex').slice(0, 24);
const compare = (a, b) => a < b ? -1 : a > b ? 1 : 0;

/** Offline declaration discovery. This is neither a source parser nor a schema admission gate. */
export function extractTypeGraph(sourceRoot, options = {}) {
  const root = path.resolve(sourceRoot);
  const relative = (file) => {
    const normalized = file.replaceAll('\\', '/');
    const dependency = normalized.lastIndexOf('/node_modules/');
    return dependency >= 0 ? `node_modules/${normalized.slice(dependency + 14)}`
      : path.relative(root, file).replaceAll('\\', '/');
  };
  const cleanText = (text) => text.replaceAll(root.replaceAll('\\', '/'), '<source>')
    .replace(/(?:[A-Za-z]:)?\/[^\s"']*\/node_modules\//g, 'node_modules/');
  const diagnostics = [];
  const configPath = path.join(root, 'tsconfig.json');
  const read = ts.readConfigFile(configPath, ts.sys.readFile);
  if (read.error) return failure(read.error);
  const config = ts.parseJsonConfigFileContent(read.config, ts.sys, root, undefined, configPath);
  if (config.errors.length) return failure(...config.errors);
  const portfolio = options.roots ? null : discoverSourcePortfolio(root);
  const selections = options.roots ?? portfolio.selections;
  diagnostics.push(...(portfolio?.diagnostics ?? []));
  const maxNodes = options.maxNodes ?? 4000;
  if (!Number.isInteger(maxNodes) || maxNodes < 1) throw new Error('maxNodes must be a positive integer');
  const host = ts.createCompilerHost(config.options);
  const virtualFile = path.join(root, PORTFOLIO_FILE);
  const originalGetSourceFile = host.getSourceFile.bind(host);
  host.getSourceFile = (file, ...args) => portfolio && path.resolve(file) === virtualFile
    ? ts.createSourceFile(file, portfolio.virtualSource, ts.ScriptTarget.Latest, true)
    : originalGetSourceFile(file, ...args);
  const program = ts.createProgram([...config.fileNames, ...(portfolio ? [virtualFile] : [])], config.options, host);
  const checker = program.getTypeChecker();
  const nodes = new Map();
  const seen = new Map();
  const selectedDeclarations = new Set();
  const roots = [];

  function location(declaration) {
    const source = declaration.getSourceFile();
    const at = source.getLineAndCharacterOfPosition(declaration.getStart());
    return { file: relative(source.fileName), line: at.line + 1, column: at.character + 1 };
  }

  function declarations(symbol) {
    return (symbol?.getDeclarations() ?? []).map((declaration) => {
      selectedDeclarations.add(declaration);
      return location(declaration);
    }).sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  }

  // Named declaration ancestry avoids compiler-local IDs and line/offset identities.
  function declarationKey(declaration) {
    const names = [];
    for (let current = declaration; current && !ts.isSourceFile(current); current = current.parent) {
      if (current.name && typeof current.name.getText === 'function') names.unshift(current.name.getText());
      else if (ts.isTypeLiteralNode(current)) names.unshift('$object');
      else if (ts.isMappedTypeNode(current)) names.unshift('$mapped');
    }
    return `${relative(declaration.getSourceFile().fileName)}#${names.join('.') || '$anonymous'}`;
  }

  function label(type) {
    return cleanText(checker.typeToString(type, undefined,
      ts.TypeFormatFlags.NoTruncation | ts.TypeFormatFlags.UseAliasDefinedOutsideCurrentScope));
  }

  function identity(type) {
    const flags = type.flags;
    for (const [flag, name] of [[ts.TypeFlags.String, 'string'], [ts.TypeFlags.Number, 'number'],
      [ts.TypeFlags.Boolean, 'boolean'], [ts.TypeFlags.Null, 'null'], [ts.TypeFlags.Undefined, 'undefined'],
      [ts.TypeFlags.Never, 'never'], [ts.TypeFlags.Unknown, 'unknown'], [ts.TypeFlags.NonPrimitive, 'object']]) {
      if (flags === flag) return `primitive:${name}`;
    }
    if (flags & ts.TypeFlags.StringLiteral) return `literal:string:${JSON.stringify(type.value)}`;
    if (flags & ts.TypeFlags.NumberLiteral) return `literal:number:${type.value}`;
    if (flags & ts.TypeFlags.BooleanLiteral) return `literal:boolean:${type.intrinsicName}`;
    if (flags & ts.TypeFlags.Any) return type.intrinsicName === 'error'
      ? `unresolved:${digest(label(type))}` : 'primitive:any';
    const symbol = type.aliasSymbol ?? type.getSymbol();
    const origin = symbol?.getDeclarations()?.[0];
    const args = type.aliasTypeArguments ?? (type.objectFlags & ts.ObjectFlags.Reference ? checker.getTypeArguments(type) : []);
    if (origin) {
      // Several anonymous objects can share the same declaration ancestry, and
      // an inline object under Shared<T> resolves separately for each T. Their
      // resolved shape is part of their identity, while named declarations keep
      // an identity that survives ordinary field additions.
      const anonymous = !type.aliasSymbol && symbol.getName().startsWith('__');
      const suffix = anonymous ? argumentIdentity(type) : args.length ? args.map((arg) => argumentIdentity(arg)).join('|') : null;
      return `${declarationKey(origin)}${suffix === null ? '' : `@${digest(suffix)}`}`;
    }
    return `structural:${digest(argumentIdentity(type))}`;
  }

  function argumentIdentity(type, ancestors = new Set()) {
    const symbol = type.aliasSymbol ?? type.getSymbol();
    const origin = symbol?.getDeclarations()?.[0];
    const own = `${origin ? declarationKey(origin) : type.flags}:${type.intrinsicName ?? ''}:${label(type)}`;
    if (ancestors.has(type)) return own;
    const next = new Set(ancestors).add(type);
    const args = type.aliasTypeArguments ?? (type.objectFlags & ts.ObjectFlags.Reference ? checker.getTypeArguments(type) : []);
    const parts = args.map((arg) => argumentIdentity(arg, next));
    if (type.isUnion() || type.isIntersection()) {
      parts.push(...type.types.map((member) => argumentIdentity(member, next)).sort(compare));
    } else if (type.flags & ts.TypeFlags.Object && !type.aliasSymbol && symbol?.getName().startsWith('__')) {
      // Printed names are insufficient: two modules may each declare `Foo`.
      // Resolve member identities to retain the actual referenced declarations.
      parts.push(...checker.getPropertiesOfType(type).map((property) => {
        const declaration = property.valueDeclaration ?? property.declarations?.[0] ?? origin;
        return `${property.getName()}:${argumentIdentity(checker.getTypeOfSymbolAtLocation(property, declaration), next)}`;
      }).sort(compare));
      parts.push(...checker.getIndexInfosOfType(type).map((info) =>
        `${argumentIdentity(info.keyType, next)}:${argumentIdentity(info.type, next)}`).sort(compare));
    }
    return `${own}[${parts.join('|')}]`;
  }

  function problem(code, message, node, declaration) {
    diagnostics.push({ code, message, ...(node ? { node } : {}),
      ...(declaration ? { location: location(declaration) } : {}) });
  }

  function visit(type, context) {
    if (seen.has(type)) return seen.get(type);
    const id = identity(type);
    seen.set(type, id);
    if (nodes.has(id)) return id;
    if (nodes.size >= maxNodes) {
      const limit = 'unsupported:closure-limit';
      if (!nodes.has(limit)) {
        nodes.set(limit, { id: limit, kind: 'unsupported', reason: `Graph exceeds ${maxNodes} nodes` });
        problem('closure-limit', `Graph exceeds ${maxNodes} nodes; selected closure is incomplete`, limit);
      }
      seen.set(type, limit);
      return limit;
    }
    const symbol = type.aliasSymbol ?? type.getSymbol();
    const origin = symbol?.getDeclarations()?.[0] ?? context;
    const node = { id, kind: 'pending', ...(symbol && !symbol.getName().startsWith('__')
      ? { name: symbol.getName() } : {}), declaredAt: declarations(symbol) };
    // Register before traversing children: references can recur through arrays or aliases.
    nodes.set(id, node);
    const flags = type.flags;
    if (type.isUnion() || type.isIntersection()) {
      node.kind = type.isUnion() ? 'union' : 'intersection';
      node.members = type.types.map((member) => visit(member, origin)).sort(compare);
      // Intersections can carry inherited fields which are not repeated in every constituent.
      if (type.isIntersection()) node.fields = fields(type, origin);
    } else if (flags & (ts.TypeFlags.StringLiteral | ts.TypeFlags.NumberLiteral | ts.TypeFlags.BooleanLiteral)) {
      node.kind = 'literal';
      node.value = flags & ts.TypeFlags.BooleanLiteral ? type.intrinsicName === 'true' : type.value;
    } else if (flags & ts.TypeFlags.Any) {
      if (type.intrinsicName === 'error') {
        node.kind = 'unresolved';
        problem('unresolved-type', `Compiler could not resolve ${label(type)}`, id, origin);
      } else { node.kind = 'open'; node.domain = 'any'; }
    } else if (flags & ts.TypeFlags.Unknown) { node.kind = 'open'; node.domain = 'unknown'; }
    else if (flags & ts.TypeFlags.NonPrimitive) { node.kind = 'open'; node.domain = 'object'; }
    else if (flags & ts.TypeFlags.TemplateLiteral) {
      node.kind = 'template'; node.text = type.texts;
      node.parameters = type.types.map((parameter) => visit(parameter, origin));
    }
    else if (flags & (ts.TypeFlags.String | ts.TypeFlags.Number | ts.TypeFlags.Boolean |
      ts.TypeFlags.Null | ts.TypeFlags.Undefined | ts.TypeFlags.Never)) {
      node.kind = 'primitive'; node.value = label(type);
    } else if (checker.isTupleType(type)) {
      node.kind = 'tuple';
      node.elements = checker.getTypeArguments(type).map((element, index) => ({
        ref: visit(element, origin), optional: !!(type.target.elementFlags[index] & ts.ElementFlags.Optional),
        rest: !!(type.target.elementFlags[index] & (ts.ElementFlags.Rest | ts.ElementFlags.Variadic)),
      }));
      node.readonly = !!type.target.readonly;
    } else if (checker.isArrayType(type) ||
      (type.getSymbol()?.getName() === 'ReadonlyArray' && type.objectFlags & ts.ObjectFlags.Reference)) {
      node.kind = 'array';
      node.element = visit(checker.getTypeArguments(type)[0], origin);
      node.readonly = type.getSymbol()?.getName() === 'ReadonlyArray';
    } else if (flags & ts.TypeFlags.Object) {
      const calls = checker.getSignaturesOfType(type, ts.SignatureKind.Call);
      const constructors = checker.getSignaturesOfType(type, ts.SignatureKind.Construct);
      if (calls.length || constructors.length) {
        node.kind = 'unsupported'; node.reason = 'Function or constructor is not a persisted source value';
        problem('callable-source-type', node.reason, id, origin);
      } else if (type.getSymbol()?.getDeclarations()?.some(ts.isClassDeclaration)) {
        node.kind = 'unsupported'; node.reason = 'Runtime class instance is outside the source graph boundary';
        problem('class-instance-source-type', node.reason, id, origin);
      } else {
        node.kind = 'object';
        node.fields = fields(type, origin);
        if (type.objectFlags & ts.ObjectFlags.Interface) {
          const bases = checker.getBaseTypes(type) ?? [];
          if (bases.length) node.extends = bases.map((base) => visit(base, origin)).sort(compare);
        }
        node.indexSignatures = checker.getIndexInfosOfType(type).map((info) => ({
          key: visit(info.keyType, info.declaration), value: visit(info.type, info.declaration),
          readonly: info.isReadonly,
        })).sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
        if (!node.fields.length && !node.indexSignatures.length) {
          const explicitEmpty = origin && ((ts.isTypeLiteralNode(origin) && !origin.members.length)
            || (ts.isInterfaceDeclaration(origin) && !origin.members.length && !origin.heritageClauses?.length));
          if (explicitEmpty) { node.kind = 'open'; node.domain = 'non-nullish'; }
          else problem('empty-resolved-object', 'An empty resolved object requires an explicit source decision', id, origin);
        }
      }
    } else {
      node.kind = 'unsupported'; node.reason = `Unresolved or unsupported TypeScript construct: ${label(type)}`;
      problem('unsupported-type', node.reason, id, origin);
    }
    const args = type.aliasTypeArguments ?? (type.objectFlags & ts.ObjectFlags.Reference ? checker.getTypeArguments(type) : []);
    if (args.length && !['array', 'tuple', 'unsupported'].includes(node.kind)) {
      // Generic inputs are compiler provenance, not necessarily persisted values.
      // Foundry SourceFromSchema<TSchema>, for example, takes DataField classes.
      // Resolved properties above are the value closure; traversing schema inputs
      // as values would incorrectly pull all Foundry runtime methods into it.
      node.typeArguments = args.map((arg) => ({ expression: label(arg),
        declaredAt: (arg.aliasSymbol ?? arg.getSymbol())?.getDeclarations()?.map(location) ?? [] }));
    }
    // Union null/undefined are retained as their own graph nodes, not erased by serde policy.
    return id;
  }

  function fields(type, context) {
    return checker.getPropertiesOfType(type).map((property) => {
      const declaration = property.valueDeclaration ?? property.declarations?.[0];
      const symbolKey = property.getName().startsWith('__@');
      const name = symbolKey ? declaration?.name?.getText() ?? '$symbol' : property.getName();
      if (symbolKey) problem('symbol-keyed-field', `Symbol-keyed field ${name} is not a JSON member`, undefined, declaration);
      const resolved = checker.getTypeOfSymbolAtLocation(property, declaration ?? context);
      const alternatives = resolved.isUnion() ? resolved.types : [resolved];
      const present = alternatives.filter((part) => !(part.flags & ts.TypeFlags.Undefined));
      return { name, ref: visit(resolved, declaration ?? context),
        optional: !!(property.flags & ts.SymbolFlags.Optional),
        nullable: alternatives.some((part) => !!(part.flags & ts.TypeFlags.Null)),
        undefinedAllowed: alternatives.some((part) => !!(part.flags & ts.TypeFlags.Undefined)),
        forbidden: present.every((part) => !!(part.flags & ts.TypeFlags.Never)),
        declaredAt: declarations(property) };
    }).sort((a, b) => compare(a.name, b.name));
  }

  for (const selection of [...selections].sort((a, b) => compare(a.name, b.name))) {
    const source = program.getSourceFile(path.resolve(root, selection.file));
    const module = source && checker.getSymbolAtLocation(source);
    let symbol = module && checker.getExportsOfModule(module).find((entry) => entry.getName() === selection.name);
    if (!symbol) {
      problem('missing-root', `Export ${selection.name} was not found in ${selection.file}`);
      roots.push({ ...selection, ref: null });
      continue;
    }
    if (symbol.flags & ts.SymbolFlags.Alias) symbol = checker.getAliasedSymbol(symbol);
    const type = checker.getDeclaredTypeOfSymbol(symbol);
    const ref = visit(type, symbol.declarations?.[0]);
    roots.push({ ...selection, ref });
  }

  if (portfolio) {
    for (const family of portfolio.families) {
      const ref = roots.find((entry) => entry.documentKind === family.documentKind)?.ref;
      const literals = new Set();
      const inspected = new Set();
      function discriminate(id) {
        if (inspected.has(id)) return;
        inspected.add(id);
        const node = nodes.get(id);
        if (node?.kind === 'union') { for (const member of node.members) discriminate(member); }
        else {
          const field = node?.fields?.find((entry) => entry.name === 'type');
          const type = nodes.get(field?.ref);
          if (type?.kind === 'literal' && typeof type.value === 'string') literals.add(type.value);
          else problem('family-discriminator', `${family.documentKind} source member has no literal type discriminator`, id);
        }
      }
      discriminate(ref);
      family.discovered = [...literals].sort(compare);
      if (JSON.stringify(family.registered) !== JSON.stringify(family.discovered)) {
        problem('family-coverage', `${family.documentKind} registered families differ from source discriminators`);
      }
    }
  }

  const projectDiagnostics = { selected: [], unrelated: [] };
  const selectedRanges = [...selectedDeclarations].map((declaration) => ({
    file: declaration.getSourceFile().fileName, start: declaration.getStart(), end: declaration.getEnd(),
  }));
  for (const diagnostic of ts.getPreEmitDiagnostics(program)) {
    const serialized = serializeDiagnostic(diagnostic);
    const selected = diagnostic.file && ((portfolio && diagnostic.file.fileName === virtualFile)
      || (diagnostic.start !== undefined && selectedRanges.some((range) =>
        range.file === diagnostic.file.fileName && range.start <= diagnostic.start && diagnostic.start < range.end)));
    projectDiagnostics[selected ? 'selected' : 'unrelated'].push(serialized);
  }
  for (const group of Object.values(projectDiagnostics)) group.sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  diagnostics.sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  const complete = !diagnostics.length && !projectDiagnostics.selected.some((entry) => entry.category === 'error');
  return { format: 'atlas-source-type-graph/v1', typescript: ts.version, complete,
    status: complete ? 'complete' : 'incomplete',
    roots, ...(portfolio ? { portfolio: { documentKinds: portfolio.documentKinds, families: portfolio.families,
      ruleKeys: portfolio.ruleKeys } } : {}),
    nodes: [...nodes.values()].sort((a, b) => compare(a.id, b.id)), diagnostics, projectDiagnostics };

  function serializeDiagnostic(diagnostic) {
    return { code: diagnostic.code, category: ts.DiagnosticCategory[diagnostic.category].toLowerCase(),
      message: cleanText(ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n')),
      ...(diagnostic.file ? { location: {
        file: relative(diagnostic.file.fileName),
        line: diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line + 1,
        column: diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start ?? 0).character + 1,
      } } : {}) };
  }

  function failure(...errors) {
    return { format: 'atlas-source-type-graph/v1', typescript: ts.version, complete: false, status: 'incomplete',
      roots: [], nodes: [], diagnostics: errors.map((error) => ({ code: 'configuration-error',
        message: cleanText(ts.flattenDiagnosticMessageText(error.messageText, '\n')) })),
      projectDiagnostics: { selected: [], unrelated: [] } };
  }
}
