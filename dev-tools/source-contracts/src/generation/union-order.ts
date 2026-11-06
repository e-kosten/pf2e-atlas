import { createHash } from 'node:crypto';
import type { GraphNode } from '../contracts.js';

interface MemberOrder { key: string; ordinal: number; serdeVariant?: string }
export interface UnionOrderRegistry {
  format: 'atlas-source-union-order/v1';
  unions: Record<string, { name?: string; members: MemberOrder[] }>;
}
interface CollectionState { nullable: boolean; onlyNullish: boolean }
const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0;
const hash = (value: unknown) => createHash('sha256').update(JSON.stringify(value)).digest('hex');


export function assertUnionOrderRegistry(value: unknown): asserts value is UnionOrderRegistry {
  const registry = value as UnionOrderRegistry;
  if (!registry || typeof registry !== 'object' || registry.format !== 'atlas-source-union-order/v1' || !registry.unions
    || typeof registry.unions !== 'object' || Array.isArray(registry.unions))
    throw new Error('Invalid union order registry');
  for (const [key, entry] of Object.entries(registry.unions)) {
    if (!/^[a-f0-9]{64}$/.test(key) || !Array.isArray(entry?.members)
      || entry.name !== undefined && typeof entry.name !== 'string') throw new Error('Invalid union order entry');
    const keys = new Set<string>(), ordinals = new Set<number>(), variants = new Set<string>();
    for (const member of entry.members) {
      if (!/^[a-f0-9]{64}$/.test(member.key) || !Number.isSafeInteger(member.ordinal) || member.ordinal < 1
        || keys.has(member.key) || ordinals.has(member.ordinal)
        || member.serdeVariant !== undefined && (!/^[A-Z][A-Za-z0-9]*$/.test(member.serdeVariant) || variants.has(member.serdeVariant)))
        throw new Error(`Invalid or duplicate union member policy: ${key}`);
      keys.add(member.key); ordinals.add(member.ordinal);
      if (member.serdeVariant) variants.add(member.serdeVariant);
    }
  }
}

/** Persisted names belong to Atlas, rather than compiler-local graph IDs. */
export class UnionOrder {
  private readonly nodes = new Map<string, GraphNode>();
  private readonly components = new Map<string, object>();
  private readonly fingerprints = new Map<string, string>();
  private readonly entries = new Map<string, UnionOrderRegistry['unions'][string]>();
  private readonly keys = new Map<string, string>();
  private readonly stableUnions = new Set<string>();
  private work = 0;
  private readonly variantsUsed = new Set<MemberOrder>();

  constructor(
    refs: string[],
    private readonly normalized: (ref: string) => GraphNode,
    private readonly collectionState: (ref: string, indexed?: boolean) => CollectionState,
    private readonly openTraitArrays: Set<string>,
    private readonly reviewed?: UnionOrderRegistry,
    private readonly workLimit = 1_000_000,
  ) {
    if (!Number.isSafeInteger(workLimit) || workLimit < 1) throw new Error('Invalid union fingerprint work limit');
    const add = (ref: string) => {
      const node = normalized(ref);
      if (this.nodes.has(node.id)) return;
      this.nodes.set(node.id, node);
      this.references(node).forEach(add);
    };
    refs.forEach(add);
    this.findComponents();
    if (reviewed !== undefined) assertUnionOrderRegistry(reviewed);
    for (const node of this.nodes.values()) {
      if (node.kind !== 'union') continue;
      // Primitive/literal IDs and atomic ordering are already explicit semantic
      // constructions. Keep vocabulary contents in the fresh declaration graph.
      const requiresReview = node.members.some(ref => {
        const member = normalized(ref);
        if (ref !== member.id || member.name || member.typeArguments?.length) return true;
        if (member.kind === 'primitive') return member.id !== `primitive:${member.value}`;
        if (member.kind === 'literal') return member.id !== `literal:${typeof member.value}:${typeof member.value === 'string' ? JSON.stringify(member.value) : member.value}`;
        if (member.kind === 'open') return !['any', 'unknown', 'object'].includes(member.domain) || member.id !== `primitive:${member.domain}`;
        return true;
      });
      if (!requiresReview) { this.stableUnions.add(node.id); continue; }
      const key = this.fingerprint(node.id);
      const members = node.members.map((ref, index) => ({ key: this.fingerprint(ref), ordinal: index + 1 }));
      if (new Set(members.map(member => member.key)).size !== members.length)
        throw new Error(`Ambiguous semantic union members: ${node.id}`);
      const expected = reviewed?.unions[key];
      if (reviewed !== undefined && !expected) throw new Error(`Unreviewed union ${node.name ?? node.id}: ${key}; members ${JSON.stringify(members)}`);
      if (expected && (expected.members.length !== members.length
        || members.some(member => !expected.members.some(old => old.key === member.key))))
        throw new Error(`Reviewed union member drift: ${key}`);
      const entry = expected ? structuredClone(expected) : { ...(node.name ? { name: node.name } : {}), members };
      const prior = this.entries.get(key);
      if (reviewed === undefined && prior && JSON.stringify(prior.members) !== JSON.stringify(members))
        throw new Error(`Equivalent unions have conflicting predecessor order: ${key}`);
      this.entries.set(key, prior ?? entry);
      this.keys.set(node.id, key);
      // Mutate only the memoized normalized view, never declaration/collection evidence.
      node.members.sort((a, b) => this.ordinal(node, a) - this.ordinal(node, b));
    }
    if (reviewed !== undefined && Object.keys(reviewed.unions).some(key => !this.entries.has(key)))
      throw new Error('Reviewed union registry contains obsolete entries');
  }

  ordinal(node: GraphNode, member: string): number {
    const key = this.keys.get(node.id);
    if (key === undefined && this.stableUnions.has(node.id) && node.kind === 'union' && node.members.includes(member)) return node.members.indexOf(member) + 1;
    const entry = this.entries.get(key!);
    const found = entry?.members.find(value => value.key === this.fingerprint(member));
    if (!found) throw new Error(`Missing reviewed union ordinal: ${node.id}`);
    return found.ordinal;
  }

  variant(node: GraphNode, member: string, value: string): string {
    const key = this.keys.get(node.id);
    if (key === undefined) {
      if (this.stableUnions.has(node.id) && node.kind === 'union' && node.members.includes(member)) return value;
      throw new Error(`Missing reviewed union variant: ${node.id}`);
    }
    const entry = this.entries.get(key)!;
    const arm = entry.members.find(arm => arm.key === this.fingerprint(member))!;
    if (this.reviewed !== undefined && arm.serdeVariant !== value)
      throw new Error(`Persisted union variant drift: ${entry.name ?? node.id}: ${arm.serdeVariant ?? '<unreviewed>'} -> ${value}`);
    if (arm.serdeVariant !== undefined && arm.serdeVariant !== value)
      throw new Error(`Equivalent union has conflicting persisted variant names: ${node.id}`);
    arm.serdeVariant = value;
    this.variantsUsed.add(arm);
    return value;
  }

  finish() {
    if (this.reviewed !== undefined && [...this.entries.values()].some(entry => entry.members.some(member => member.serdeVariant !== undefined && !this.variantsUsed.has(member))))
      throw new Error('Reviewed persisted union variant was not emitted');
  }

  registry(): UnionOrderRegistry {
    return { format: 'atlas-source-union-order/v1', unions: Object.fromEntries(
      [...this.entries].sort(([a], [b]) => compare(a, b)).map(([key, value]) =>
        [key, { ...value, members: [...value.members].sort((a, b) => a.ordinal - b.ordinal) }]),
    ) };
  }


  private references(node: GraphNode): string[] {
    const collection = (ref: string, indexed = false) => this.collectionState(ref, indexed).onlyNullish ? [] : [ref];
    if (node.kind === 'array') return this.openTraitArrays.has(node.id) ? ['primitive:string'] : collection(node.element);
    if (node.kind === 'tuple') return node.elements.flatMap(element => collection(element.ref));
    if (node.kind === 'union') return node.members;
    if (node.kind === 'template') return node.parameters;
    if (node.kind === 'object' || node.kind === 'intersection' || node.kind === 'open')
      return [...(node.fields ?? []).filter(field => !field.forbidden).map(field => field.ref),
        ...(node.indexSignatures ?? []).flatMap(index => collection(index.value, true))];
    return [];
  }

  private findComponents() {
    let sequence = 0;
    const index = new Map<string, number>(), low = new Map<string, number>(), stack: string[] = [], active = new Set<string>();
    const visit = (id: string) => {
      index.set(id, sequence); low.set(id, sequence++); stack.push(id); active.add(id);
      for (const ref of this.references(this.nodes.get(id)!)) {
        const child = this.normalized(ref).id;
        if (!index.has(child)) { visit(child); low.set(id, Math.min(low.get(id)!, low.get(child)!)); }
        else if (active.has(child)) low.set(id, Math.min(low.get(id)!, index.get(child)!));
      }
      if (index.get(id) === low.get(id)) {
        const component = {}; let child: string;
        do { child = stack.pop()!; active.delete(child); this.components.set(child, component); } while (child !== id);
      }
    };
    for (const id of this.nodes.keys()) if (!index.has(id)) visit(id);
  }

  private fingerprint(ref: string): string {
    const id = this.normalized(ref).id;
    if (!this.fingerprints.has(id)) this.fingerprints.set(id, this.walk(id, []));
    return this.fingerprints.get(id)!;
  }

  private walk(id: string, ancestors: string[]): string {
    if (++this.work > this.workLimit) throw new Error('Semantic union fingerprint work limit exceeded');
    const previous = ancestors.indexOf(id);
    if (previous >= 0) return hash(['back-reference', ancestors.length - previous]);
    const node = this.nodes.get(id)!, next = [...ancestors, id];
    // An internal SCC hash depends on its ancestor stack. Memoize only across components.
    const child = (ref: string): string => {
      const target = this.normalized(ref).id;
      return this.components.get(id) === this.components.get(target) ? this.walk(target, next) : this.fingerprint(target);
    };
    const collection = (ref: string, indexed = false) => {
      const state = this.collectionState(ref, indexed);
      return state.onlyNullish ? ['nullish', state.nullable] : [state.nullable, child(ref)];
    };
    const fields = () => [...('fields' in node ? node.fields ?? [] : [])].sort((a, b) => compare(a.name, b.name))
      .map(field => [field.name, field.optional, field.nullable, field.undefinedAllowed, field.forbidden,
        ...(field.forbidden ? [] : [child(field.ref)])]);
    const indexed = () => ('indexSignatures' in node ? node.indexSignatures ?? [] : [])
      .map(index => [index.key, index.readonly, collection(index.value, true)])
      .sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
    let shape: unknown[];
    switch (node.kind) {
      case 'primitive': case 'literal': shape = [node.value]; break;
      case 'union': shape = [node.members.map(child).sort(compare)]; break;
      case 'object': case 'intersection': shape = [fields(), indexed(), node.kind === 'intersection' && !!node.impossible]; break;
      case 'array': shape = [node.readonly, this.openTraitArrays.has(node.id) ? [false, child('primitive:string')] : collection(node.element)]; break;
      case 'tuple': shape = [node.readonly, node.elements.map(element => [element.optional, element.rest, collection(element.ref)])]; break;
      case 'template': shape = [node.text, node.parameters.map(child)]; break;
      case 'open': shape = [node.domain, fields(), indexed()]; break;
      default: throw new Error(`Unsupported semantic union shape: ${node.id}`);
    }
    return hash([node.kind, node.name ?? null, ...shape]);
  }
}
