import { ReferenceHelpLink } from "./ReferenceHelp";
import { theme } from "antd";
import { DisplayChip } from "../ui/DisplayChip";

export function RecordRarity({
  rarity,
  linked = true,
}: {
  linked?: boolean;
  rarity: import("../../generated/atlas").ReferenceValueView | null | undefined;
}) {
  const { token } = theme.useToken();
  const normalized = rarity?.value.trim().toLowerCase();
  if (!rarity?.label.trim()) return null;
  const role =
    normalized && ["common", "uncommon", "rare", "unique"].includes(normalized)
      ? normalized
      : null;
  return (
    <DisplayChip
      aria-label={`Rarity: ${rarity?.label}`}
      styles={{
        root: {
          color: role ? `var(--rarity-${role}-text)` : token.colorTextSecondary,
          background: role ? `var(--rarity-${role}-bg)` : token.colorFillAlter,
          borderColor: role ? `var(--rarity-${role}-border)` : token.colorBorder,
        },
      }}
    >
      <ReferenceHelpLink help={linked ? rarity?.help : null}>
        {rarity?.label}
      </ReferenceHelpLink>
    </DisplayChip>
  );
}
