import type { ReactNode } from 'react';
import { InfoTip } from '@/components/tooltip';
import styles from './params.module.css';

type HelpRowProps = {
  /** 说明按钮的无障碍名称 */
  label: string;
  /** 悬停后看到的详细介绍 */
  title: string;
  /** 勾选框让按钮紧跟文字；输入框则把按钮对齐到框的中线 */
  align?: 'field' | 'check';
  children: ReactNode;
};

/** 参数控件和它的说明按钮排在一行。 */
export function HelpRow({ label, title, align = 'field', children }: HelpRowProps) {
  return (
    <div className={align === 'check' ? styles.check : styles.help}>
      <div className={align === 'check' ? styles.checkBody : styles.body}>{children}</div>
      <InfoTip className={align === 'field' ? styles.mark : undefined} label={label} title={title} />
    </div>
  );
}
