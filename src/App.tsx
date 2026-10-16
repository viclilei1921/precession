import { TitleBar } from '@/layout/titlebar/index';
import { useAppStore } from '@/store/app';
import styles from './app.module.css';
import { ContentView } from './layout/content';
import { LockView } from './layout/lock';

export default function App() {
  const unlockedDb = useAppStore((state) => state.dbStatus?.unlocked);

  return (
    <div className={styles.app}>
      <TitleBar />
      {unlockedDb ? <ContentView /> : <LockView />}
    </div>
  );
}
