import { Tag, theme } from "antd";
import { sentenceCase } from "../ui/labels";

export function RecordRarity({ rarity }: { rarity: string | null | undefined }) {
  const { token } = theme.useToken();
  const normalized = rarity?.trim().toLowerCase();
  if (!normalized) return null;
  const role = ["common", "uncommon", "rare", "unique"].includes(normalized)
    ? normalized
    : null;
  return (
    <Tag
      aria-label={`Rarity: ${sentenceCase(normalized)}`}
      styles={{
        root: {
          color: role ? `var(--rarity-${role}-text)` : token.colorTextSecondary,
          background: role ? `var(--rarity-${role}-bg)` : token.colorFillAlter,
          borderColor: role ? `var(--rarity-${role}-border)` : token.colorBorder,
          fontWeight: normalized === "unique" ? 600 : undefined,
        },
      }}
    >
      {sentenceCase(normalized)}
    </Tag>
  );
}
