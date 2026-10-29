import type { Almanac } from '@/bridge/calendar';
import styles from './calendar.module.css';
import { adviceWords } from './hours';

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
        <p className={styles.note}>{pending ? '正在读取…' : '这一天没有黄历。'}</p>
      </article>
    );
  }

  const yi = tagItems(adviceWords(almanac.yi));
  const ji = tagItems(adviceWords(almanac.ji));
  const line = [`${almanac.animal}年`, `${almanac.ganZhi}日`, almanac.jianChu].filter(Boolean).join(' · ');
  const gods = [
    almanac.chongSha,
    almanac.caiShen ? `财神${almanac.caiShen}` : '',
    almanac.xiShen ? `喜神${almanac.xiShen}` : '',
    almanac.fuShen ? `福神${almanac.fuShen}` : '',
    almanac.pengZu ? `彭祖${almanac.pengZu}` : ''
  ]
    .filter(Boolean)
    .join(' · ');

  return (
    <article className={styles.card}>
      <p className={styles.lunar}>{almanac.lunar.text || '农历未收录'}</p>
      <p className={styles.ganzhi}>{line}</p>
      <div className={styles.advice}>
        <div className={styles.yi}>
          <h3>宜</h3>
          <ul className={styles.tags}>
            {yi.map((item) => (
              <li key={item.key}>{item.word}</li>
            ))}
          </ul>
        </div>
        <div className={styles.ji}>
          <h3>忌</h3>
          <ul className={styles.tags}>
            {ji.map((item) => (
              <li key={item.key}>{item.word}</li>
            ))}
          </ul>
        </div>
      </div>
      {gods ? <p className={styles.gods}>{gods}</p> : null}
    </article>
  );
}
