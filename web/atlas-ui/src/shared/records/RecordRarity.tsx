import { Tag, theme } from "antd";
import { sentenceCase } from "../ui/labels";

export function RecordRarity({ rarity }: { rarity: string | null | undefined }) {
  const { token } = theme.useToken();
  if (!rarity?.trim() || rarity === "common") return null;
  const accented = rarity === "rare" || rarity === "unique";
  return (
    <Tag
      aria-label={`Rarity: ${sentenceCase(rarity)}`}
      styles={{
        root: {
          color: accented ? token.colorPrimaryText : token.colorTextSecondary,
          background: accented ? token.colorPrimaryBg : token.colorFillAlter,
          borderColor:
            rarity === "unique"
              ? token.colorPrimaryBorderHover
              : accented
                ? token.colorPrimaryBorder
                : token.colorBorder,
          fontWeight: rarity === "unique" ? 600 : undefined,
        },
      }}
    >
      {sentenceCase(rarity)}
    </Tag>
  );
}
