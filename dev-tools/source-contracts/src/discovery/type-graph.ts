import type { DiscoveryDiagnostic, FieldSerialization, GraphBase, GraphField, GraphNode, GraphShape, Location, ProjectDiagnostic, RootSelection, RuleArrayInput, TypeGraph } from '../contracts.js';
import { declarationName, intrinsicName, objectFlags, typeArguments } from './compiler-types.js';
import { createHash } from 'node:crypto';
import path from 'node:path';
import ts from 'typescript';
import { choicePredicateInputs, modifierCallbackProjection, predicateSourceArray } from './source-serialization.js';
import { discoverSourcePortfolio, PORTFOLIO_FILE } from './source-portfolio.js';

const digest = (text: string) => createHash('sha256').update(text).digest('hex').slice(0, 24);
const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0;

/** Offline declaration discovery. This is neither a source parser nor a schema admission gate. */
export function extractTypeGraph(sourceRoot: string, options: { roots?: RootSelection[]; maxNodes?: number } = {}): TypeGraph {
  const root = path.resolve(sourceRoot);
  const relative = (file: string) => {
    const normalized = file.replaceAll('\\', '/');
    const dependency = normalized.lastIndexOf('/node_modules/');
    return dependency >= 0 ? `node_modules/${normalized.slice(dependency + 14)}`
      : path.relative(root, file).replaceAll('\\', '/');
  };
  const cleanText = (text: string) => text.replaceAll(root.replaceAll('\\', '/'), '<source>')
    .replace(/(?:[A-Za-z]:)?\/[^\s"']*\/node_modules\//g, 'node_modules/');
  const diagnostics: DiscoveryDiagnostic[] = [];
  const configPath = path.join(root, 'tsconfig.json');
  const read = ts.readConfigFile(configPath, ts.sys.readFile);
  if (read.error) return failure(read.error);
  const config = ts.parseJsonConfigFileContent(read.config, ts.sys, root, undefined, configPath);
  if (config.errors.length) return failure(...config.errors);
  const portfolio = options.roots ? null : discoverSourcePortfolio(root);
  const selections = options.roots ?? portfolio!.selections;
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
  const nodes = new Map<string, GraphNode | (GraphBase & { kind: "pending" })>();
  const seen = new Map<ts.Type, string>();
  const selectedDeclarations = new Set<ts.Declaration>();
  const roots: TypeGraph["roots"] = [];

  function location(declaration: ts.Node): Location {
    const source = declaration.getSourceFile();
    const at = source.getLineAndCharacterOfPosition(declaration.getStart());
    return { file: relative(source.fileName), line: at.line + 1, column: at.character + 1 };
  }

  function declarations(symbol: ts.Symbol | undefined): Location[] {
    return (symbol?.getDeclarations() ?? []).map((declaration) => {
      selectedDeclarations.add(declaration);
      return location(declaration);
    }).sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  }

  // Named declaration ancestry avoids compiler-local IDs and line/offset identities.
  function declarationKey(declaration: ts.Node): string {
    const names = [];
    for (let current: ts.Node | undefined = declaration; current && !ts.isSourceFile(current); current = current.parent) {
      const name = declarationName(current);
      if (name) names.unshift(name.getText());
      else if (ts.isTypeLiteralNode(current)) names.unshift('$object');
      else if (ts.isMappedTypeNode(current)) names.unshift('$mapped');
    }
    return `${relative(declaration.getSourceFile().fileName)}#${names.join('.') || '$anonymous'}`;
  }

  function label(type: ts.Type): string {
    return cleanText(checker.typeToString(type, undefined,
      ts.TypeFormatFlags.NoTruncation | ts.TypeFormatFlags.UseAliasDefinedOutsideCurrentScope));
  }

  function identity(type: ts.Type): string {
    const flags = type.flags;
    for (const [flag, name] of [[ts.TypeFlags.String, 'string'], [ts.TypeFlags.Number, 'number'],
      [ts.TypeFlags.Boolean, 'boolean'], [ts.TypeFlags.Null, 'null'], [ts.TypeFlags.Undefined, 'undefined'],
      [ts.TypeFlags.Never, 'never'], [ts.TypeFlags.Unknown, 'unknown'], [ts.TypeFlags.NonPrimitive, 'object']]) {
      if (flags === flag) return `primitive:${name}`;
    }
    if (flags & ts.TypeFlags.StringLiteral) return `literal:string:${JSON.stringify((type as ts.StringLiteralType | ts.NumberLiteralType).value)}`;
    if (flags & ts.TypeFlags.NumberLiteral) return `literal:number:${(type as ts.StringLiteralType | ts.NumberLiteralType).value}`;
    if (flags & ts.TypeFlags.BooleanLiteral) return `literal:boolean:${intrinsicName(type)}`;
    if (flags & ts.TypeFlags.Any) return intrinsicName(type) === 'error'
      ? `unresolved:${digest(label(type))}` : 'primitive:any';
    const symbol = type.aliasSymbol ?? type.getSymbol();
    const origin = symbol?.getDeclarations()?.[0];
    const args = typeArguments(type, checker);
    if (origin) {
      // Several anonymous objects can share the same declaration ancestry, and
      // an inline object under Shared<T> resolves separately for each T. Their
      // resolved shape is part of their identity, while named declarations keep
      // an identity that survives ordinary field additions.
      const anonymous = !type.aliasSymbol && symbol!.getName().startsWith('__');
      const suffix = anonymous ? argumentIdentity(type) : args.length ? args.map((arg) => argumentIdentity(arg)).join('|') : null;
      return `${declarationKey(origin)}${suffix === null ? '' : `@${digest(suffix)}`}`;
    }
    return `structural:${digest(argumentIdentity(type))}`;
  }

  function argumentIdentity(type: ts.Type, ancestors = new Set<ts.Type>()): string {
    const symbol = type.aliasSymbol ?? type.getSymbol();
    const origin = symbol?.getDeclarations()?.[0];
    const own = `${origin ? declarationKey(origin) : type.flags}:${intrinsicName(type) ?? ''}:${label(type)}`;
    if (ancestors.has(type)) return own;
    const next = new Set(ancestors).add(type);
    const args = typeArguments(type, checker);
    const parts = args.map((arg) => argumentIdentity(arg, next));
    if (type.isUnion() || type.isIntersection()) {
      parts.push(...type.types.map((member) => argumentIdentity(member, next)).sort(compare));
    } else if (type.flags & ts.TypeFlags.Object && !type.aliasSymbol && symbol?.getName().startsWith('__')) {
      // Printed names are insufficient: two modules may each declare `Foo`.
      // Resolve member identities to retain the actual referenced declarations.
      parts.push(...checker.getPropertiesOfType(type).map((property) => {
        const declaration = property.valueDeclaration ?? property.declarations?.[0] ?? origin;
        return `${property.getName()}:${argumentIdentity(checker.getTypeOfSymbolAtLocation(property, declaration!), next)}`;
      }).sort(compare));
      parts.push(...checker.getIndexInfosOfType(type).map((info) =>
        `${argumentIdentity(info.keyType, next)}:${argumentIdentity(info.type, next)}`).sort(compare));
    }
    return `${own}[${parts.join('|')}]`;
  }

  function problem(code: string, message: string, node?: string, declaration?: ts.Node) {
    diagnostics.push({ code, message, ...(node ? { node } : {}),
      ...(declaration ? { location: location(declaration) } : {}) });
  }

  function visit(type: ts.Type, context?: ts.Node): string {
    if (seen.has(type)) return seen.get(type)!;
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
    const pending: GraphBase & { kind: 'pending' } = { id, kind: 'pending', ...(symbol && !symbol!.getName().startsWith('__')
      ? { name: symbol.getName() } : {}), declaredAt: declarations(symbol) };
    // Register before traversing children: references can recur through arrays or aliases.
    nodes.set(id, pending);
    let shape: GraphShape;
    const flags = type.flags;
    if (type.isUnion() || type.isIntersection()) {
      const members = type.types.map((member) => visit(member, origin)).sort(compare);
      shape = type.isUnion() ? { kind: 'union', members } : { kind: 'intersection', members, fields: fields(type, origin),
        indexSignatures: indices(type),
        ...(checker.isTypeAssignableTo(type, checker.getNeverType()) ? { impossible: true } : {}) };
    } else if (flags & (ts.TypeFlags.StringLiteral | ts.TypeFlags.NumberLiteral | ts.TypeFlags.BooleanLiteral)) {
      shape = { kind: 'literal', value: flags & ts.TypeFlags.BooleanLiteral ? intrinsicName(type) === 'true' : (type as ts.StringLiteralType | ts.NumberLiteralType).value };
    } else if (flags & ts.TypeFlags.Any) {
      if (intrinsicName(type) === 'error') {
        shape = { kind: 'unresolved' };
        problem('unresolved-type', `Compiler could not resolve ${label(type)}`, id, origin);
      } else { shape = { kind: 'open', domain: 'any' }; }
    } else if (flags & ts.TypeFlags.Unknown) { shape = { kind: 'open', domain: 'unknown' }; }
    else if (flags & ts.TypeFlags.NonPrimitive) { shape = { kind: 'open', domain: 'object' }; }
    else if (flags & ts.TypeFlags.TemplateLiteral) {
      const template = type as ts.TemplateLiteralType;
      shape = { kind: 'template', text: template.texts, parameters: template.types.map((parameter) => visit(parameter, origin)) };
    }
    else if (flags & (ts.TypeFlags.String | ts.TypeFlags.Number | ts.TypeFlags.Boolean |
      ts.TypeFlags.Null | ts.TypeFlags.Undefined | ts.TypeFlags.Never)) {
      shape = { kind: 'primitive', value: label(type) };
    } else if (checker.isTupleType(type)) {
      const tuple = type as ts.TupleTypeReference;
      shape = { kind: 'tuple', elements: checker.getTypeArguments(tuple).map((element, index) => ({
        ref: visit(element, origin), optional: !!(tuple.target.elementFlags[index] & ts.ElementFlags.Optional),
        rest: !!(tuple.target.elementFlags[index] & (ts.ElementFlags.Rest | ts.ElementFlags.Variadic)),
      })), readonly: !!tuple.target.readonly };
    } else if (checker.isArrayType(type) ||
      (type.getSymbol()?.getName() === 'ReadonlyArray' && objectFlags(type) & ts.ObjectFlags.Reference)) {
      shape = { kind: 'array', element: visit(typeArguments(type, checker)[0], origin), readonly: type.getSymbol()?.getName() === 'ReadonlyArray' };
    } else if (flags & ts.TypeFlags.Object) {
      const calls = checker.getSignaturesOfType(type, ts.SignatureKind.Call);
      const constructors = checker.getSignaturesOfType(type, ts.SignatureKind.Construct);
      if (calls.length || constructors.length) {
        shape = { kind: 'unsupported', reason: 'Function or constructor is not a persisted source value' };
        problem('callable-source-type', shape.reason, id, origin);
      } else if (type.getSymbol()?.getDeclarations()?.some(ts.isClassDeclaration)) {
        const projection = predicateSourceArray(type, checker, relative);
        if (projection && projection.error === undefined) {
          shape = { kind: 'array', element: visit(checker.getTypeArguments(projection.raw)[0], origin), readonly: false, serialization: { basis: 'array-subclass', rawRef: visit(projection.raw, origin),
            method: 'toObject', declaredAt: declarations(projection.method) } };
        } else {
          shape = { kind: 'unsupported', reason: projection?.error ?? 'Runtime class instance is outside the source graph boundary' };
          problem('class-instance-source-type', shape.reason, id, origin);
        }
      } else {
        const objectFields = fields(type, origin);
        const bases = objectFlags(type) & ts.ObjectFlags.Interface ? checker.getBaseTypes(type as ts.InterfaceType) ?? [] : [];
        const object: Extract<GraphShape, { kind: 'object' }> = { kind: 'object', fields: objectFields,
          ...(bases.length ? { extends: bases.map((base) => visit(base, origin)).sort(compare) } : {}), indexSignatures: [] };
        object.indexSignatures = indices(type);
        if (!object.fields.length && !object.indexSignatures.length) {
          const explicitEmpty = origin && ((ts.isTypeLiteralNode(origin) && !origin.members.length)
            || (ts.isInterfaceDeclaration(origin) && !origin.members.length && !origin.heritageClauses?.length));
          if (!explicitEmpty) problem('empty-resolved-object', 'An empty resolved object requires an explicit source decision', id, origin);
          shape = explicitEmpty ? { ...object, kind: 'open', domain: 'non-nullish' } : object;
        } else { shape = object; }
      }
    } else {
      shape = { kind: 'unsupported', reason: `Unresolved or unsupported TypeScript construct: ${label(type)}` };
      problem('unsupported-type', shape.reason, id, origin);
    }
    const node: GraphNode = { ...pending, ...shape };
    nodes.set(id, node);
    const args = typeArguments(type, checker);
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

  function indices(type: ts.Type) {
    return checker.getIndexInfosOfType(type).map(info => ({
      key: visit(info.keyType, info.declaration), value: visit(info.type, info.declaration),
      readonly: info.isReadonly,
    })).sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  }

  function fields(type: ts.Type, context?: ts.Node): GraphField[] {
    return checker.getPropertiesOfType(type).map((property): GraphField => {
      const declaration = property.valueDeclaration ?? property.declarations?.[0];
      const symbolKey = property.getName().startsWith('__@');
      const name = symbolKey ? (declaration && declarationName(declaration))?.getText() ?? '$symbol' : property.getName();
      if (symbolKey) problem('symbol-keyed-field', `Symbol-keyed field ${name} is not a JSON member`, undefined, declaration);
      const resolved = checker.getTypeOfSymbolAtLocation(property, (declaration ?? context)!);
      const projection = modifierCallbackProjection(type, property, resolved, checker, relative);
      if (projection?.error) problem('source-serialization-drift', projection.error, undefined, declaration);
      if (projection && projection.error === undefined) {
        const ref = 'primitive:never';
        if (!nodes.has(ref)) nodes.set(ref, { id: ref, kind: 'primitive', value: 'never' });
        return { name, ref, optional: true, nullable: false, undefinedAllowed: true,
          forbidden: true, declaredAt: declarations(property), serialization: {
            basis: 'omitted-function-property', declaredType: label(resolved),
            declaredOptional: projection.declaredOptional,
          } };
      }
      const choice = choicePredicateInputs(type, property, resolved, checker, relative);
      if (choice?.error) problem('source-serialization-drift', choice.error, undefined, declaration);
      let sourceRef: string | undefined;
      let serialization: FieldSerialization | undefined;
      if (choice && choice.error === undefined) {
        sourceRef = `${identity(type)}#${name}:constructor-input`;
        const members = [...new Set(choice.inputs.map((input) => visit(input, declaration)))].sort(compare);
        nodes.set(sourceRef, { id: sourceRef, kind: 'union', members, declaredAt: declarations(property) });
        serialization = { basis: 'predicate-constructor-input', declaredRef: visit(choice.predicate, declaration),
          declaredType: label(resolved), declaredAt: declarations(choice.constructor) };
      }
      const alternatives = resolved.isUnion() ? resolved.types : [resolved];
      const present = alternatives.filter((part) => !(part.flags & ts.TypeFlags.Undefined));
      return { name, ref: sourceRef ?? visit(resolved, declaration ?? context),
        optional: !!(property.flags & ts.SymbolFlags.Optional),
        nullable: alternatives.some((part) => !!(part.flags & ts.TypeFlags.Null)),
        undefinedAllowed: alternatives.some((part) => !!(part.flags & ts.TypeFlags.Undefined)),
        forbidden: present.every((part) => !!(part.flags & ts.TypeFlags.Never)),
        declaredAt: declarations(property), ...(serialization ? { serialization } : {}) };
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
    const arrayInputs:RuleArrayInput[]=[];
    if (selection.ruleKey && module) {
      const schemaSymbol=checker.getExportsOfModule(module).find(entry=>entry.getName()===`RuleSchema_${Buffer.from(selection.ruleKey!).toString('hex')}`);
      if (!schemaSymbol) problem('missing-rule-schema',`Cannot inspect schema fields for ${selection.ruleKey}`,ref);
      else for (const property of checker.getPropertiesOfType(checker.getDeclaredTypeOfSymbol(schemaSymbol))) {
        const declaration=property.valueDeclaration??property.declarations?.[0];
        if (!declaration) continue;
        const fieldType=checker.getTypeOfSymbolAtLocation(property,declaration);
        const symbol=fieldType.getSymbol();
        const fieldClass=symbol?.getName();
        const expectedFile=fieldClass==='ArrayField'?'types/foundry/common/data/fields.d.ts'
          :fieldClass==='StrictArrayField'?'src/module/system/schema-data-fields.ts':null;
        if (!expectedFile || !symbol?.getDeclarations()?.some(declaration=>relative(declaration.getSourceFile().fileName)===expectedFile)) continue;
        const node=nodes.get(ref);
        const sourceField=node && 'fields' in node ? node.fields?.find(field=>field.name===property.getName()):undefined;
        if (!sourceField) {problem('missing-array-source-field',`Missing source field ${selection.ruleKey}.${property.getName()}`,ref);continue;}
        const sourceNode=nodes.get(sourceField.ref);
        const arrays=(sourceNode?.kind==='union'?sourceNode.members:[sourceField.ref]).map(ref=>nodes.get(ref)).filter(node=>node?.kind==='array');
        if (arrays.length!==1) {problem('array-source-shape',`Unresolved array shape ${selection.ruleKey}.${property.getName()}`,ref);continue;}
        const array=arrays[0]!;
        arrayInputs.push({field:property.getName(),arrayRef:array.id,elementRef:array.element,
          fieldClass:fieldClass as RuleArrayInput['fieldClass'],declaredAt:declarations(property)});
      }
    }
    roots.push({ ...selection, ref, ...(selection.ruleKey?{arrayInputs:arrayInputs.sort((a,b)=>compare(a.field,b.field))}:{}) });
  }

  if (portfolio) {
    for (const family of portfolio.families) {
      const ref = roots.find((entry) => entry.documentKind === family.documentKind)?.ref;
      const literals = new Set<string>();
      const inspected = new Set<string | null | undefined>();
      function discriminate(id: string | null | undefined) {
        if (inspected.has(id)) return;
        inspected.add(id);
        const node = id ? nodes.get(id) : undefined;
        if (node?.kind === 'union') { for (const member of node.members) discriminate(member); }
        else {
          const field = node && 'fields' in node ? node.fields?.find((entry) => entry.name === 'type') : undefined;
          const type = field && nodes.get(field.ref);
          if (type?.kind === 'literal' && typeof type.value === 'string') literals.add(type.value);
          else problem('family-discriminator', `${family.documentKind} source member has no literal type discriminator`, id ?? undefined);
        }
      }
      discriminate(ref);
      family.discovered = [...literals].sort(compare);
      if (JSON.stringify(family.registered) !== JSON.stringify(family.discovered)) {
        problem('family-coverage', `${family.documentKind} registered families differ from source discriminators`);
      }
    }
  }

  const projectDiagnostics: TypeGraph["projectDiagnostics"] = { selected: [], unrelated: [] };
  const selectedRanges = [...selectedDeclarations].map((declaration) => ({
    file: declaration.getSourceFile().fileName, start: declaration.getStart(), end: declaration.getEnd(),
  }));
  for (const diagnostic of ts.getPreEmitDiagnostics(program)) {
    const serialized = serializeDiagnostic(diagnostic);
    const selected = diagnostic.file && ((portfolio && diagnostic.file.fileName === virtualFile)
      || (diagnostic.start !== undefined && selectedRanges.some((range) =>
        range.file === diagnostic.file!.fileName && range.start <= diagnostic.start! && diagnostic.start! < range.end)));
    projectDiagnostics[selected ? 'selected' : 'unrelated'].push(serialized);
  }
  for (const group of Object.values(projectDiagnostics)) group.sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  diagnostics.sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  const complete = !diagnostics.length && !projectDiagnostics.selected.some((entry) => entry.category === 'error');
  return { format: 'atlas-source-type-graph/v1', typescript: ts.version, complete,
    status: complete ? 'complete' : 'incomplete',
    roots, ...(portfolio ? { portfolio: { documentKinds: portfolio.documentKinds, families: portfolio.families,
      ruleKeys: portfolio.ruleKeys } } : {}),
    nodes: [...nodes.values()].map((node): GraphNode => { if (node.kind === "pending") throw new Error(`Unfinished graph node ${node.id}`); return node; }).sort((a, b) => compare(a.id, b.id)), diagnostics, projectDiagnostics };

  function serializeDiagnostic(diagnostic: ts.Diagnostic): ProjectDiagnostic {
    return { code: diagnostic.code, category: ts.DiagnosticCategory[diagnostic.category].toLowerCase(),
      message: cleanText(ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n')),
      ...(diagnostic.file ? { location: {
        file: relative(diagnostic.file.fileName),
        line: diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line + 1,
        column: diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start ?? 0).character + 1,
      } } : {}) };
  }

  function failure(...errors: ts.Diagnostic[]): TypeGraph {
    return { format: 'atlas-source-type-graph/v1', typescript: ts.version, complete: false, status: 'incomplete',
      roots: [], nodes: [], diagnostics: errors.map((error) => ({ code: 'configuration-error',
        message: cleanText(ts.flattenDiagnosticMessageText(error.messageText, '\n')) })),
      projectDiagnostics: { selected: [], unrelated: [] } };
  }
}
