import assert from 'node:assert/strict';
import test from 'node:test';
import type { TraitCatalog } from '../src/contracts.js';
import {traitLabelKeys, generateTraitLabelModule} from '../src/generation/trait-labels.js';
const catalog=(entries:{identifier:string;labelKey:string|null}[]):TraitCatalog=>({complete:true,catalogs:[{name:'itemTraits',namespace:'trait',complete:true,entries}] as TraitCatalog['catalogs'],descriptions:[],diagnostics:[]});
test('key-only metadata shares repeated keys, sorts deterministically and never emits labels/descriptions',()=>{const keys=traitLabelKeys(catalog([{identifier:'undead',labelKey:'PF2E.TraitUndead'},{identifier:'acid',labelKey:'PF2E.TraitAcid'},{identifier:'undead',labelKey:'PF2E.TraitUndead'},{identifier:'unknown',labelKey:null}]));assert.deepEqual(keys,{acid:'PF2E.TraitAcid',undead:'PF2E.TraitUndead'});const rust=generateTraitLabelModule(keys,'// generated');assert.match(rust,/"acid" => Some\("PF2E.TraitAcid"\)/);assert.match(rust,/_ => None/);assert(!rust.includes('description'));});
test('conflicting localization key identity and incomplete extraction fail explicitly',()=>{assert.throws(()=>traitLabelKeys(catalog([{identifier:'acid',labelKey:'one'},{identifier:'acid',labelKey:'two'}])),/Conflicting/);const incomplete=catalog([]);incomplete.complete=false;assert.throws(()=>traitLabelKeys(incomplete),/incomplete/);});
