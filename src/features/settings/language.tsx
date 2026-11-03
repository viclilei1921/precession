import { GlobeIcon } from '@phosphor-icons/react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Card } from '@/components/card';
import type { Language, LanguagePreference } from '@/i18n';
import { getCurrentLanguage, getLanguagePreference, setLanguagePreference } from '@/i18n';
import { CardHead } from './card-head';
import styles from './page.module.css';

type LanguageSectionProps = {
  onNotice: (message: string) => void;
};

export function LanguageSection({ onNotice }: LanguageSectionProps) {
  const { t, i18n } = useTranslation('settings');
  const [current, setCurrent] = useState<LanguagePreference>(() => getLanguagePreference());
  const effective = getCurrentLanguage();

  function languageName(lang: Language): string {
    return lang === 'zh-CN' ? t('language.zhCN.label') : t('language.enUS.label');
  }

  const LANGS: { value: LanguagePreference; label: string; hint: string }[] = [
    { value: 'zh-CN', label: t('language.zhCN.label'), hint: t('language.zhCN.hint') },
    { value: 'en-US', label: t('language.enUS.label'), hint: t('language.enUS.hint') },
    { value: 'system', label: t('language.system.label'), hint: t('language.system.hint') }
  ];

  async function choose(next: LanguagePreference) {
    const lang = await setLanguagePreference(next);
    setCurrent(next);
    // changeLanguage 之后组件还没重渲染，提示文字用 i18n.t 直接取新语言。
    const name = i18n.t(lang === 'zh-CN' ? 'settings:language.zhCN.label' : 'settings:language.enUS.label');
    onNotice(
      next === 'system'
        ? i18n.t('settings:language.noticeSystem', { lang: name })
        : i18n.t('settings:language.noticeSwitched', { lang: name })
    );
  }

  const currentLangLabel =
    current === 'system' ? t('language.extraSystem', { lang: languageName(effective) }) : languageName(current);

  return (
    <Card id="language" className={styles.section}>
      <CardHead
        tone="system"
        icon={GlobeIcon}
        title={t('language.title')}
        detail={t('language.detail')}
        extra={currentLangLabel}
      />
      <fieldset className={styles.themes} aria-label={t('language.title')}>
        {LANGS.map((item) => (
          <button
            key={item.value}
            type="button"
            className={styles.theme}
            data-on={current === item.value ? 'true' : undefined}
            aria-pressed={current === item.value}
            onClick={() => void choose(item.value)}
          >
            <span className={styles.themeCopy}>
              <strong>{item.label}</strong>
              <small>{item.hint}</small>
            </span>
          </button>
        ))}
      </fieldset>
    </Card>
  );
}
