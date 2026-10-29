import type { Icon } from '@phosphor-icons/react';
import { CubeIcon, MagnifyingGlassIcon, PaletteIcon, ShieldCheckIcon, TextAaIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { useEffect, useRef, useState } from 'react';
import { Button } from '@/components/button';
import { rippleHost, useRipple } from '@/components/ripple';
import { Scroll } from '@/components/scroll';
import { usePageSearch } from '@/hooks/search';
import { useSearchStore } from '@/store/search';
import { AppearanceSection } from './appearance';
import styles from './page.module.css';
import { ReadingSection } from './reading';
import { SecuritySection } from './security';
import { VersionsSection } from './versions';

const SECTIONS = [
  {
    id: 'appearance',
    label: '外观',
    icon: PaletteIcon,
    words: '外观 主题 浅色 深色 跟随系统 侧栏 动效 动画 界面'
  },
  {
    id: 'reading',
    label: '阅读',
    icon: TextAaIcon,
    words: '阅读 字号 字体 行距 页宽 主题 排版 书库 文楷 宋体 黑体'
  },
  {
    id: 'security',
    label: '数据与安全',
    icon: ShieldCheckIcon,
    words: '数据 安全 锁定 档案 设备解锁 密码 文件 数据库 路径 导入 导出'
  },
  {
    id: 'versions',
    label: '版本',
    icon: CubeIcon,
    words: '版本 应用 系统 webview ffmpeg cjxl avifenc 侧车'
  }
] as const;

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
  const query = usePageSearch('搜索设置').trim();
  const viewportRef = useRef<HTMLDivElement>(null);
  const [active, setActive] = useState<string>(SECTIONS[0].id);
  const [notice, setNotice] = useState('');
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
      <Scroll className={styles.scroll} axis="vertical" viewportRef={viewportRef}>
        <div className={styles.body}>
          <nav className={styles.jump} aria-label="设置分类">
            {visible.map((item) => (
              <JumpLink
                key={item.id}
                label={item.label}
                icon={item.icon}
                current={item.id === active}
                onClick={() => jumpTo(item.id)}
              />
            ))}
            <span className={styles.jumpNote}>{visible.length} 组设置</span>
          </nav>
          <p className={styles.notice} role="status">
            {notice}
          </p>
          {visible.length === 0 ? (
            <div className={styles.empty}>
              <MagnifyingGlassIcon className={styles.emptyIcon} weight="regular" />
              <p>没有找到相关设置。换个词试试，比如「主题」「字号」或「导入」。</p>
              <Button variant="text" onClick={() => useSearchStore.getState().setValue('')}>
                清除搜索
              </Button>
            </div>
          ) : (
            <div className={styles.stack}>
              {shown.has('appearance') ? <AppearanceSection onNotice={setNotice} /> : null}
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
