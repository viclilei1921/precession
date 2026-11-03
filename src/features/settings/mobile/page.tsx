import {
  CaretRightIcon,
  CubeIcon,
  DownloadSimpleIcon,
  FileTextIcon,
  FingerprintIcon,
  GlobeIcon,
  LockIcon,
  PaletteIcon,
  TextAaIcon
} from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import type { ReactNode } from 'react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { arch, platform, version } from '@/bridge';
import { archiveExportShare, archiveImport, pickArchiveOpen, todayArchiveName } from '@/bridge/archive';
import { dbDisableDeviceUnlock, dbEnableDeviceUnlock, dbLock, dbStatus } from '@/bridge/db';
import { Button } from '@/components/button';
import { PasswordInput } from '@/components/password-input';
import type { ReaderFont } from '@/features/library/reading-settings';
import { loadReadingSettings } from '@/features/library/reading-settings';
import { formatBytes } from '@/features/settings/bytes';
import type { ThemePreference } from '@/features/settings/preferences';
import { ReadingSection } from '@/features/settings/reading';
import { Switch } from '@/features/settings/switch';
import type { Language, LanguagePreference } from '@/i18n';
import { getCurrentLanguage, getLanguagePreference, setLanguagePreference } from '@/i18n';
import { AppBar } from '@/layout/mobile/app-bar';
import { popRoute } from '@/layout/mobile/back';
import { Sheet } from '@/layout/mobile/sheet';
import { useToastStore } from '@/layout/mobile/toast';
import { appVersionsQuery } from '@/query/about/query';
import { archiveInfoQuery } from '@/query/archive/query';
import { queryClient } from '@/query/client';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import styles from './page.module.css';

type SheetName = 'theme' | 'language' | 'reading' | 'versions' | 'archive' | 'device' | 'lockconfirm';

/** 移动端设置。面板开合写在地址栏的 sheet 上。 */
export function MobileSettingsPage() {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();
  const search = useSearch({ from: '/settings' });
  const showToast = useToastStore((state) => state.show);
  const preference = useAppStore((state) => state.themePreference);
  const reduceMotion = useAppStore((state) => state.reduceMotion);
  const setReduceMotion = useAppStore((state) => state.setReduceMotion);
  const deviceUnlock = useAppStore((state) => state.dbStatus?.deviceUnlock ?? false);
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const info = useQuery(archiveInfoQuery);
  const versions = useQuery(appVersionsQuery);
  const [reading, setReading] = useState(loadReadingSettings);

  const themeLabel =
    preference === 'dark'
      ? t('appearance.theme.dark.label')
      : preference === 'system'
        ? t('appearance.theme.system.label')
        : t('appearance.theme.light.label');
  const languageName = (lang: Language) => (lang === 'zh-CN' ? t('language.zhCN.label') : t('language.enUS.label'));
  const languagePreference = getLanguagePreference();
  const languageLabel =
    languagePreference === 'system'
      ? t('language.extraSystem', { lang: languageName(getCurrentLanguage()) })
      : languageName(languagePreference);
  const fontLabel: Record<ReaderFont, string> = {
    wenkai: t('mobile.fonts.wenkai'),
    serif: t('mobile.fonts.serif'),
    sans: t('mobile.fonts.sans'),
    default: t('mobile.fonts.default')
  };
  const fileValue = info.data
    ? t('mobile.file.valueCount', { count: info.data.fileCount, size: formatBytes(info.data.fileBytes) })
    : t('mobile.file.valueLocal');

  useEffect(() => {
    if (search.sheet !== 'reading') {
      setReading(loadReadingSettings());
    }
  }, [search.sheet]);

  function openSheet(sheet: SheetName) {
    void navigate({ to: '/settings', search: { sheet } });
  }

  function closeSheet() {
    popRoute(() => {
      void navigate({ to: '/settings', search: {}, replace: true });
    });
  }

  async function toggleDevice() {
    if (!deviceUnlock) {
      openSheet('device');
      return;
    }
    try {
      await dbDisableDeviceUnlock();
      setDbStatus(await dbStatus());
      showToast(t('mobile.device.disabledNotice'));
    } catch (err) {
      showToast(errorMessage(err));
    }
  }

  return (
    <section className={styles.page}>
      <AppBar title={t('mobile.screenTitle')} subtitle={t('mobile.screenSubtitle')}>
        <button
          type="button"
          className={styles.icon}
          aria-label={t('mobile.versionsAria')}
          onClick={() => openSheet('versions')}
        >
          <CubeIcon weight="regular" />
        </button>
      </AppBar>
      <div className={styles.body}>
        <div className={styles.profile}>
          <div>
            <strong>Precession</strong>
            <span>{t('mobile.profileTagline')}</span>
          </div>
          <span className={styles.pill}>
            {versions.data?.app ? `v${versions.data.app}` : t('versions.extraDefault')}
          </span>
        </div>

        <Group title={t('appearance.title')} detail={t('mobile.appearanceDetail')}>
          <Row
            icon={<PaletteIcon weight="regular" />}
            tone="system"
            title={t('mobile.theme.title')}
            hint={t('mobile.theme.subtitle')}
            value={themeLabel}
            onClick={() => openSheet('theme')}
          />
          <Row
            icon={<GlobeIcon weight="regular" />}
            tone="system"
            title={t('language.title')}
            hint={t('language.detail')}
            value={languageLabel}
            onClick={() => openSheet('language')}
          />
          <div className={styles.row}>
            <span className={styles.mark} data-tone="system">
              <PaletteIcon weight="regular" />
            </span>
            <span className={styles.copy}>
              <span className={styles.title}>{t('appearance.motionRow.title')}</span>
              <span className={styles.hint}>{t('mobile.motionHint')}</span>
            </span>
            <Switch
              label={t('appearance.motionRow.title')}
              checked={reduceMotion}
              onClick={() => {
                const next = !reduceMotion;
                setReduceMotion(next);
                showToast(next ? t('appearance.motionNoticeOn') : t('appearance.motionNoticeOff'));
              }}
            />
          </div>
        </Group>

        <Group title={t('reading.title')} detail={t('reading.detail')}>
          <Row
            icon={<TextAaIcon weight="regular" />}
            tone="library"
            title={t('mobile.readingRowTitle')}
            hint={t('mobile.readingRowHint')}
            value={`${reading.fontSize} px · ${fontLabel[reading.fontFamily]}`}
            onClick={() => openSheet('reading')}
          />
        </Group>

        <Group title={t('security.title')} detail={t('security.extra')}>
          <div className={styles.row}>
            <span className={styles.mark} data-tone="plan">
              <FingerprintIcon weight="regular" />
            </span>
            <span className={styles.copy}>
              <span className={styles.title}>{t('mobile.device.title')}</span>
              <span className={styles.hint}>
                {deviceUnlock ? t('mobile.device.hintOn') : t('mobile.device.hintOff')}
              </span>
            </span>
            <Switch label={t('mobile.device.title')} checked={deviceUnlock} onClick={() => void toggleDevice()} />
          </div>
          <Row
            icon={<FileTextIcon weight="regular" />}
            tone="growth"
            title={t('security.fileRow.title')}
            hint={t('security.fileRow.hintDefault')}
            value={fileValue}
            onClick={() => void navigate({ to: PATH.files })}
          />
          <Row
            icon={<DownloadSimpleIcon weight="regular" />}
            tone="system"
            title={t('security.ioRow.title')}
            hint={t('mobile.ioHint')}
            onClick={() => openSheet('archive')}
          />
          <Row
            icon={<LockIcon weight="regular" />}
            tone="growth"
            title={t('mobile.lock.title')}
            hint={t('mobile.lock.hint')}
            onClick={() => openSheet('lockconfirm')}
          />
        </Group>

        <Group title={t('mobile.about.title')} detail={t('mobile.about.detail')}>
          <Row
            icon={<CubeIcon weight="regular" />}
            tone="journal"
            title={t('versions.title')}
            hint={t('mobile.about.versionsHint')}
            value={versions.data?.app ? `v${versions.data.app}` : ''}
            onClick={() => openSheet('versions')}
          />
        </Group>
      </div>

      {search.sheet === 'theme' ? <ThemeSheet onClose={closeSheet} /> : null}
      {search.sheet === 'language' ? <LanguageSheet onClose={closeSheet} /> : null}
      {search.sheet === 'reading' ? (
        <Sheet
          tall
          title={t('mobile.readingSheetTitle')}
          subtitle={t('mobile.readingSheetSubtitle')}
          onClose={closeSheet}
        >
          <ReadingSection />
        </Sheet>
      ) : null}
      {search.sheet === 'versions' ? <VersionsSheet onClose={closeSheet} /> : null}
      {search.sheet === 'archive' ? <ArchiveSheet onClose={closeSheet} /> : null}
      {search.sheet === 'device' ? <DeviceSheet onClose={closeSheet} /> : null}
      {search.sheet === 'lockconfirm' ? <LockSheet onClose={closeSheet} /> : null}
    </section>
  );
}

function Group({ title, detail, children }: { title: string; detail: string; children: ReactNode }) {
  return (
    <section className={styles.group}>
      <div className={styles.groupHead}>
        <strong>{title}</strong>
        <span>{detail}</span>
      </div>
      <div className={styles.card}>{children}</div>
    </section>
  );
}

function Row({
  icon,
  tone,
  title,
  hint,
  value,
  onClick
}: {
  icon: ReactNode;
  tone: string;
  title: string;
  hint: string;
  value?: string;
  onClick: () => void;
}) {
  return (
    <button type="button" className={styles.row} onClick={onClick}>
      <span className={styles.mark} data-tone={tone}>
        {icon}
      </span>
      <span className={styles.copy}>
        <span className={styles.title}>{title}</span>
        <span className={styles.hint}>{hint}</span>
      </span>
      {value ? <span className={styles.value}>{value}</span> : null}
      <CaretRightIcon className={styles.chev} weight="regular" />
    </button>
  );
}

function ThemeSheet({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation('settings');
  const preference = useAppStore((state) => state.themePreference);
  const setThemePreference = useAppStore((state) => state.setThemePreference);
  const showToast = useToastStore((state) => state.show);

  const themes: { value: ThemePreference; label: string; hint: string }[] = [
    { value: 'light', label: t('appearance.theme.light.label'), hint: t('appearance.theme.light.hint') },
    { value: 'dark', label: t('appearance.theme.dark.label'), hint: t('appearance.theme.dark.hint') },
    { value: 'system', label: t('appearance.theme.system.label'), hint: t('appearance.theme.system.hint') }
  ];

  return (
    <Sheet title={t('mobile.theme.title')} subtitle={t('mobile.theme.subtitle')} onClose={onClose}>
      <div className={styles.choices}>
        {themes.map((item) => (
          <button
            key={item.value}
            type="button"
            data-on={preference === item.value ? 'true' : undefined}
            onClick={() => {
              setThemePreference(item.value);
              showToast(
                item.value === 'system'
                  ? t('appearance.noticeSystem')
                  : t('appearance.noticeSwitched', { theme: item.label })
              );
              onClose();
            }}
          >
            <strong>{item.label}</strong>
            <span>{item.hint}</span>
          </button>
        ))}
      </div>
    </Sheet>
  );
}

function LanguageSheet({ onClose }: { onClose: () => void }) {
  const { t, i18n } = useTranslation('settings');
  const showToast = useToastStore((state) => state.show);
  const [preference, setPreference] = useState<LanguagePreference>(() => getLanguagePreference());

  const choices: { value: LanguagePreference; label: string; hint: string }[] = [
    { value: 'zh-CN', label: t('language.zhCN.label'), hint: t('language.zhCN.hint') },
    { value: 'en-US', label: t('language.enUS.label'), hint: t('language.enUS.hint') },
    { value: 'system', label: t('language.system.label'), hint: t('language.system.hint') }
  ];

  async function choose(next: LanguagePreference) {
    await setLanguagePreference(next);
    setPreference(next);
    // changeLanguage 之后组件还没重渲染，提示文字用 i18n.t 直接取新语言。
    const lang = getCurrentLanguage();
    const name = i18n.t(lang === 'zh-CN' ? 'settings:language.zhCN.label' : 'settings:language.enUS.label');
    showToast(
      next === 'system'
        ? i18n.t('settings:language.noticeSystem', { lang: name })
        : i18n.t('settings:language.noticeSwitched', { lang: name })
    );
    onClose();
  }

  return (
    <Sheet title={t('language.title')} subtitle={t('language.detail')} onClose={onClose}>
      <div className={styles.choices}>
        {choices.map((item) => (
          <button
            key={item.value}
            type="button"
            data-on={preference === item.value ? 'true' : undefined}
            onClick={() => void choose(item.value)}
          >
            <strong>{item.label}</strong>
            <span>{item.hint}</span>
          </button>
        ))}
      </div>
    </Sheet>
  );
}

function VersionsSheet({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation('settings');
  const versions = useQuery(appVersionsQuery);
  const rows = [
    {
      label: t('versions.app'),
      value: versions.data?.app || (versions.isPending ? t('versions.loading') : t('versions.unknown'))
    },
    { label: t('versions.system'), value: `${platform()} ${version()} (${arch()})` },
    {
      label: t('versions.webview'),
      value: versions.data?.webview || (versions.isPending ? t('versions.loading') : t('versions.unknown'))
    }
  ];

  return (
    <Sheet title={t('versions.title')} subtitle={t('mobile.versions.sheetSubtitle')} onClose={onClose}>
      <dl className={styles.versions}>
        {rows.map((row) => (
          <div key={row.label}>
            <dt>{row.label}</dt>
            <dd>{row.value}</dd>
          </div>
        ))}
      </dl>
      <p className={styles.note}>{t('mobile.versions.note')}</p>
    </Sheet>
  );
}

function ArchiveSheet({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation('settings');
  const showToast = useToastStore((state) => state.show);
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const [busy, setBusy] = useState<'export' | 'import' | ''>('');
  const [pending, setPending] = useState('');
  const [error, setError] = useState('');

  async function onExport() {
    setError('');
    setBusy('export');
    try {
      await archiveExportShare(todayArchiveName());
      showToast(t('mobile.archive.exportNotice'));
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy('');
    }
  }

  async function chooseImport() {
    setError('');
    const path = await pickArchiveOpen();
    if (path) {
      setPending(path);
    }
  }

  async function confirmImport() {
    setBusy('import');
    setError('');
    try {
      await archiveImport(pending);
      queryClient.clear();
      setDbStatus(await dbStatus());
    } catch (err) {
      setError(errorMessage(err));
      setBusy('');
    }
  }

  return (
    <Sheet title={t('mobile.archive.sheetTitle')} subtitle={t('mobile.archive.sheetSubtitle')} onClose={onClose}>
      <div className={styles.zone}>
        <strong>{t('mobile.archive.exportTitle')}</strong>
        <span>{t('mobile.archive.exportHint')}</span>
      </div>
      <Button disabled={busy !== ''} onClick={() => void onExport()}>
        {busy === 'export' ? t('security.ioRow.exporting') : t('mobile.archive.export')}
      </Button>
      <div className={styles.zone}>
        <strong>{t('mobile.archive.importTitle')}</strong>
        <span>{t('mobile.archive.importHint')}</span>
      </div>
      {pending ? (
        <>
          <p className={styles.note}>{pending.split(/[/\\]/).pop()}</p>
          <Button variant="outlined" disabled={busy !== ''} onClick={() => void confirmImport()}>
            {busy === 'import' ? t('security.ioRow.importing') : t('mobile.archive.confirmImport')}
          </Button>
        </>
      ) : (
        <Button variant="outlined" disabled={busy !== ''} onClick={() => void chooseImport()}>
          {t('mobile.archive.choose')}
        </Button>
      )}
      {error ? <p className={styles.error}>{error}</p> : null}
      <p className={styles.note}>{t('mobile.archive.note')}</p>
    </Sheet>
  );
}

function DeviceSheet({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation(['settings', 'common']);
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const showToast = useToastStore((state) => state.show);
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function enable() {
    if (!password) {
      setError(t('mobile.device.passwordRequired'));
      return;
    }
    setBusy(true);
    setError('');
    try {
      await dbEnableDeviceUnlock(password);
      setDbStatus(await dbStatus());
      showToast(t('mobile.device.enabledNotice'));
      onClose();
    } catch (err) {
      setError(errorMessage(err));
      setBusy(false);
    }
  }

  return (
    <Sheet
      title={t('mobile.device.sheetTitle')}
      subtitle={t('mobile.device.sheetSubtitle')}
      onClose={onClose}
      footer={
        <>
          <Button variant="text" disabled={busy} onClick={onClose}>
            {t('common:button.cancel')}
          </Button>
          <Button disabled={busy} onClick={() => void enable()}>
            {busy ? t('mobile.device.enabling') : t('mobile.device.enable')}
          </Button>
        </>
      }
    >
      <p className={styles.note}>{t('mobile.device.note')}</p>
      <PasswordInput
        label={t('mobile.device.password')}
        autoComplete="current-password"
        placeholder={t('mobile.device.placeholder')}
        value={password}
        disabled={busy}
        onChange={(event) => setPassword(event.target.value)}
      />
      {error ? <p className={styles.error}>{error}</p> : null}
    </Sheet>
  );
}

function LockSheet({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation(['settings', 'common']);
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function lockArchive() {
    setBusy(true);
    setError('');
    try {
      await dbLock();
      queryClient.clear();
      setDbStatus(await dbStatus());
    } catch (err) {
      setError(errorMessage(err));
      setBusy(false);
    }
  }

  return (
    <Sheet
      title={t('mobile.lock.title')}
      subtitle={t('mobile.lock.sheetSubtitle')}
      onClose={onClose}
      footer={
        <>
          <Button variant="text" disabled={busy} onClick={onClose}>
            {t('common:button.cancel')}
          </Button>
          <Button disabled={busy} onClick={() => void lockArchive()}>
            {busy ? t('mobile.lock.locking') : t('mobile.lock.title')}
          </Button>
        </>
      }
    >
      <p className={styles.note}>{t('mobile.lock.note')}</p>
      {error ? <p className={styles.error}>{error}</p> : null}
    </Sheet>
  );
}
