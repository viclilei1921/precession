import { useTranslation } from 'react-i18next';
import type { ImageQuality } from '@/bridge/task';
import styles from './quality-line.module.css';

type QualityLineProps = {
  quality: ImageQuality;
};

/** 完成的图片任务上显示 SSIMULACRA2 分档，或打分没有完成的说明。 */
export function QualityLine({ quality }: QualityLineProps) {
  const { t } = useTranslation('image');
  if (quality.problem) {
    return <p className={styles.problem}>{t(`quality.problems.${quality.problem}`)}</p>;
  }
  if (quality.band == null || quality.score == null) {
    return null;
  }
  return (
    <div className={styles.quality}>
      <p className={styles.lead}>{t(`quality.bands.${quality.band}`)}</p>
      <p className={styles.meta}>{t('quality.score', { score: quality.score.toFixed(1) })}</p>
    </div>
  );
}
