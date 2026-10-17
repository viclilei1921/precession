import type { LinkProps } from '@tanstack/react-router';
import { Link } from '@tanstack/react-router';
import clsx from 'clsx';
import type { ReactNode } from 'react';
import { rippleHost, useRipple } from '@/components/ripple';
import styles from './index.module.css';

type ButtonVariant = 'filled' | 'outlined' | 'text';
type ButtonTone = 'destructive';

type LinkButtonProps = {
  to: LinkProps['to'];
  variant?: ButtonVariant;
  tone?: ButtonTone;
  className?: string;
  children: ReactNode;
};

const variantClass: Record<ButtonVariant, string> = {
  filled: styles.filled,
  outlined: styles.outlined,
  text: styles.text
};

/** 外观与 Button 相同的站内链接。 */
export function LinkButton({ to, variant = 'filled', tone, className, children }: LinkButtonProps) {
  const ripple = useRipple();

  return (
    <Link
      to={to}
      className={clsx(rippleHost, styles.button, variantClass[variant], className)}
      data-tone={tone}
      onPointerDown={ripple.onPointerDown}
      onKeyDown={ripple.onKeyDown}
    >
      {ripple.nodes}
      {children}
    </Link>
  );
}
