import type { Icon } from '@phosphor-icons/react';
import {
  CubeIcon,
  GlobeIcon,
  MagnifyingGlassIcon,
  PaletteIcon,
  ShieldCheckIcon,
  TextAaIcon
} from '@phosphor-icons/react';
import clsx from 'clsx';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/button';
import { rippleHost, useRipple } from '@/components/ripple';
import { Scroll } from '@/components/scroll';
import { usePageSearch } from '@/hooks/search';
import { useSearchStore } from '@/store/search';
import { AppearanceSection } from './appearance';
import { LanguageSection } from './language';
import styles from './page.module.css';
import { ReadingSection } from './reading';
import { SecuritySection } from './security';
import { VersionsSection } from './versions';

function matches(words: string, label: string, query: string) {
  if (!query) {
    return true;
  }
  return `${label} ${words}`.toLowerCase().includes(query.toLowerCase());
}

function JumpLink({
  label,
  icon: Glyph,
  current,
  onClick
}: {
  label: string;
  icon: Icon;
  current: boolean;
  onClick: () => void;
}) {
  const ripple = useRipple();

  return (
    <button
      type="button"
      className={clsx(rippleHost, styles.jumpLink)}
      aria-current={current ? 'true' : undefined}
      onClick={onClick}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      <Glyph weight="regular" />
      {label}
    </button>
  );
}

export function SettingsPage() {
  const { t } = useTranslation('settings');
  const query = usePageSearch(t('page.searchPlaceholder')).trim();
  const viewportRef = useRef<HTMLDivElement>(null);
  const [active, setActive] = useState<string>('appearance');
  const [notice, setNotice] = useState('');

  const SECTIONS = [
    {
      id: 'appearance',
      label: t('section.appearance.label'),
      words: t('section.appearance.words'),
      icon: PaletteIcon
    },
    {
      id: 'language',
      label: t('section.language.label'),
      words: t('section.language.words'),
      icon: GlobeIcon
    },
    {
      id: 'reading',
      label: t('section.reading.label'),
      words: t('section.reading.words'),
      icon: TextAaIcon
    },
    {
      id: 'security',
      label: t('section.security.label'),
      words: t('section.security.words'),
      icon: ShieldCheckIcon
    },
    {
      id: 'versions',
      label: t('section.versions.label'),
      words: t('section.versions.words'),
      icon: CubeIcon
    }
  ];

  const visible = SECTIONS.filter((item) => matches(item.words, item.label, query));
  const visibleKey = visible.map((item) => item.id).join();

  useEffect(() => {
    if (!notice) {
      return;
    }
    const timer = window.setTimeout(() => setNotice(''), 2400);
    return () => window.clearTimeout(timer);
  }, [notice]);

  useEffect(() => {
    if (visible.length > 0 && !visible.some((item) => item.id === active)) {
      setActive(visible[0].id);
    }
  }, [active, visible]);

  useEffect(() => {
    const root = viewportRef.current;
    if (!root) {
      return;
    }
    const nodes = visibleKey
      .split(',')
      .filter(Boolean)
      .map((id) => document.getElementById(id))
      .filter((node): node is HTMLElement => node instanceof HTMLElement);
    if (nodes.length === 0) {
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        const hit = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];
        if (hit?.target.id) {
          setActive(hit.target.id);
        }
      },
      { root, rootMargin: '-10% 0px -55% 0px', threshold: [0.15, 0.4, 0.75] }
    );
    for (const node of nodes) {
      observer.observe(node);
    }
    return () => observer.disconnect();
  }, [visibleKey]);

  function jumpTo(id: string) {
    document.getElementById(id)?.scrollIntoView({ block: 'start' });
    setActive(id);
  }

  const shown = new Set(visible.map((item) => item.id));

  return (
    <section className={styles.page}>
      <Scroll className={styles.scroll} axis="vertical" edge viewportRef={viewportRef}>
        <div className={styles.body}>
          <nav className={styles.jump} aria-label={t('page.navAria')}>
            {visible.map((item) => (
              <JumpLink
                key={item.id}
                label={item.label}
                icon={item.icon}
                current={item.id === active}
                onClick={() => jumpTo(item.id)}
              />
            ))}
            <span className={styles.jumpNote}>{t('page.groupCount', { count: visible.length })}</span>
          </nav>
          <p className={styles.notice} role="status">
            {notice}
          </p>
          {visible.length === 0 ? (
            <div className={styles.empty}>
              <MagnifyingGlassIcon className={styles.emptyIcon} weight="regular" />
              <p>{t('page.emptyTitle')}</p>
              <Button variant="text" onClick={() => useSearchStore.getState().setValue('')}>
                {t('page.emptyClear')}
              </Button>
            </div>
          ) : (
            <div className={styles.stack}>
              {shown.has('appearance') ? <AppearanceSection onNotice={setNotice} /> : null}
              {shown.has('language') ? <LanguageSection onNotice={setNotice} /> : null}
              {shown.has('reading') ? <ReadingSection /> : null}
              {shown.has('security') || shown.has('versions') ? (
                <div className={styles.pair}>
                  {shown.has('security') ? <SecuritySection onNotice={setNotice} /> : null}
                  {shown.has('versions') ? <VersionsSection onNotice={setNotice} /> : null}
                </div>
              ) : null}
            </div>
          )}
        </div>
      </Scroll>
    </section>
  );
}
