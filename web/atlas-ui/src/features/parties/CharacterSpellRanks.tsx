import { Collapse } from "antd";
import type { SheetSpellRankView } from "../../generated/atlas";
import { ReferenceHelpLink } from "../../shared/records/ReferenceHelp";
import { SpellListRow } from "../../shared/records/SpellListRow";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import type { SheetPresentation } from "./CharacterSheetParts";

export function CharacterSpellRanks({
  ranks,
  links,
  generation,
  open,
  onOpenChange,
}: SheetPresentation & {
  ranks: SheetSpellRankView[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const hasPreparations = ranks.some((rank) => rank.prepared.length > 0);
  const preparedOnly = hasPreparations && !ranks.some((rank) => rank.known.length);

  function renderRanks(list: "known" | "prepared", showSlots: boolean) {
    return (
      <div className="character-magic__ranks">
        {ranks.map((rank, index) => {
          const spells = rank[list];
          const slots = showSlots ? rank.slots : undefined;
          if (!spells.length && slots === undefined) return null;
          return (
            <SpellListRow
              key={`${rank.label}-${index}`}
              label={
                <>
                  {rank.label}
                  {slots !== undefined && (
                    <span className="character-magic__slots">
                      {" "}
                      {slots} {slots === 1 ? "slot" : "slots"}
                    </span>
                  )}
                </>
              }
            >
              {spells.map((spell, i) => {
                const key = spell.ability_key && links.get(spell.ability_key);
                return (
                  <span key={spell.label}>
                    {i > 0 && ", "}
                    <ReferenceHelpLink
                      help={
                        key
                          ? {
                              definition_key: null,
                              destination: {
                                kind: "record",
                                navigation: rootNavigation(key),
                              },
                            }
                          : null
                      }
                      generation={generation}
                    >
                      {spell.label}
                    </ReferenceHelpLink>
                    {spell.count > 1 && ` ×${spell.count}`}
                  </span>
                );
              })}
            </SpellListRow>
          );
        })}
      </div>
    );
  }

  return (
    <>
      {preparedOnly && (
        <h4 className="character-magic__prepared-label">Prepared at export</h4>
      )}
      {renderRanks(preparedOnly ? "prepared" : "known", true)}
      {!preparedOnly && hasPreparations && (
        <Collapse
          className="character-magic__prepared"
          size="small"
          activeKey={open ? ["prepared"] : []}
          onChange={(keys) => onOpenChange(keys.includes("prepared"))}
          items={[
            {
              key: "prepared",
              label: "Prepared at export",
              children: renderRanks("prepared", false),
            },
          ]}
        />
      )}
    </>
  );
}
