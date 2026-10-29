import React from 'react';
import ReactDOM from 'react-dom/client';
import { applyAppearance, loadAppearance } from '@/features/settings/preferences';
import App from './App';
import './style/index.css';

applyAppearance(loadAppearance());

ReactDOM.createRoot(document.body).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
