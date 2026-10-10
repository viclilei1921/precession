import { TitleBar } from '@/layout/titleBar/index';
import { useAppStore } from '@/store/app';
import styles from './app.module.css';
import { ContentView } from './layout/contentView';
import { LockView } from './layout/lockView';

export default function App() {
  const unlockedDb = useAppStore((state) => state.dbStatus?.unlocked);

  return (
    <div className={styles.app}>
      <TitleBar />
      <main className={styles.main}>
        {unlockedDb ? <ContentView /> : <LockView />}
      </main>
    </div>
  );
}
