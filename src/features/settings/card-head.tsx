import type { Icon } from '@phosphor-icons/react';
import styles from './page.module.css';

type CardTone = 'system' | 'library' | 'plan' | 'journal';

type CardHeadProps = {
  tone: CardTone;
  icon: Icon;
  title: string;
  detail: string;
  extra: string;
};

/** 设置卡片的图标、标题和右侧状态。 */
export function CardHead({ tone, icon: Glyph, title, detail, extra }: CardHeadProps) {
  return (
    <header className={styles.cardHead}>
      <span className={styles.badge} data-tone={tone}>
        <Glyph weight="regular" />
      </span>
      <div className={styles.cardTitle}>
        <h2 className={styles.heading}>{title}</h2>
        <p className={styles.sub}>{detail}</p>
      </div>
      <span className={styles.extra}>{extra}</span>
    </header>
  );
}
