import { theme } from "antd";
import { antDesignTheme } from "./atlasTheme";

function luminance(color: string) {
  const channels = color.startsWith("#")
    ? color
        .slice(1)
        .match(/../g)!
        .map((value) => parseInt(value, 16))
    : color
        .match(/[\d.]+/g)!
        .slice(0, 3)
        .map(Number);
  const linear = channels.map((value) => {
    const channel = value / 255;
    return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
}
function contrast(first: string, second: string) {
  const values = [luminance(first), luminance(second)].sort((a, b) => b - a);
  return (values[0] + 0.05) / (values[1] + 0.05);
}

it.each(["light", "dark"] as const)(
  "keeps primary text and links readable in %s",
  (scheme) => {
    const token = theme.getDesignToken(antDesignTheme(scheme));
    for (const background of [
      token.colorPrimary,
      token.colorPrimaryHover,
      token.colorPrimaryActive,
    ]) {
      expect(contrast(token.colorTextLightSolid, background)).toBeGreaterThanOrEqual(
        4.5,
      );
    }
    for (const foreground of [
      token.colorLink,
      token.colorLinkHover,
      token.colorLinkActive,
      token.colorTextDescription,
    ]) {
      expect(contrast(foreground, token.colorBgContainer)).toBeGreaterThanOrEqual(4.5);
    }
  },
);
