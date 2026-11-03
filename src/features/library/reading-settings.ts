import i18n from '@/i18n';
import { useAppStore } from '@/store/app';

const STORAGE_KEY = 'precession-reading-settings';

export type ReaderTheme = 'system' | 'paper' | 'sepia' | 'night' | 'eye_green';
export type ReaderFont = 'wenkai' | 'serif' | 'sans' | 'default';
export type ReaderGap = 'compact' | 'normal' | 'relaxed';
export type ReaderWidth = 70 | 82 | 90 | 97;

export type ReadingSettings = {
  fontSize: number;
  lineHeight: number;
  pageMaxWidth: ReaderWidth;
  fontFamily: ReaderFont;
  readerTheme: ReaderTheme;
  paragraphGap: ReaderGap;
};

const DEFAULTS: ReadingSettings = {
  fontSize: 25,
  lineHeight: 1.8,
  pageMaxWidth: 90,
  fontFamily: 'wenkai',
  readerTheme: 'system',
  paragraphGap: 'normal'
};

const FONTS: ReaderFont[] = ['wenkai', 'serif', 'sans', 'default'];
const THEMES: ReaderTheme[] = ['system', 'paper', 'sepia', 'night', 'eye_green'];
const GAPS: ReaderGap[] = ['compact', 'normal', 'relaxed'];
const WIDTHS: ReaderWidth[] = [70, 82, 90, 97];

export type ReaderOption<T> = {
  value: T;
  /** 当前语言下的显示文案。保留这个字段给旧调用方，渲染时优先用 labelKey */
  label: string;
  /** `library` 命名空间下的文案 key */
  labelKey: string;
};

function readerOptions<T>(defs: { value: T; labelKey: string }[]): ReaderOption<T>[] {
  return defs.map((def) => ({
    value: def.value,
    labelKey: def.labelKey,
    get label() {
      return i18n.t(`library:${def.labelKey}`);
    }
  }));
}

export const FONT_OPTIONS: ReaderOption<ReaderFont>[] = readerOptions([
  { value: 'wenkai', labelKey: 'reading.fonts.wenkai' },
  { value: 'serif', labelKey: 'reading.fonts.serif' },
  { value: 'sans', labelKey: 'reading.fonts.sans' },
  { value: 'default', labelKey: 'reading.fonts.default' }
]);

export const THEME_OPTIONS: ReaderOption<ReaderTheme>[] = readerOptions([
  { value: 'system', labelKey: 'reading.themes.system' },
  { value: 'paper', labelKey: 'reading.themes.paper' },
  { value: 'sepia', labelKey: 'reading.themes.sepia' },
  { value: 'eye_green', labelKey: 'reading.themes.eye_green' },
  { value: 'night', labelKey: 'reading.themes.night' }
]);

export const WIDTH_OPTIONS: ReaderOption<ReaderWidth>[] = readerOptions([
  { value: 70, labelKey: 'reading.widths.narrow' },
  { value: 82, labelKey: 'reading.widths.medium' },
  { value: 90, labelKey: 'reading.widths.wide' },
  { value: 97, labelKey: 'reading.widths.extraWide' }
]);

export const GAP_OPTIONS: ReaderOption<ReaderGap>[] = readerOptions([
  { value: 'compact', labelKey: 'reading.gaps.compact' },
  { value: 'normal', labelKey: 'reading.gaps.normal' },
  { value: 'relaxed', labelKey: 'reading.gaps.relaxed' }
]);

const FONT_CSS: Record<ReaderFont, string> = {
  wenkai: 'var(--font-sans)',
  serif: '"Songti SC", "SimSun", "Noto Serif SC", serif',
  sans: '"PingFang SC", "Microsoft YaHei", "Noto Sans SC", sans-serif',
  default: 'inherit'
};

function nearestWidth(raw: number): ReaderWidth {
  return WIDTHS.reduce((best, value) => (Math.abs(value - raw) < Math.abs(best - raw) ? value : best));
}

function fallbackSettings(): ReadingSettings {
  const platform = useAppStore.getState().platform;
  const mobile = platform === 'android' || platform === 'ios';
  return mobile ? { ...DEFAULTS, fontSize: 19 } : { ...DEFAULTS };
}

export function loadReadingSettings(): ReadingSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return fallbackSettings();
    }
    const parsed = JSON.parse(raw) as Partial<ReadingSettings>;
    return {
      fontSize:
        typeof parsed.fontSize === 'number' && parsed.fontSize >= 16 && parsed.fontSize <= 40
          ? parsed.fontSize
          : fallbackSettings().fontSize,
      lineHeight: DEFAULTS.lineHeight,
      pageMaxWidth: typeof parsed.pageMaxWidth === 'number' ? nearestWidth(parsed.pageMaxWidth) : DEFAULTS.pageMaxWidth,
      fontFamily: FONTS.includes(parsed.fontFamily as ReaderFont)
        ? (parsed.fontFamily as ReaderFont)
        : DEFAULTS.fontFamily,
      readerTheme: THEMES.includes(parsed.readerTheme as ReaderTheme)
        ? (parsed.readerTheme as ReaderTheme)
        : DEFAULTS.readerTheme,
      paragraphGap: GAPS.includes(parsed.paragraphGap as ReaderGap)
        ? (parsed.paragraphGap as ReaderGap)
        : DEFAULTS.paragraphGap
    };
  } catch {
    return fallbackSettings();
  }
}

export function saveReadingSettings(settings: ReadingSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

export function fontFamilyCss(font: ReaderFont) {
  return FONT_CSS[font];
}

export function paragraphGapCss(gap: ReaderGap) {
  if (gap === 'compact') {
    return '0.6em';
  }
  if (gap === 'relaxed') {
    return '1.4em';
  }
  return '1em';
}
