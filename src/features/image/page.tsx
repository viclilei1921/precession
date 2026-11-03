import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { fileSizes, taskEnqueue } from '@/bridge';
import type { Task } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { FileDrop } from '@/components/file-drop';
import { InfoTip } from '@/components/tooltip';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import type { ImageFile } from './components/file-list';
import { FileList } from './components/file-list';
import { ParamsPanel } from './components/params-panel';
import type { ImageTab } from './components/queue';
import { ImageQueue } from './components/queue';
import styles from './page.module.css';
import { buildAvifParams, buildJxlParams, formFromParams } from './params';
import { defaultAvifSummary, defaultJxlSummary, initialImageDraft } from './params-draft';
import { imageOutput } from './path';

const imageExtensions = ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp', 'tif', 'tiff'];

type ImageFormat = 'avif' | 'jxl';

type FormError = { key: string } | { text: string };

/** 前端校验错误用稳定的 key 表示，避免比较翻译后的文本。 */
class ValidationError extends Error {
  readonly key: string;

  constructor(key: string) {
    super(key);
    this.name = 'ValidationError';
    this.key = key;
  }
}

let fileKey = 0;

function nextFileKey() {
  fileKey += 1;
  return `image-${fileKey}`;
}

export function ImagePage() {
  const mobile = useAppStore((state) => state.isMobile());
  if (mobile) {
    return <section className={styles.page} />;
  }
  return <ImageWorkspace />;
}

function ImageWorkspace() {
  const { t } = useTranslation('image');
  const queryClient = useQueryClient();
  const [format, setFormat] = useState<ImageFormat>('avif');
  const [mode, setMode] = useState<'default' | 'custom'>('default');
  const [draft, setDraft] = useState(initialImageDraft);
  const [files, setFiles] = useState<ImageFile[]>([]);
  const [formError, setFormError] = useState<FormError | null>(null);
  const [tab, setTab] = useState<ImageTab>('run');
  const extension = format === 'avif' ? '.avif' : '.jxl';

  const enqueue = useMutation({
    mutationFn: async () => {
      const selected = files.filter((file) => file.checked);
      if (files.length === 0) {
        throw new ValidationError('page.error.noFiles');
      }
      if (selected.length === 0) {
        throw new ValidationError('page.error.noSelection');
      }
      for (const file of selected) {
        const output = imageOutput(file.input, extension);
        if (format === 'avif') {
          await taskEnqueue({
            kind: 'convertAvif',
            input: file.input,
            output,
            ...(mode === 'custom' ? { params: buildAvifParams(draft) } : {})
          });
        } else {
          await taskEnqueue({
            kind: 'convertJxl',
            input: file.input,
            output,
            ...(mode === 'custom' ? { params: buildJxlParams(draft) } : {})
          });
        }
      }
    },
    onSuccess: async () => {
      setFiles((current) => current.filter((file) => !file.checked));
      setFormError(null);
      setTab('wait');
      await refreshTasks(queryClient);
    },
    onError: (error) => {
      setFormError(error instanceof ValidationError ? { key: error.key } : { text: errorMessage(error) });
    }
  });

  async function addFiles(paths: string[]) {
    const incoming = paths.filter((path) => path.length > 0);
    if (incoming.length === 0) {
      return;
    }
    const sizes = new Map<string, number>();
    try {
      const found = await fileSizes(incoming);
      for (const item of found) {
        sizes.set(item.path, item.size);
      }
    } catch {
      sizes.clear();
    }
    setFiles((current) => {
      const known = new Set(current.map((file) => file.input));
      const next = incoming
        .filter((path) => !known.has(path))
        .map((input) => ({
          key: nextFileKey(),
          input,
          size: sizes.get(input) ?? null,
          checked: true
        }));
      return next.length === 0 ? current : [...current, ...next];
    });
    setFormError(null);
  }

  function restore(task: Task) {
    const saved = task.draft;
    if (!saved || (saved.kind !== 'convertAvif' && saved.kind !== 'convertJxl')) {
      return;
    }
    const form = formFromParams(saved.kind, saved.params);
    setFormat(saved.kind === 'convertAvif' ? 'avif' : 'jxl');
    setMode(form.mode);
    setDraft(form.draft);
    setFiles([
      {
        key: nextFileKey(),
        input: saved.input,
        size: null,
        checked: true
      }
    ]);
    setFormError(null);
    void fileSizes([saved.input])
      .then((found) => {
        const size = found.find((item) => item.path === saved.input)?.size ?? null;
        setFiles((current) => current.map((file) => (file.input === saved.input ? { ...file, size } : file)));
      })
      .catch(() => undefined);
  }

  return (
    <section className={styles.page}>
      <div className={styles.workspace}>
        <div className={styles.column}>
          <article className={styles.formatCard}>
            <div className={styles.formatRow}>
              <div className={styles.formatItem}>
                <span className={styles.formatLabel}>{t('page.formatLabel')}</span>
                <div className={styles.group} role="tablist" aria-label={t('page.formatAria')}>
                  <SegmentButton selected={format === 'avif'} onClick={() => setFormat('avif')}>
                    AVIF
                  </SegmentButton>
                  <SegmentButton selected={format === 'jxl'} onClick={() => setFormat('jxl')}>
                    JXL
                  </SegmentButton>
                </div>
                <InfoTip label={t('page.hintLabel')} title={t('page.hint')} />
              </div>
              <div className={styles.formatItem}>
                <span className={styles.formatLabel}>{t('page.modeLabel')}</span>
                <div className={styles.group} role="tablist" aria-label={t('page.modeAria')}>
                  <SegmentButton selected={mode === 'default'} onClick={() => setMode('default')}>
                    {t('page.default')}
                  </SegmentButton>
                  <SegmentButton selected={mode === 'custom'} onClick={() => setMode('custom')}>
                    {t('page.custom')}
                  </SegmentButton>
                </div>
              </div>
            </div>
            {mode === 'default' ? (
              <p className={styles.defaults}>{format === 'avif' ? defaultAvifSummary() : defaultJxlSummary()}</p>
            ) : (
              <ParamsPanel format={format} draft={draft} onChange={setDraft} />
            )}
          </article>
          <article className={styles.fileCard}>
            <div className={styles.fileTools}>
              <FileDrop accept={imageExtensions} onFiles={(paths) => void addFiles(paths)} />
              <div className={styles.toolsEnd}>
                {formError ? (
                  <p className={styles.error}>{'key' in formError ? t(formError.key) : formError.text}</p>
                ) : null}
                <Button disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
                  {t('page.start')}
                </Button>
              </div>
            </div>
            <FileList
              files={files}
              onChecked={(key, checked) => {
                setFiles((current) => current.map((file) => (file.key === key ? { ...file, checked } : file)));
                if (formError && 'key' in formError && formError.key === 'page.error.noSelection') {
                  setFormError(null);
                }
              }}
              onRemove={(key) => {
                setFiles((current) => current.filter((file) => file.key !== key));
              }}
              onSelectAll={() => {
                setFiles((current) => current.map((file) => ({ ...file, checked: true })));
                setFormError(null);
              }}
              onInvert={() => {
                setFiles((current) => current.map((file) => ({ ...file, checked: !file.checked })));
              }}
              onClear={() => setFiles([])}
              onDeleteSelected={() => {
                setFiles((current) => current.filter((file) => !file.checked));
              }}
            />
          </article>
        </div>
        <ImageQueue tab={tab} onTab={setTab} onEdit={restore} />
      </div>
    </section>
  );
}
