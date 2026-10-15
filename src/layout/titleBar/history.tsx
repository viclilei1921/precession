import { ArrowLeftIcon, ArrowRightIcon } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { router } from '@/router';
import { useAppStore } from '@/store/app';
import styles from './index.module.css';

type HistoryNav = {
  back: boolean;
  forward: boolean;
};

/** 本次会话里到过的最远历史位置。刷新后前进栈无法从浏览器读出，所以从当前位置重新记。 */
let maxIndex = router.history.location.state.__TSR_index ?? 0;

function readHistoryNav(): HistoryNav {
  const index = router.history.location.state.__TSR_index ?? 0;
  return {
    back: index > 0,
    forward: index < maxIndex
  };
}

/** 标题栏里的路由后退、前进。锁屏时不能改底下的页面。 */
export function TitleBarHistory() {
  const unlocked = useAppStore((state) => Boolean(state.dbStatus?.unlocked));
  const [nav, setNav] = useState(readHistoryNav);

  useEffect(() => {
    return router.history.subscribe(({ location, action }) => {
      const index = location.state.__TSR_index ?? 0;
      if (action.type === 'PUSH') {
        maxIndex = index;
      } else if (index > maxIndex) {
        maxIndex = index;
      }
      setNav({
        back: index > 0,
        forward: index < maxIndex
      });
    });
  }, []);

  return (
    <div className={styles.history}>
      <button
        type="button"
        className={styles.navButton}
        title="后退"
        aria-label="后退"
        disabled={!unlocked || !nav.back}
        onClick={() => router.history.back()}
      >
        <ArrowLeftIcon className={styles.icon} weight="regular" />
      </button>
      <button
        type="button"
        className={styles.navButton}
        title="前进"
        aria-label="前进"
        disabled={!unlocked || !nav.forward}
        onClick={() => router.history.forward()}
      >
        <ArrowRightIcon className={styles.icon} weight="regular" />
      </button>
    </div>
  );
}
