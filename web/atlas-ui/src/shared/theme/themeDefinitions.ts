export type ResolvedColorScheme = "light" | "dark";
export type StatusRole = "success" | "warning" | "error" | "info";
export type RarityRole = "common" | "uncommon" | "rare" | "unique";

export interface BadgeColors {
  text: string;
  background: string;
  border: string;
}

export interface DarkNeutralRamp {
  shell: string;
  background: string;
  surface: string;
  elevated: string;
  subtle: string;
  separator: string;
  border: string;
}

export interface BrandMarkColors {
  ring: string;
  inner: string;
  cardinal: string;
  shade: string;
  outline: string;
  secondary: string;
  tick: string;
  dieRim: string;
  dieField: string;
  dieFaces: readonly [
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
  ];
}

export const darkNeutralRamp = {
  shell: "#111110",
  background: "#171615",
  surface: "#211F1C",
  elevated: "#28241F",
  subtle: "#302A23",
  separator: "#4A4136",
  border: "#5C5144",
} as const satisfies DarkNeutralRamp;

export interface AtlasPalette {
  primary: string;
  primaryHover: string;
  primaryActive: string;
  onPrimary: string;
  link: string;
  background: string;
  surface: string;
  elevated: string;
  subtle: string;
  text: string;
  muted: string;
  border: string;
  separator: string;
  selection: {
    background: string;
    text: string;
    hover: string;
    border: string;
  };
  contextBar: string;
  shell: BadgeColors;
  brand: {
    text: string;
    mark: BrandMarkColors;
  };
  actionGlyph: string;
  navigation: {
    activeText: string;
    hoverText: string;
    pressedText: string;
    activeBackground: string;
    hoverBackground: string;
  };
  recordKind: BadgeColors;
  filter: {
    include: BadgeColors;
    exclude: BadgeColors;
  };
  rarity: Record<RarityRole, BadgeColors>;
  status: Record<StatusRole, BadgeColors>;
  hp: {
    healthy: string;
    wounded: string;
    critical: string;
    temporary: string;
    missing: string;
  };
  backdrop: string;
  shadow: string;
}

// One product identity, expressed as parchment in daylight and a warmer table
// palette in low light. Semantic roles remain stable between appearances.
export const atlasPalettes: Record<ResolvedColorScheme, AtlasPalette> = {
  light: {
    primary: "#8C333A",
    primaryHover: "#9D4349",
    primaryActive: "#74272E",
    onPrimary: "#FFFFFF",
    link: "#346C63",
    background: "#F6EFE3",
    surface: "#FFFCF4",
    elevated: "#FFFDF8",
    subtle: "#EFE4D4",
    text: "#292621",
    muted: "#685F54",
    border: "#C3B096",
    separator: "#D6C6AD",
    selection: {
      background: "#EADDCB",
      text: "#6E272D",
      hover: "#E1D0B9",
      border: "#8C333A",
    },
    contextBar: "#FFFCF4",
    shell: { background: "#F6EFE3", text: "#292621", border: "#D6C6AD" },
    brand: {
      text: "#8C333A",
      mark: {
        ring: "#8C333A",
        inner: "#FFFCF4",
        cardinal: "#FFFCF4",
        shade: "#D6C6AD",
        outline: "#74272E",
        secondary: "#346C63",
        tick: "#74272E",
        dieRim: "#8C333A",
        dieField: "#F6EFE3",
        dieFaces: [
          "#F6EFE3",
          "#F3EBDD",
          "#F5EDE1",
          "#F4ECDF",
          "#F2E9DB",
          "#DDD0BD",
          "#F1E8D9",
          "#D5C6B1",
          "#F3EADC",
          "#DACCB9",
        ],
      },
    },
    actionGlyph: "#74272E",
    navigation: {
      activeText: "#8C333A",
      hoverText: "#9D4349",
      pressedText: "#74272E",
      activeBackground: "#EADDCB",
      hoverBackground: "#EADDCB",
    },
    recordKind: { text: "#2D625A", background: "#EEF4F0", border: "#9EB8B1" },
    filter: {
      include: { text: "#2D625A", background: "#EEF4F0", border: "#9EB8B1" },
      exclude: { text: "#8C2934", background: "#F6E2E3", border: "#C87A80" },
    },
    rarity: {
      common: { text: "#5D554B", background: "#F1E9DC", border: "#C3B096" },
      uncommon: { text: "#78451F", background: "#F5E2D0", border: "#C98B5C" },
      rare: { text: "#7D2730", background: "#F4DDDE", border: "#C67A80" },
      unique: { text: "#63377D", background: "#ECE0F2", border: "#A987BD" },
    },
    status: {
      success: { text: "#2E6B32", background: "#E7F2E5", border: "#80AA7D" },
      warning: { text: "#855000", background: "#F7E8CF", border: "#D4A55B" },
      error: { text: "#8C2934", background: "#F6E2E3", border: "#C87A80" },
      info: { text: "#315F78", background: "#E2EEF3", border: "#7EA4B6" },
    },
    hp: {
      healthy: "#2E7D32",
      wounded: "#C46A00",
      critical: "#B12A37",
      temporary: "#74877D",
      missing: "#F0E8DC",
    },
    backdrop: "rgba(41, 38, 33, 0.5)",
    shadow: "rgba(72, 52, 33, 0.18)",
  },
  dark: {
    primary: "#7F2F24",
    primaryHover: "#954033",
    primaryActive: "#6E281F",
    onPrimary: "#FFF8EF",
    link: "#D6BA86",
    background: darkNeutralRamp.background,
    surface: darkNeutralRamp.surface,
    elevated: darkNeutralRamp.elevated,
    subtle: darkNeutralRamp.subtle,
    text: "#F3E9D7",
    muted: "#C4B6A1",
    border: darkNeutralRamp.border,
    separator: darkNeutralRamp.separator,
    selection: {
      background: "#382622",
      text: "#F3E9D7",
      hover: "#493029",
      border: "#9B4737",
    },
    contextBar: darkNeutralRamp.surface,
    shell: {
      background: darkNeutralRamp.shell,
      text: "#F3E9D7",
      border: darkNeutralRamp.separator,
    },
    brand: {
      text: "#D6BA86",
      mark: {
        ring: "#D6BA86",
        inner: darkNeutralRamp.background,
        cardinal: "#F3E9D7",
        shade: "#D6BA86",
        outline: "#C4B6A1",
        secondary: "#954033",
        tick: "#D6BA86",
        dieRim: "#7F2F24",
        dieField: "#E4D9C6",
        dieFaces: [
          "#E4D9C6",
          "#DFD3C0",
          "#E3D7C4",
          "#E1D5C2",
          "#DCD1BD",
          "#8F8576",
          "#D7CBB7",
          "#807567",
          "#DBCFBB",
          "#8B8072",
        ],
      },
    },
    actionGlyph: "#D6BA86",
    navigation: {
      activeText: "#D6BA86",
      hoverText: "#D6BA86",
      pressedText: "#D6BA86",
      activeBackground: "#382622",
      hoverBackground: "#382622",
    },
    recordKind: { text: "#A8C7B0", background: "#29312B", border: "#5C7662" },
    filter: {
      include: { text: "#A8C7B0", background: "#29312B", border: "#5C7662" },
      exclude: { text: "#EE9A89", background: "#422622", border: "#B95540" },
    },
    rarity: {
      common: { text: "#C4B6A1", background: "#302E2A", border: "#5C5144" },
      uncommon: { text: "#E1B27A", background: "#3D2E21", border: "#865722" },
      rare: { text: "#E8A093", background: "#422622", border: "#9B4737" },
      unique: { text: "#D2AFE8", background: "#35283D", border: "#684582" },
    },
    status: {
      success: { text: "#A8C99F", background: "#273126", border: "#668F59" },
      warning: { text: "#E4B66D", background: "#3B2E1E", border: "#C98224" },
      error: { text: "#EE9A89", background: "#422622", border: "#B95540" },
      info: { text: "#9FC3D5", background: "#24343C", border: "#5D879A" },
    },
    hp: {
      healthy: "#668F59",
      wounded: "#C98224",
      critical: "#D8583F",
      temporary: "#A39784",
      missing: "#38342F",
    },
    backdrop: "rgba(0, 0, 0, 0.72)",
    shadow: "rgba(0, 0, 0, 0.5)",
  },
};

export function resolveAtlasPalette(scheme: ResolvedColorScheme): AtlasPalette {
  return atlasPalettes[scheme];
}
