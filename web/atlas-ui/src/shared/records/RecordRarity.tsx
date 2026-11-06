import { Tag, theme } from "antd";

export function RecordRarity({
  rarity,
}: {
  rarity: import("../../generated/atlas").ReferenceValueView | null | undefined;
}) {
  const { token } = theme.useToken();
  const normalized = rarity?.value.trim().toLowerCase();
  if (!normalized) return null;
  const role = ["common", "uncommon", "rare", "unique"].includes(normalized)
    ? normalized
    : null;
  return (
    <Tag
      aria-label={`Rarity: ${rarity?.label}`}
      styles={{
        root: {
          color: role ? `var(--rarity-${role}-text)` : token.colorTextSecondary,
          background: role ? `var(--rarity-${role}-bg)` : token.colorFillAlter,
          borderColor: role ? `var(--rarity-${role}-border)` : token.colorBorder,
          fontWeight: normalized === "unique" ? 600 : undefined,
        },
      }}
    >
      {rarity?.label}
    </Tag>
  );
}
