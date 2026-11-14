import { ArrowLeftIcon, ArrowRightIcon } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { IconButton } from '@/components/button/icon';
import { closeTopLayer } from '@/layout/mobile/back';
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

/**
 * 标题栏里的返回 / 前进，返回键分层：
 *
 * 1. 有打开的浮层（弹窗、看图、全屏预览、对比等，走 useBackLayer 注册）
 *    就先收掉最上面一层——比如浏览图片时退回看图页自己的空态、弹窗打开时先关弹窗；
 * 2. 没有浮层时才退路由历史。
 *
 * 也就是说按返回是「先收浮层，再跳路由」，每按一次退一层。前进键只管路由。
 */
export function TitleBarHistory() {
  const { t } = useTranslation('layout');
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

  function onBack() {
    if (closeTopLayer()) {
      return;
    }
    router.history.back();
  }

  return (
    <div className={styles.history}>
      <IconButton label={t('titleBar.back')} size="compact" disabled={!unlocked} onClick={onBack}>
        <ArrowLeftIcon weight="regular" />
      </IconButton>
      <IconButton
        label={t('titleBar.forward')}
        size="compact"
        disabled={!unlocked || !nav.forward}
        onClick={() => router.history.forward()}
      >
        <ArrowRightIcon weight="regular" />
      </IconButton>
    </div>
  );
}
