import {
  CaretRightIcon,
  CubeIcon,
  DownloadSimpleIcon,
  FileTextIcon,
  FingerprintIcon,
  LockIcon,
  PaletteIcon,
  TextAaIcon
} from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate, useSearch } from '@tanstack/react-router';
import type { ReactNode } from 'react';
import { useEffect, useState } from 'react';
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
import { AppBar } from '@/layout/mobile/app-bar';
import { Sheet } from '@/layout/mobile/sheet';
import { useToastStore } from '@/layout/mobile/toast';
import { appVersionsQuery } from '@/query/about/query';
import { archiveInfoQuery } from '@/query/archive/query';
import { queryClient } from '@/query/client';
import { PATH } from '@/router/path';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import styles from './page.module.css';

const THEMES: { value: ThemePreference; label: string; hint: string }[] = [
  { value: 'light', label: '浅色', hint: '明亮通透' },
  { value: 'dark', label: '深色', hint: '夜间阅读' },
  { value: 'system', label: '跟随系统', hint: '随系统切换' }
];

const FONT_LABEL: Record<ReaderFont, string> = {
  wenkai: '文楷',
  serif: '宋体',
  sans: '黑体',
  default: '默认'
};

type SheetName = 'theme' | 'reading' | 'versions' | 'archive' | 'device' | 'lockconfirm';

/** 移动端设置。面板开合写在地址栏的 sheet 上。 */
export function MobileSettingsPage() {
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
  const themeLabel = THEMES.find((item) => item.value === preference)?.label ?? '浅色';
  const fileValue = info.data ? `${info.data.fileCount} 个 · ${formatBytes(info.data.fileBytes)}` : '本机';

  useEffect(() => {
    if (search.sheet !== 'reading') {
      setReading(loadReadingSettings());
    }
  }, [search.sheet]);

  function openSheet(sheet: SheetName) {
    void navigate({ to: '/settings', search: { sheet } });
  }

  function closeSheet() {
    void navigate({ to: '/settings', search: {} });
  }

  async function toggleDevice() {
    if (!deviceUnlock) {
      openSheet('device');
      return;
    }
    try {
      await dbDisableDeviceUnlock();
      setDbStatus(await dbStatus());
      showToast('已关闭设备解锁');
    } catch (err) {
      showToast(errorMessage(err));
    }
  }

  return (
    <section className={styles.page}>
      <AppBar title="设置" subtitle="本机档案 · 已解锁">
        <button type="button" className={styles.icon} aria-label="版本信息" onClick={() => openSheet('versions')}>
          <CubeIcon weight="regular" />
        </button>
      </AppBar>
      <div className={styles.body}>
        <div className={styles.profile}>
          <div>
            <strong>Precession</strong>
            <span>本地优先的个人档案</span>
          </div>
          <span className={styles.pill}>{versions.data?.app ? `v${versions.data.app}` : '版本'}</span>
        </div>

        <Group title="外观" detail="外观与界面动效">
          <Row
            icon={<PaletteIcon weight="regular" />}
            tone="system"
            title="主题"
            hint="浅色、深色或跟随系统"
            value={themeLabel}
            onClick={() => openSheet('theme')}
          />
          <div className={styles.row}>
            <span className={styles.mark} data-tone="system">
              <PaletteIcon weight="regular" />
            </span>
            <span className={styles.copy}>
              <span className={styles.title}>减少动效</span>
              <span className={styles.hint}>关闭过渡和渐隐</span>
            </span>
            <Switch
              label="减少动效"
              checked={reduceMotion}
              onClick={() => {
                const next = !reduceMotion;
                setReduceMotion(next);
                showToast(next ? '已减少界面动效' : '已恢复界面动效');
              }}
            />
          </div>
        </Group>

        <Group title="阅读" detail="书库默认排版">
          <Row
            icon={<TextAaIcon weight="regular" />}
            tone="library"
            title="阅读偏好"
            hint="字号、字体、主题与版心"
            value={`${reading.fontSize} px · ${FONT_LABEL[reading.fontFamily]}`}
            onClick={() => openSheet('reading')}
          />
        </Group>

        <Group title="数据与安全" detail="仅本机">
          <div className={styles.row}>
            <span className={styles.mark} data-tone="plan">
              <FingerprintIcon weight="regular" />
            </span>
            <span className={styles.copy}>
              <span className={styles.title}>设备解锁</span>
              <span className={styles.hint}>
                {deviceUnlock ? '已启用 · 使用指纹或系统验证' : '启用后可以用指纹或系统验证打开'}
              </span>
            </span>
            <Switch label="设备解锁" checked={deviceUnlock} onClick={() => void toggleDevice()} />
          </div>
          <Row
            icon={<FileTextIcon weight="regular" />}
            tone="growth"
            title="文件"
            hint="记录上的附件都存在本机"
            value={fileValue}
            onClick={() => void navigate({ to: PATH.files })}
          />
          <Row
            icon={<DownloadSimpleIcon weight="regular" />}
            tone="system"
            title="导入与导出"
            hint="打包数据库与全部文件"
            onClick={() => openSheet('archive')}
          />
          <Row
            icon={<LockIcon weight="regular" />}
            tone="growth"
            title="锁定档案"
            hint="回到锁屏，重新验证后打开"
            onClick={() => openSheet('lockconfirm')}
          />
        </Group>

        <Group title="关于" detail="本机">
          <Row
            icon={<CubeIcon weight="regular" />}
            tone="journal"
            title="版本"
            hint="应用、系统与 WebView"
            value={versions.data?.app ? `v${versions.data.app}` : ''}
            onClick={() => openSheet('versions')}
          />
        </Group>
      </div>

      {search.sheet === 'theme' ? <ThemeSheet onClose={closeSheet} /> : null}
      {search.sheet === 'reading' ? (
        <Sheet tall title="阅读设置" subtitle="移动端默认 19 px" onClose={closeSheet}>
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
  const preference = useAppStore((state) => state.themePreference);
  const setThemePreference = useAppStore((state) => state.setThemePreference);
  const showToast = useToastStore((state) => state.show);

  return (
    <Sheet title="主题" subtitle="浅色、深色或跟随系统" onClose={onClose}>
      <div className={styles.choices}>
        {THEMES.map((item) => (
          <button
            key={item.value}
            type="button"
            data-on={preference === item.value ? 'true' : undefined}
            onClick={() => {
              setThemePreference(item.value);
              showToast(item.value === 'system' ? '主题将跟随系统外观' : `已切换到${item.label}主题`);
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

function VersionsSheet({ onClose }: { onClose: () => void }) {
  const versions = useQuery(appVersionsQuery);
  const rows = [
    { label: '应用', value: versions.data?.app || (versions.isPending ? '正在读取…' : '未知') },
    { label: '系统', value: `${platform()} ${version()} (${arch()})` },
    { label: 'WebView', value: versions.data?.webview || (versions.isPending ? '正在读取…' : '未知') }
  ];

  return (
    <Sheet title="版本" subtitle="侧车工具只在桌面端使用" onClose={onClose}>
      <dl className={styles.versions}>
        {rows.map((row) => (
          <div key={row.label}>
            <dt>{row.label}</dt>
            <dd>{row.value}</dd>
          </div>
        ))}
      </dl>
      <p className={styles.note}>移动端不包含加密与转码工具。</p>
    </Sheet>
  );
}

function ArchiveSheet({ onClose }: { onClose: () => void }) {
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
      showToast('已导出，请在分享面板里选择保存位置');
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
    <Sheet title="导入与导出" subtitle="把数据库和全部文件打成一个压缩包" onClose={onClose}>
      <div className={styles.zone}>
        <strong>导出当前档案</strong>
        <span>会包含数据库和全部附件</span>
      </div>
      <Button disabled={busy !== ''} onClick={() => void onExport()}>
        {busy === 'export' ? '正在导出…' : '导出档案'}
      </Button>
      <div className={styles.zone}>
        <strong>从压缩包恢复</strong>
        <span>会替换这台设备上的全部内容</span>
      </div>
      {pending ? (
        <>
          <p className={styles.note}>{pending.split(/[/\\]/).pop()}</p>
          <Button variant="outlined" disabled={busy !== ''} onClick={() => void confirmImport()}>
            {busy === 'import' ? '正在导入…' : '确认导入'}
          </Button>
        </>
      ) : (
        <Button variant="outlined" disabled={busy !== ''} onClick={() => void chooseImport()}>
          选择压缩包
        </Button>
      )}
      {error ? <p className={styles.error}>{error}</p> : null}
      <p className={styles.note}>导入会关闭设备解锁，之后要用这份档案的密码重新打开。</p>
    </Sheet>
  );
}

function DeviceSheet({ onClose }: { onClose: () => void }) {
  const setDbStatus = useAppStore((state) => state.setDbStatus);
  const showToast = useToastStore((state) => state.show);
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function enable() {
    if (!password) {
      setError('请输入档案密码');
      return;
    }
    setBusy(true);
    setError('');
    try {
      await dbEnableDeviceUnlock(password);
      setDbStatus(await dbStatus());
      showToast('这台设备已启用免密解锁');
      onClose();
    } catch (err) {
      setError(errorMessage(err));
      setBusy(false);
    }
  }

  return (
    <Sheet
      title="启用设备解锁"
      subtitle="用指纹或系统验证打开档案"
      onClose={onClose}
      footer={
        <>
          <Button variant="text" disabled={busy} onClick={onClose}>
            取消
          </Button>
          <Button disabled={busy} onClick={() => void enable()}>
            {busy ? '正在启用…' : '启用'}
          </Button>
        </>
      }
    >
      <p className={styles.note}>需要输入一次档案密码，之后这台设备可以用指纹或系统验证直接打开。</p>
      <PasswordInput
        label="档案密码"
        autoComplete="current-password"
        placeholder="输入档案密码"
        value={password}
        disabled={busy}
        onChange={(event) => setPassword(event.target.value)}
      />
      {error ? <p className={styles.error}>{error}</p> : null}
    </Sheet>
  );
}

function LockSheet({ onClose }: { onClose: () => void }) {
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
      title="锁定档案"
      subtitle="锁定后会回到锁屏"
      onClose={onClose}
      footer={
        <>
          <Button variant="text" disabled={busy} onClick={onClose}>
            取消
          </Button>
          <Button disabled={busy} onClick={() => void lockArchive()}>
            {busy ? '正在锁定…' : '锁定档案'}
          </Button>
        </>
      }
    >
      <p className={styles.note}>已启用设备解锁时，再打开需要指纹或系统验证；否则要重新输入档案密码。</p>
      {error ? <p className={styles.error}>{error}</p> : null}
    </Sheet>
  );
}
