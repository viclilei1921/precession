import { CheckIcon, PaletteIcon } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';
import { Card } from '@/components/card';
import type { ThemePreference } from '@/features/settings/preferences';
import { Switch } from '@/features/settings/switch';
import { useAppStore } from '@/store/app';
import { CardHead } from './card-head';
import styles from './page.module.css';
import { SettingRow } from './row';

type AppearanceSectionProps = {
  onNotice: (message: string) => void;
};

export function AppearanceSection({ onNotice }: AppearanceSectionProps) {
  const { t } = useTranslation('settings');
  const mobile = useAppStore((state) => state.isMobile());
  const preference = useAppStore((state) => state.themePreference);
  const theme = useAppStore((state) => state.theme);
  const collapsed = useAppStore((state) => state.sidebarCollapsed);
  const reduceMotion = useAppStore((state) => state.reduceMotion);
  const imageConvert = useAppStore((state) => state.imageConvert);
  const setThemePreference = useAppStore((state) => state.setThemePreference);
  const setSidebarCollapsed = useAppStore((state) => state.setSidebarCollapsed);
  const setReduceMotion = useAppStore((state) => state.setReduceMotion);
  const setImageConvert = useAppStore((state) => state.setImageConvert);

  const THEMES: { value: ThemePreference; label: string; hint: string; shot: 'light' | 'dark' | 'system' }[] = [
    { value: 'light', label: t('appearance.theme.light.label'), hint: t('appearance.theme.light.hint'), shot: 'light' },
    { value: 'dark', label: t('appearance.theme.dark.label'), hint: t('appearance.theme.dark.hint'), shot: 'dark' },
    {
      value: 'system',
      label: t('appearance.theme.system.label'),
      hint: t('appearance.theme.system.hint'),
      shot: 'system'
    }
  ];

  const tone = theme === 'dark' ? t('appearance.currentDark') : t('appearance.currentLight');
  const extra =
    preference === 'system' ? t('appearance.extraSystem', { tone }) : t('appearance.extraCurrent', { tone });

  function choose(next: ThemePreference) {
    setThemePreference(next);
    onNotice(
      next === 'system'
        ? t('appearance.noticeSystem')
        : t('appearance.noticeSwitched', {
            theme: next === 'dark' ? t('appearance.theme.dark.label') : t('appearance.theme.light.label')
          })
    );
  }

  return (
    <Card id="appearance" className={styles.section}>
      <CardHead
        tone="system"
        icon={PaletteIcon}
        title={t('appearance.title')}
        detail={t('appearance.detail')}
        extra={extra}
      />
      <fieldset className={styles.themes} aria-label={t('appearance.title')}>
        {THEMES.map((item) => (
          <button
            key={item.value}
            type="button"
            className={styles.theme}
            data-on={preference === item.value ? 'true' : undefined}
            aria-pressed={preference === item.value}
            onClick={() => choose(item.value)}
          >
            <span className={styles.shot} data-shot={item.shot} aria-hidden="true">
              <span className={styles.shotBar}>
                <span />
                <span />
                <span />
              </span>
              <span className={styles.shotSide} />
              <span className={styles.shotMain}>
                <span className={styles.shotHeading} />
                <span className={styles.shotLine} />
                <span className={styles.shotLine} data-short="true" />
                <span className={styles.shotPill} />
              </span>
              <span className={styles.shotCheck}>
                <CheckIcon weight="regular" />
              </span>
            </span>
            <span className={styles.themeCopy}>
              <strong>{item.label}</strong>
              <small>{item.hint}</small>
            </span>
          </button>
        ))}
      </fieldset>
      <div className={styles.rows}>
        {mobile ? null : (
          <SettingRow title={t('appearance.sidebarRow.title')} hint={t('appearance.sidebarRow.hint')}>
            <Switch
              label={t('appearance.sidebarRow.title')}
              checked={!collapsed}
              onClick={() => {
                const nextCollapsed = !collapsed;
                setSidebarCollapsed(nextCollapsed);
                onNotice(
                  nextCollapsed ? t('appearance.sidebarNoticeCollapsed') : t('appearance.sidebarNoticeExpanded')
                );
              }}
            />
          </SettingRow>
        )}
        {mobile ? null : (
          <SettingRow title={t('appearance.imageConvertRow.title')} hint={t('appearance.imageConvertRow.hint')}>
            <Switch
              label={t('appearance.imageConvertRow.title')}
              checked={imageConvert}
              onClick={() => {
                const next = !imageConvert;
                setImageConvert(next);
                onNotice(next ? t('appearance.imageConvertNoticeOn') : t('appearance.imageConvertNoticeOff'));
              }}
            />
          </SettingRow>
        )}
        <SettingRow title={t('appearance.motionRow.title')} hint={t('appearance.motionRow.hint')}>
          <Switch
            label={t('appearance.motionRow.title')}
            checked={reduceMotion}
            onClick={() => {
              const next = !reduceMotion;
              setReduceMotion(next);
              onNotice(next ? t('appearance.motionNoticeOn') : t('appearance.motionNoticeOff'));
            }}
          />
        </SettingRow>
      </div>
    </Card>
  );
}
