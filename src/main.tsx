import React from 'react';
import ReactDOM from 'react-dom/client';
import { preloadPreferences } from '@/bridge/preferences';
import { hydrateAppearance } from '@/store/app';
import App from './App';
import { setupI18n } from './i18n';
import './style/index.css';

async function bootstrap() {
  // 先读 Store，再初始化 i18n 和外观，避免首屏语言/主题闪一下。
  await preloadPreferences(['language', 'appearance']);
  await setupI18n();
  hydrateAppearance();

  ReactDOM.createRoot(document.body).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
}

void bootstrap();
