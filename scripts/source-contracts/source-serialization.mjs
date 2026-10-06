import ts from 'typescript';

// Upstream source declarations include runtime types. Keep their JSON
// projections bounded to the declarations whose serialization was inspected.
function declarationIs(type, file, name, relative) {
  const symbol = type.aliasSymbol ?? type.getSymbol();
  return symbol?.getName() === name && symbol.getDeclarations()?.some((declaration) =>
    relative(declaration.getSourceFile().fileName) === file);
}

export function predicateSourceArray(type, checker, relative) {
  if (!declarationIs(type, 'src/module/system/predication.ts', 'Predicate', relative)) return null;
  const method = checker.getPropertyOfType(type, 'toObject');
  const declaration = method?.valueDeclaration ?? method?.declarations?.[0];
  const signatures = declaration && checker.getSignaturesOfType(
    checker.getTypeOfSymbolAtLocation(method, declaration), ts.SignatureKind.Call);
  const raw = signatures?.length === 1 && signatures[0].parameters.length === 0
    ? checker.getReturnTypeOfSignature(signatures[0]) : null;
  const baseArray = checker.getBaseTypes(type)?.find((base) => checker.isArrayType(base));
  // A custom toJSON hook would take precedence over ordinary array serialization.
  if (!baseArray || !raw || !checker.isArrayType(raw)
    || checker.getTypeArguments(baseArray)[0] !== checker.getTypeArguments(raw)[0]
    || checker.getPropertyOfType(type, 'toJSON')) {
    return { error: 'Predicate must extend Array and expose one zero-argument toObject returning the same element type, without a toJSON override' };
  }
  return { raw, method };
}

export function modifierCallbackProjection(owner, property, resolved, checker, relative) {
  if (!declarationIs(owner, 'src/module/actor/modifiers.ts', 'ModifierAdjustment', relative)
    || !['test', 'getNewValue', 'getDamageType'].includes(property.getName())) return null;
  const alternatives = resolved.isUnion() ? resolved.types : [resolved];
  const present = alternatives.filter((part) => !(part.flags & ts.TypeFlags.Undefined));
  if (!present.length || present.some((part) => !checker.getSignaturesOfType(part, ts.SignatureKind.Call).length)) {
    return { error: `ModifierAdjustment.${property.getName()} no longer contains only a function (and optional undefined)` };
  }
  // JSON.stringify omits function-valued object members. Keeping a forbidden
  // field records the declaration and prevents later consumers accepting a JSON
  // string/object/null under that name as though it were a serialized callback.
  return { declaredOptional: !!(property.flags & ts.SymbolFlags.Optional) };
}

export function choicePredicateInputs(owner, property, resolved, checker, relative) {
  if (!declarationIs(owner, 'src/module/apps/pick-a-thing-prompt.ts', 'PickableThing', relative)
    || property.getName() !== 'predicate') return null;
  const alternatives = resolved.isUnion() ? resolved.types : [resolved];
  const present = alternatives.filter((part) => !(part.flags & ts.TypeFlags.Undefined));
  if (present.length !== 1) return { error: 'PickableThing.predicate must refer to Predicate (and optional undefined)' };
  const predicate = present[0];
  const array = predicateSourceArray(predicate, checker, relative);
  if (!array || array.error) return { error: array?.error ?? 'PickableThing.predicate no longer refers to the inspected Predicate declaration' };
  const symbol = predicate.getSymbol();
  const declaration = symbol.valueDeclaration ?? symbol.declarations?.[0];
  const constructors = checker.getSignaturesOfType(
    checker.getTypeOfSymbolAtLocation(symbol, declaration), ts.SignatureKind.Construct);
  const parameter = constructors.length === 1 && constructors[0].parameters.length === 1
    ? constructors[0].parameters[0] : null;
  if (!parameter?.valueDeclaration?.dotDotDotToken) {
    return { error: 'Predicate constructor input must be one rest parameter' };
  }
  const argumentsType = checker.getTypeOfSymbolAtLocation(parameter, parameter.valueDeclaration);
  const variants = argumentsType.isUnion() ? argumentsType.types : [argumentsType];
  const inputs = [];
  for (const variant of variants) {
    const elements = checker.getTypeArguments(variant);
    if (checker.isArrayType(variant) || (checker.isTupleType(variant) && elements.length === 1
      && variant.target.elementFlags[0] === ts.ElementFlags.Required)) inputs.push(elements[0]);
    else return { error: 'Predicate constructor rest input contains an unsupported argument shape' };
  }
  // ChoiceSet passes c.predicate as one constructor argument. Derive the
  // accepted statement-or-array inputs from its declaration, rather than
  // broadening every predicate field based on three corpus occurrences.
  return { inputs, predicate, constructor: parameter };
}
