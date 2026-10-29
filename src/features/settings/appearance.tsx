import { CheckIcon, PaletteIcon } from '@phosphor-icons/react';
import { Card } from '@/components/card';
import type { ThemePreference } from '@/features/settings/preferences';
import { Switch } from '@/features/settings/switch';
import { useAppStore } from '@/store/app';
import { CardHead } from './card-head';
import styles from './page.module.css';
import { SettingRow } from './row';

const THEMES: { value: ThemePreference; label: string; hint: string; shot: 'light' | 'dark' | 'system' }[] = [
  { value: 'light', label: '浅色', hint: '明亮通透', shot: 'light' },
  { value: 'dark', label: '深色', hint: '夜间阅读', shot: 'dark' },
  { value: 'system', label: '跟随系统', hint: '随系统切换', shot: 'system' }
];

type AppearanceSectionProps = {
  onNotice: (message: string) => void;
};

export function AppearanceSection({ onNotice }: AppearanceSectionProps) {
  const mobile = useAppStore((state) => state.isMobile());
  const preference = useAppStore((state) => state.themePreference);
  const theme = useAppStore((state) => state.theme);
  const collapsed = useAppStore((state) => state.sidebarCollapsed);
  const reduceMotion = useAppStore((state) => state.reduceMotion);
  const setThemePreference = useAppStore((state) => state.setThemePreference);
  const setSidebarCollapsed = useAppStore((state) => state.setSidebarCollapsed);
  const setReduceMotion = useAppStore((state) => state.setReduceMotion);

  const tone = theme === 'dark' ? '深色' : '浅色';
  const extra = preference === 'system' ? `跟随系统 · ${tone}` : `当前 ${tone}`;

  function choose(next: ThemePreference) {
    setThemePreference(next);
    onNotice(next === 'system' ? '主题将跟随系统外观' : `已切换到${next === 'dark' ? '深色' : '浅色'}主题`);
  }

  return (
    <Card id="appearance" className={styles.section}>
      <CardHead tone="system" icon={PaletteIcon} title="外观" detail="主题、侧栏与界面动效" extra={extra} />
      <fieldset className={styles.themes} aria-label="主题">
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
          <SettingRow title="展开侧栏" hint="关闭后导航收起，内容区更宽。">
            <Switch
              label="展开侧栏"
              checked={!collapsed}
              onClick={() => {
                const nextCollapsed = !collapsed;
                setSidebarCollapsed(nextCollapsed);
                onNotice(nextCollapsed ? '侧栏已收起' : '侧栏已展开');
              }}
            />
          </SettingRow>
        )}
        <SettingRow title="减少动效" hint="关闭过渡和渐隐，页面切换更直接。">
          <Switch
            label="减少动效"
            checked={reduceMotion}
            onClick={() => {
              const next = !reduceMotion;
              setReduceMotion(next);
              onNotice(next ? '已减少界面动效' : '已恢复界面动效');
            }}
          />
        </SettingRow>
      </div>
    </Card>
  );
}
