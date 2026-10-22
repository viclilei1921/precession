import type { Icon } from '@phosphor-icons/react';
import styles from './tile.module.css';

type TileTone = 'plan' | 'journal' | 'growth' | 'library' | 'toolbox';

type TileHeadProps = {
  tone: TileTone;
  icon: Icon;
  title: string;
  extra?: string;
  /** 逾期这类需要被看见的附加信息 */
  warn?: boolean;
};

export function TileHead({ tone, icon: IconGlyph, title, extra, warn }: TileHeadProps) {
  return (
    <header className={styles.head}>
      <span className={styles.badge} data-tone={tone}>
        <IconGlyph weight="regular" />
      </span>
      <h2>{title}</h2>
      {extra ? <span className={warn ? styles.warn : styles.extra}>{extra}</span> : null}
    </header>
  );
}
