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

export const FONT_OPTIONS: { value: ReaderFont; label: string }[] = [
  { value: 'wenkai', label: '霞鹜文楷' },
  { value: 'serif', label: '宋体 / 衬线' },
  { value: 'sans', label: '黑体 / 无衬线' },
  { value: 'default', label: '系统默认' }
];

export const THEME_OPTIONS: { value: ReaderTheme; label: string }[] = [
  { value: 'system', label: '跟随应用' },
  { value: 'paper', label: '纸质白' },
  { value: 'sepia', label: '羊皮纸' },
  { value: 'eye_green', label: '护眼绿' },
  { value: 'night', label: '深色' }
];

export const WIDTH_OPTIONS: { value: ReaderWidth; label: string }[] = [
  { value: 70, label: '窄' },
  { value: 82, label: '适中' },
  { value: 90, label: '宽' },
  { value: 97, label: '超宽' }
];

export const GAP_OPTIONS: { value: ReaderGap; label: string }[] = [
  { value: 'compact', label: '紧凑' },
  { value: 'normal', label: '标准' },
  { value: 'relaxed', label: '宽松' }
];

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
