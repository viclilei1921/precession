import { TextAaIcon } from '@phosphor-icons/react';
import type { CSSProperties } from 'react';
import { useState } from 'react';
import { SegmentButton } from '@/components/button/segment';
import { Card } from '@/components/card';
import { Slider } from '@/components/slider';
import type { ReaderFont, ReadingSettings } from '@/features/library/reading-settings';
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

const FONT_LABEL: Record<ReaderFont, string> = {
  wenkai: '文楷',
  serif: '宋体',
  sans: '黑体',
  default: '默认'
};

const PREVIEW_META: Record<string, string> = {
  compact: '紧凑',
  normal: '标准',
  relaxed: '宽松'
};

export function ReadingSection() {
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

  return (
    <Card id="reading" className={styles.section}>
      <CardHead
        tone="library"
        icon={TextAaIcon}
        title="阅读"
        detail="书库阅读器的默认排版"
        extra={`${settings.fontSize} px`}
      />
      <div className={styles.reading}>
        <div className={styles.controls}>
          <div>
            <Slider
              label="字号"
              min={16}
              max={40}
              value={settings.fontSize}
              onChange={(fontSize) => update({ fontSize })}
            />
            <div className={styles.scale}>
              <span>小</span>
              <span>大</span>
            </div>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>字体</div>
            <fieldset className={styles.segments} aria-label="阅读字体">
              {(Object.keys(FONT_LABEL) as ReaderFont[]).map((value) => (
                <SegmentButton
                  key={value}
                  selected={settings.fontFamily === value}
                  onClick={() => update({ fontFamily: value })}
                >
                  {FONT_LABEL[value]}
                </SegmentButton>
              ))}
            </fieldset>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>行距</div>
            <fieldset className={styles.segments} aria-label="阅读行距">
              {GAP_OPTIONS.map((item) => (
                <SegmentButton
                  key={item.value}
                  selected={settings.paragraphGap === item.value}
                  onClick={() => update({ paragraphGap: item.value })}
                >
                  {item.label}
                </SegmentButton>
              ))}
            </fieldset>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>页宽</div>
            <fieldset className={styles.segments} aria-label="阅读页宽">
              {WIDTH_OPTIONS.map((item) => (
                <SegmentButton
                  key={item.value}
                  selected={settings.pageMaxWidth === item.value}
                  onClick={() => update({ pageMaxWidth: item.value })}
                >
                  {item.label}
                </SegmentButton>
              ))}
            </fieldset>
          </div>
          <div className={styles.control}>
            <div className={styles.controlHead}>
              阅读主题
              <span>纸面配色</span>
            </div>
            <fieldset className={styles.swatches} aria-label="阅读主题">
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
                  {item.label}
                </button>
              ))}
            </fieldset>
          </div>
        </div>
        <figure className={styles.preview}>
          <figcaption className={styles.previewHead}>
            <span>阅读预览</span>
            <span>
              {settings.fontSize} px · {FONT_LABEL[settings.fontFamily]} · {PREVIEW_META[settings.paragraphGap]}
            </span>
          </figcaption>
          <div className={styles.previewStage}>
            <article className={styles.previewPage} data-theme={settings.readerTheme} style={previewStyle}>
              <p className={styles.previewChapter}>第一章</p>
              <h3 className={styles.previewTitle}>山中一日</h3>
              <p className={styles.previewText}>
                清晨的雾还没有散，山路在松针下面发出细碎的响声。走到半山，回头看见整个镇子浮在云里。
              </p>
              <p className={styles.previewText}>风从背后推了一把，树叶翻过来，露出银白的背面。</p>
            </article>
          </div>
          <p className={styles.previewNote}>预览按比例缩小，实际排版以阅读器为准。</p>
        </figure>
      </div>
    </Card>
  );
}
