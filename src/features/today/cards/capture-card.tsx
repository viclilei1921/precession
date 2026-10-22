import { FlagIcon, ListChecksIcon, PencilSimpleIcon, QuotesIcon } from '@phosphor-icons/react';
import { useNavigate } from '@tanstack/react-router';
import { ChipButton } from '@/components/button/chip';
import { useCardLayout } from '@/features/today/board/board';
import { PATH } from '@/router/path';
import styles from './tile.module.css';
import { TileHead } from './tile-head';

/** 四个入口沿用各模块的新建跳转。 */
export function CaptureCard() {
  const navigate = useNavigate();

  useCardLayout('capture', 'ready');

  return (
    <article className={styles.card} data-static="true">
      <TileHead tone="plan" icon={PencilSimpleIcon} title="待记" />
      <div className={styles.chips}>
        <ChipButton
          tone="journal"
          onClick={() => void navigate({ to: PATH.journal, search: { create: true, kind: 'diary' } })}
        >
          <PencilSimpleIcon weight="regular" />
          文字
        </ChipButton>
        <ChipButton
          tone="growth"
          onClick={() => void navigate({ to: PATH.growth, search: { create: true, kind: 'milestone' } })}
        >
          <FlagIcon weight="regular" />
          里程碑
        </ChipButton>
        <ChipButton tone="library" onClick={() => void navigate({ to: PATH.library })}>
          <QuotesIcon weight="regular" />
          书摘
        </ChipButton>
        <ChipButton tone="plan" onClick={() => void navigate({ to: PATH.plan, search: { create: true } })}>
          <ListChecksIcon weight="regular" />
          计划
        </ChipButton>
      </div>
    </article>
  );
}
