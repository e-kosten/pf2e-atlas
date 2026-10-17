import ts from 'typescript';

/** Access the compiler's internal parse diagnostics without widening the AST. */
export function parseDiagnostics(source: ts.SourceFile): readonly ts.DiagnosticWithLocation[] {
  return (source as ts.SourceFile & { parseDiagnostics: readonly ts.DiagnosticWithLocation[] }).parseDiagnostics;
}
/** Intrinsic names distinguish the compiler's unresolved-error type from explicit any. */
export function intrinsicName(type: ts.Type): string | undefined {
  return (type as ts.Type & { intrinsicName?: string }).intrinsicName;
}
export function objectFlags(type: ts.Type): ts.ObjectFlags {
  return type.flags & ts.TypeFlags.Object ? (type as ts.ObjectType).objectFlags : 0;
}
export function typeArguments(type: ts.Type, checker: ts.TypeChecker): readonly ts.Type[] {
  return type.aliasTypeArguments ?? (objectFlags(type) & ts.ObjectFlags.Reference
    ? checker.getTypeArguments(type as ts.TypeReference) : []);
}
export function declarationName(node: ts.Node): ts.DeclarationName | undefined {
  // Named declarations share this member; unnamed AST nodes do not.
  return (node as ts.Node & { name?: ts.DeclarationName }).name;
}
