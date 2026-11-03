import { useTranslation } from 'react-i18next';
import type { Almanac } from '@/bridge/calendar';
import {
  translateAlmanacWord,
  translateAnimal,
  translateChongSha,
  translateDirection,
  translateGanZhi,
  translateJianChu,
  translatePengZu
} from './almanac-dict';
import styles from './calendar.module.css';
import { adviceWords, isNoData } from './hours';

type AlmanacCardProps = {
  almanac: Almanac | undefined;
  pending: boolean;
  error: string;
};

function tagItems(words: string[]) {
  const seen = new Map<string, number>();
  return words.map((word) => {
    const count = (seen.get(word) ?? 0) + 1;
    seen.set(word, count);
    return { key: count === 1 ? word : `${word}-${count}`, word };
  });
}

export function AlmanacCard({ almanac, pending, error }: AlmanacCardProps) {
  const { t } = useTranslation('today');

  if (error) {
    return (
      <article className={styles.card}>
        <p className={styles.error}>{error}</p>
      </article>
    );
  }
  if (!almanac) {
    return (
      <article className={styles.card}>
        <p className={styles.note}>{pending ? t('common.loading') : t('almanac.noData')}</p>
      </article>
    );
  }

  const yi = tagItems(adviceWords(almanac.yi));
  const ji = tagItems(adviceWords(almanac.ji));
  const value = (raw: string) => (isNoData(raw) ? t('hours.none') : raw);
  const line = [
    almanac.animal ? t('almanac.yearLine', { animal: translateAnimal(almanac.animal) }) : '',
    almanac.ganZhi ? t('almanac.dayLine', { ganZhi: translateGanZhi(almanac.ganZhi) }) : '',
    almanac.jianChu ? translateJianChu(almanac.jianChu) : ''
  ]
    .filter(Boolean)
    .join(' · ');
  const gods = [
    almanac.chongSha ? translateChongSha(value(almanac.chongSha)) : '',
    almanac.caiShen ? t('almanac.caiShen', { name: translateDirection(value(almanac.caiShen)) }) : '',
    almanac.xiShen ? t('almanac.xiShen', { name: translateDirection(value(almanac.xiShen)) }) : '',
    almanac.fuShen ? t('almanac.fuShen', { name: translateDirection(value(almanac.fuShen)) }) : '',
    almanac.pengZu ? t('almanac.pengZu', { name: translatePengZu(value(almanac.pengZu)) }) : ''
  ]
    .filter(Boolean)
    .join(' · ');

  return (
    <article className={styles.card}>
      <p className={styles.lunar}>{almanac.lunar.text || t('almanac.lunarMissing')}</p>
      <p className={styles.ganzhi}>{line}</p>
      <div className={styles.advice}>
        <div className={styles.yi}>
          <h3>{t('almanac.yi')}</h3>
          <ul className={styles.tags}>
            {yi.map((item) => (
              <li key={item.key}>{translateAlmanacWord(item.word)}</li>
            ))}
          </ul>
        </div>
        <div className={styles.ji}>
          <h3>{t('almanac.ji')}</h3>
          <ul className={styles.tags}>
            {ji.map((item) => (
              <li key={item.key}>{translateAlmanacWord(item.word)}</li>
            ))}
          </ul>
        </div>
      </div>
      {gods ? <p className={styles.gods}>{gods}</p> : null}
    </article>
  );
}
