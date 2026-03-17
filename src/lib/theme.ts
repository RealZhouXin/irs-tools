export const DEFAULT_THEME_COLOR = "#52525b";

const HEX_COLOR_PATTERN = /^#[0-9a-fA-F]{6}$/;

export function normalizeThemeColor(value: string | null | undefined): string {
  const candidate = value?.trim() ?? "";
  return HEX_COLOR_PATTERN.test(candidate)
    ? candidate.toLowerCase()
    : DEFAULT_THEME_COLOR;
}

export function applyThemeColor(value: string | null | undefined): string {
  const themeColor = normalizeThemeColor(value);
  if (typeof document !== "undefined") {
    document.documentElement.style.setProperty("--brand-base", themeColor);
  }
  return themeColor;
}
