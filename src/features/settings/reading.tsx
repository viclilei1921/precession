import { TextAaIcon } from '@phosphor-icons/react';
import type { CSSProperties } from 'react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { SegmentButton } from '@/components/button/segment';
import { Card } from '@/components/card';
import { Slider } from '@/components/slider';
import type {
  ReaderFont,
  ReaderGap,
  ReaderTheme,
  ReaderWidth,
  ReadingSettings
} from '@/features/library/reading-settings';
import {
  fontFamilyCss,
  GAP_OPTIONS,
  loadReadingSettings,
  paragraphGapCss,
  saveReadingSettings,
  THEME_OPTIONS,
  WIDTH_OPTIONS
} from '@/features/library/reading-settings';
import { CardHead } from './card-head';
import styles from './page.module.css';

export function ReadingSection() {
  const { t } = useTranslation('settings');
  const [settings, setSettings] = useState<ReadingSettings>(() => loadReadingSettings());

  function update(partial: Partial<ReadingSettings>) {
    setSettings((current) => {
      const next = { ...current, ...partial };
      saveReadingSettings(next);
      return next;
    });
  }

  const previewStyle = {
    width: `${settings.pageMaxWidth}%`,
    fontFamily: fontFamilyCss(settings.fontFamily),
    fontSize: `${Math.round(settings.fontSize * 0.72)}px`,
    lineHeight: settings.lineHeight,
    '--preview-gap': paragraphGapCss(settings.paragraphGap)
  } as CSSProperties;

  const fontLabel = (font: ReaderFont) => t(`reading.fonts.${font}`);
  const gapLabel = (gap: ReaderGap) => t(`reading.gaps.${gap}`);
  const widthLabel = (width: number) => {
    const map: Record<number, string> = { 70: 'narrow', 82: 'medium', 90: 'wide', 97: 'extraWide' };
    return t(`reading.widths.${map[width] ?? 'medium'}`);
  };
  const themeLabel = (theme: ReaderTheme) => t(`reading.themes.${theme}`);

  return (
    <Card id="reading" className={styles.section}>
      <CardHead
        tone="library"
        icon={TextAaIcon}
        title={t('reading.title')}
        detail={t('reading.detail')}
        extra={`${settings.fontSize} px`}
      />
      <div className={styles.reading}>
        <div className={styles.controls}>
          <div>
            <Slider
              label={t('reading.fontSize')}
              min={16}
              max={40}
              value={settings.fontSize}
              onChange={(fontSize) => update({ fontSize })}
            />
            <div className={styles.scale}>
              <span>{t('reading.fontSizeSmall')}</span>
              <span>{t('reading.fontSizeLarge')}</span>
            </div>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>{t('reading.fontFamily')}</div>
            <fieldset className={styles.segments} aria-label={t('reading.fontFamily')}>
              {(['wenkai', 'serif', 'sans', 'default'] as ReaderFont[]).map((value) => (
                <SegmentButton
                  key={value}
                  selected={settings.fontFamily === value}
                  onClick={() => update({ fontFamily: value })}
                >
                  {fontLabel(value)}
                </SegmentButton>
              ))}
            </fieldset>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>{t('reading.lineHeight')}</div>
            <fieldset className={styles.segments} aria-label={t('reading.lineHeight')}>
              {GAP_OPTIONS.map((item) => (
                <SegmentButton
                  key={item.value}
                  selected={settings.paragraphGap === item.value}
                  onClick={() => update({ paragraphGap: item.value })}
                >
                  {gapLabel(item.value)}
                </SegmentButton>
              ))}
            </fieldset>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>{t('reading.pageWidth')}</div>
            <fieldset className={styles.segments} aria-label={t('reading.pageWidth')}>
              {WIDTH_OPTIONS.map((item) => (
                <SegmentButton
                  key={item.value}
                  selected={settings.pageMaxWidth === item.value}
                  onClick={() => update({ pageMaxWidth: item.value as ReaderWidth })}
                >
                  {widthLabel(item.value)}
                </SegmentButton>
              ))}
            </fieldset>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>
              {t('reading.readerTheme')}
              <span>{t('reading.readerThemeHint')}</span>
            </div>
            <fieldset className={styles.swatches} aria-label={t('reading.readerTheme')}>
              {THEME_OPTIONS.map((item) => (
                <button
                  key={item.value}
                  type="button"
                  className={styles.swatch}
                  data-reader-theme={item.value}
                  data-on={settings.readerTheme === item.value ? 'true' : undefined}
                  aria-pressed={settings.readerTheme === item.value}
                  onClick={() => update({ readerTheme: item.value })}
                >
                  <span className={styles.dot} />
                  {themeLabel(item.value)}
                </button>
              ))}
            </fieldset>
          </div>
        </div>
        <figure className={styles.preview}>
          <figcaption className={styles.previewHead}>
            <span>{t('reading.preview')}</span>
            <span>
              {settings.fontSize} px · {fontLabel(settings.fontFamily)} · {gapLabel(settings.paragraphGap)}
            </span>
          </figcaption>
          <div className={styles.previewStage}>
            <article className={styles.previewPage} data-theme={settings.readerTheme} style={previewStyle}>
              <p className={styles.previewChapter}>{t('reading.previewChapter')}</p>
              <h3 className={styles.previewTitle}>{t('reading.previewTitle')}</h3>
              <p className={styles.previewText}>{t('reading.previewText1')}</p>
              <p className={styles.previewText}>{t('reading.previewText2')}</p>
            </article>
          </div>
          <p className={styles.previewNote}>{t('reading.previewNote')}</p>
        </figure>
      </div>
    </Card>
  );
}
