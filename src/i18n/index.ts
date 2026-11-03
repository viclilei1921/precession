import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import { invokeCommand } from '@/bridge/invoke';
import { isTauri, locale } from '@/bridge/platform';
import { readPreference, writePreference } from '@/bridge/preferences';
import enUSCipher from './locales/en-US/cipher.json';
import enUSCommon from './locales/en-US/common.json';
import enUSErrors from './locales/en-US/errors.json';
import enUSFiles from './locales/en-US/files.json';
import enUSGrowth from './locales/en-US/growth.json';
import enUSImage from './locales/en-US/image.json';
import enUSJournal from './locales/en-US/journal.json';
import enUSLayout from './locales/en-US/layout.json';
import enUSLibrary from './locales/en-US/library.json';
import enUSLock from './locales/en-US/lock.json';
import enUSNav from './locales/en-US/nav.json';
import enUSPlan from './locales/en-US/plan.json';
import enUSSettings from './locales/en-US/settings.json';
import enUSTask from './locales/en-US/task.json';
import enUSToday from './locales/en-US/today.json';
import enUSUi from './locales/en-US/ui.json';
import enUSVideo from './locales/en-US/video.json';
import zhCNCipher from './locales/zh-CN/cipher.json';
import zhCNCommon from './locales/zh-CN/common.json';
import zhCNErrors from './locales/zh-CN/errors.json';
import zhCNFiles from './locales/zh-CN/files.json';
import zhCNGrowth from './locales/zh-CN/growth.json';
import zhCNImage from './locales/zh-CN/image.json';
import zhCNJournal from './locales/zh-CN/journal.json';
import zhCNLayout from './locales/zh-CN/layout.json';
import zhCNLibrary from './locales/zh-CN/library.json';
import zhCNLock from './locales/zh-CN/lock.json';
import zhCNNav from './locales/zh-CN/nav.json';
import zhCNPlan from './locales/zh-CN/plan.json';
import zhCNSettings from './locales/zh-CN/settings.json';
import zhCNTask from './locales/zh-CN/task.json';
import zhCNToday from './locales/zh-CN/today.json';
import zhCNUi from './locales/zh-CN/ui.json';
import zhCNVideo from './locales/zh-CN/video.json';

export const SUPPORTED_LANGUAGES = ['zh-CN', 'en-US'] as const;
export type Language = (typeof SUPPORTED_LANGUAGES)[number];
/** 用户偏好。`system` 表示跟随系统，实际生效语言另行解析 */
export type LanguagePreference = Language | 'system';

/** 持久化到 Tauri Store / localStorage 的 key */
export const LANGUAGE_PREFERENCE_KEY = 'language';
/** 兼容旧代码的 localStorage key 名称 */
export const LANGUAGE_STORAGE_KEY = 'precession-language';
export const DEFAULT_LANGUAGE: Language = 'zh-CN';

const resources = {
  'zh-CN': {
    cipher: zhCNCipher,
    common: zhCNCommon,
    errors: zhCNErrors,
    files: zhCNFiles,
    growth: zhCNGrowth,
    image: zhCNImage,
    journal: zhCNJournal,
    layout: zhCNLayout,
    library: zhCNLibrary,
    lock: zhCNLock,
    nav: zhCNNav,
    plan: zhCNPlan,
    settings: zhCNSettings,
    task: zhCNTask,
    today: zhCNToday,
    ui: zhCNUi,
    video: zhCNVideo
  },
  'en-US': {
    cipher: enUSCipher,
    common: enUSCommon,
    errors: enUSErrors,
    files: enUSFiles,
    growth: enUSGrowth,
    image: enUSImage,
    journal: enUSJournal,
    layout: enUSLayout,
    library: enUSLibrary,
    lock: enUSLock,
    nav: enUSNav,
    plan: enUSPlan,
    settings: enUSSettings,
    task: enUSTask,
    today: enUSToday,
    ui: enUSUi,
    video: enUSVideo
  }
};

let preference: LanguagePreference = 'system';

/** 系统 locale 标准化：zh-* → zh-CN，en-* → en-US，其余兜底 en-US */
export function normalizeLanguage(lang: string | null | undefined): Language {
  if (!lang) {
    return DEFAULT_LANGUAGE;
  }
  const lower = lang.toLowerCase();
  if (lower.startsWith('zh')) {
    return 'zh-CN';
  }
  if (lower.startsWith('en')) {
    return 'en-US';
  }
  if ((SUPPORTED_LANGUAGES as readonly string[]).includes(lang)) {
    return lang as Language;
  }
  return 'en-US';
}

/** 读取系统语言。桌面、移动端都走 OS 插件；浏览器里退回 navigator。 */
export async function getSystemLanguage(): Promise<Language> {
  if (!isTauri()) {
    return normalizeLanguage(navigator.language);
  }
  try {
    return normalizeLanguage(await locale());
  } catch {
    return normalizeLanguage(navigator.language);
  }
}

export function resolveLanguage(pref: LanguagePreference, system: Language): Language {
  return pref === 'system' ? system : pref;
}

/** 当前用户偏好（启动时从 Store 恢复） */
export function getLanguagePreference(): LanguagePreference {
  return preference;
}

/** 当前实际生效语言 */
export function getCurrentLanguage(): Language {
  return normalizeLanguage(i18n.resolvedLanguage ?? i18n.language);
}

/** Intl、日期格式化使用的 locale */
export function currentLocale(): string {
  return getCurrentLanguage();
}

function isLanguagePreference(value: unknown): value is LanguagePreference {
  return value === 'system' || (SUPPORTED_LANGUAGES as readonly unknown[]).includes(value);
}

function applyDocumentLanguage(lang: Language): void {
  document.documentElement.lang = lang;
  document.documentElement.dir = 'ltr';
}

/** 把语言同步给 Rust，托盘等原生 UI 才能跟着切换。 */
async function syncBackendLocale(lang: Language): Promise<void> {
  try {
    await invokeCommand('set_app_locale', { language: lang });
  } catch {
    /* 浏览器调试或旧后端没有这个命令时忽略 */
  }
}

/**
 * 初始化 i18n。
 *
 * 检测优先级：Store 里的用户选择 > Tauri 系统 locale > 兜底 zh-CN。
 * 调用前应先 `preloadPreferences(['language', 'appearance'])`。
 */
export async function setupI18n(): Promise<void> {
  const stored = readPreference<LanguagePreference>(LANGUAGE_PREFERENCE_KEY);
  preference = isLanguagePreference(stored) ? stored : 'system';

  const system = await getSystemLanguage();
  const initialLang = resolveLanguage(preference, system);

  await i18n.use(initReactI18next).init({
    resources,
    lng: initialLang,
    fallbackLng: DEFAULT_LANGUAGE,
    supportedLngs: [...SUPPORTED_LANGUAGES],
    defaultNS: 'common',
    ns: Object.keys(resources[DEFAULT_LANGUAGE]),
    interpolation: { escapeValue: false },
    react: { useSuspense: false }
  });

  applyDocumentLanguage(initialLang);
  i18n.on('languageChanged', (lng) => {
    applyDocumentLanguage(normalizeLanguage(lng));
    void syncBackendLocale(normalizeLanguage(lng));
  });
  void syncBackendLocale(initialLang);
}

/**
 * 修改并持久化语言偏好。
 *
 * @returns 实际生效的语言
 */
export async function setLanguagePreference(next: LanguagePreference): Promise<Language> {
  preference = next;
  const system = await getSystemLanguage();
  const lang = resolveLanguage(next, system);

  await writePreference(LANGUAGE_PREFERENCE_KEY, next);
  if (normalizeLanguage(i18n.language) !== lang) {
    await i18n.changeLanguage(lang);
  } else {
    applyDocumentLanguage(lang);
    void syncBackendLocale(lang);
  }
  return lang;
}

export default i18n;
