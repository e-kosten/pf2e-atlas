import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { recordDestination, searchDestination } from "../../app/destinations";
import type { ReactNode } from "react";
import { PhysicalDetails } from "./PhysicalDetails";
import { Button, Typography } from "antd";
import { DEFAULT_SEARCH_STATE, searchStateQueryString } from "../filters/searchState";
import type {
  FactView,
  PhysicalPresentationView,
  PhysicalPriceView,
  RecordSummaryView,
} from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import {
  FactLine,
  FactRow,
  NumberFact,
  factText,
  hasReadingFact,
} from "./presentationFacts";
/** Item facts precede the complete authored rules in every reading host. */
export function PhysicalReference({
  item,
  identity,
  onReference,
  children,
}: {
  item: PhysicalPresentationView;
  identity: RecordSummaryView;
  onReference: RecordReferenceHandler;
  children: ReactNode;
}) {
  const consumable = item.consumable;
  const spell = consumable?.spell.value;
  const baseType = item.base_type.value;
  const baseRoute = baseType?.value.trim()
    ? searchDestination({
        query: searchStateQueryString(
          {
            ...DEFAULT_SEARCH_STATE,
            filter: {
              kind: "compare",
              field: "physical.base_type",
              op: "eq",
              value: baseType.value,
            },
          },
          new URLSearchParams(window.location.search).get("search_profile"),
        ),
      })
    : null;
  const details = consumable
    ? [
        ...(
          [
            ["Maximum HP", consumable.maximum_hp],
            ["Hardness", consumable.hardness],
          ] as const
        ).map(
          ([label, fact]) =>
            ((fact.state === "value" && fact.value !== 0) ||
              ["invalid", "null"].includes(fact.state)) && (
              <FactLine key={label} label={label} fact={fact} />
            ),
        ),
        (consumable.auto_destroy.state === "value" ||
          ["invalid", "null"].includes(consumable.auto_destroy.state)) && (
          <FactLine
            key="consumed"
            label="After final use"
            fact={consumable.auto_destroy}
            format={(value) => (value ? "Consumed" : "Retained")}
          />
        ),
        hasReadingFact(consumable.category) &&
          !identity.traits.some(
            (trait) => trait.value === consumable.category.value,
          ) && <FactLine key="category" label="Category" fact={consumable.category} />,
        consumable.damage.value && (
          <FactRow
            key="damage"
            label={
              consumable.damage.value.kind.value === "healing"
                ? "Healing"
                : consumable.damage.value.kind.value === "damage"
                  ? "Damage"
                  : "Damage or healing"
            }
          >
            {factText(consumable.damage.value.formula)}{" "}
            {factText(consumable.damage.value.damage_type)}
          </FactRow>
        ),
        consumable.damage.state === "invalid" && (
          <FactLine
            key="invalid-damage"
            label="Damage or healing"
            fact={consumable.damage}
          />
        ),
        consumable.spell.state === "invalid" && (
          <FactLine key="invalid-spell" label="Spell" fact={consumable.spell} />
        ),
        ((consumable.maximum_uses.value ?? 0) > 1 ||
          consumable.maximum_uses.state === "invalid" ||
          consumable.source_uses.state === "invalid" ||
          (consumable.source_uses.value !== null &&
            consumable.source_uses.value !== consumable.maximum_uses.value)) && (
          <FactRow key="uses" label="Uses">
            {consumable.source_uses.value !== consumable.maximum_uses.value && (
              <>{factText(consumable.source_uses)} / </>
            )}
            {factText(consumable.maximum_uses)}
          </FactRow>
        ),
        spell && !consumable.spell_presented_inline && (
          <FactRow key="spell" label="Spell">
            <Button
              type="link"
              size="small"
              href={destinationHref(
                recordDestination({
                  recordKey: spell.navigation.record_key,
                  selection: spell.navigation,
                }),
              )}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                onReference(
                  spell.navigation.record_key,
                  event.currentTarget,
                  spell.navigation,
                );
              }}
            >
              {spell.title}
            </Button>
            {spell.navigation.spell_rank && <> · Rank {spell.navigation.spell_rank}</>}
          </FactRow>
        ),
      ].filter(Boolean)
    : [];
  return (
    <div className="physical-reference">
      <div className="physical-reference__facts">
        <div className="record-fact-list">
          <PhysicalPrice
            fact={item.price}
            label={
              (item.price_per.value ?? 1) > 1
                ? `Price per ${item.price_per.value}`
                : "Price"
            }
          />
          {item.price_per.state !== "missing" &&
            (item.price_per.state !== "value" || (item.price_per.value ?? 1) < 1) && (
              <FactLine label="Price per" fact={item.price_per} />
            )}
          <FactLine
            label="Bulk"
            fact={item.bulk}
            format={(value) => (value === 0.1 ? "L" : String(value))}
          />
          <FactLine label="Usage" fact={item.usage} />
          {baseRoute && baseType ? (
            <FactRow label="Base type">
              <Typography.Link
                href={destinationHref(baseRoute)}
                onClick={(event) => {
                  event.stopPropagation();
                  if (!isPrimaryActivation(event)) return;
                  event.preventDefault();
                  navigateDestination(baseRoute);
                }}
              >
                {baseType.label}
              </Typography.Link>
            </FactRow>
          ) : item.base_type.state === "invalid" ? (
            <FactLine
              label="Base type"
              fact={item.base_type}
              format={(value) => value.label}
            />
          ) : null}
          {((item.quantity.state === "value" && item.quantity.value !== 1) ||
            ["invalid", "null"].includes(item.quantity.state)) && (
            <FactLine label="Quantity" fact={item.quantity} />
          )}
          {item.material.value &&
            (item.material.value.kind.value ||
              item.material.value.grade.value ||
              item.material.value.kind.state === "invalid" ||
              item.material.value.grade.state === "invalid") && (
              <FactRow label="Material">
                {factText(item.material.value.kind)}
                {(item.material.value.grade.value ||
                  item.material.value.grade.state === "invalid") && (
                  <> ({factText(item.material.value.grade)} grade)</>
                )}
              </FactRow>
            )}
          {item.material.state === "invalid" && (
            <FactLine label="Material" fact={item.material} />
          )}
          {item.size.state === "value" && item.size.value !== "med" && (
            <FactLine
              label="Size"
              fact={item.size}
              format={(v) =>
                ({
                  tiny: "Tiny",
                  sm: "Small",
                  lg: "Large",
                  huge: "Huge",
                  grg: "Gargantuan",
                })[v] ?? v
              }
            />
          )}
          {item.size.state === "invalid" && <FactLine label="Size" fact={item.size} />}
        </div>
        {item.details && <PhysicalDetails details={item.details} />}
        {details.length > 0 && <div className="record-fact-list">{details}</div>}
      </div>
      {children}
    </div>
  );
}
function PhysicalPrice({
  fact,
  label,
}: {
  fact: FactView<PhysicalPriceView>;
  label: string;
}) {
  if (!hasReadingFact(fact, { required: true })) return null;
  if (fact.state !== "value" || !fact.value)
    return <FactLine label={label} fact={fact} required />;
  const supplied = Object.entries(fact.value).filter(
    ([, value]) => !["missing", "not_applicable"].includes(value.state),
  );
  const hasNonzero = supplied.some(
    ([, value]) => value.state === "value" && value.value !== 0,
  );
  // Suppress only redundant known-zero denominations; retain unavailable facts,
  // adjustments and a genuinely zero price. Never convert currencies or infer free.
  const coins = supplied.filter(
    ([, value]) =>
      !hasNonzero || value.state !== "value" || value.value !== 0 || value.adjustment,
  );
  return (
    <FactRow label={label}>
      {coins.length
        ? coins.map(([unit, value], index) => (
            <span key={unit}>
              {index ? ", " : ""}
              <NumberFact fact={value} /> {unit}
            </span>
          ))
        : "Not listed"}
    </FactRow>
  );
}
