import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ffmpegProbe, fileSizes, taskEnqueue } from '@/bridge';
import type { Task, TimeSegment, VideoEncode } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Checkbox } from '@/components/checkbox';
import { FileDrop } from '@/components/file-drop';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { formatTime, parseTime } from '@/components/time-field/format';
import { InfoTip } from '@/components/tooltip';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { EncodePanel } from './components/encode-panel';
import { FileList } from './components/file-list';
import { QualityPreview } from './components/quality-preview';
import type { VideoTab } from './components/queue';
import { VideoQueue } from './components/queue';
import { buildEncode, formFromEncode } from './encode';
import { defaultEncodeSummary, initialEncodeDraft } from './encode-draft';
import styles from './page.module.css';
import { outputExtension, videoOutput } from './path';
import type { VideoFile, VideoOp } from './types';
import { nextClip, nextFileKey } from './types';

const videoExtensions = ['mp4', 'm4v', 'mov', 'mkv', 'webm', 'ts', 'm2ts', 'mpeg', 'mpg'];

const operations = [
  { id: 'convert', label: 'page.operations.convert' },
  { id: 'cut', label: 'page.operations.cut' },
  { id: 'merge', label: 'page.operations.merge' },
  { id: 'append', label: 'page.operations.append' }
] as const;

const startLabel: Record<VideoOp, string> = {
  convert: 'page.start.convert',
  cut: 'page.start.cut',
  merge: 'page.start.merge',
  append: 'page.start.append'
};

const hint = 'page.hint';

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

export function VideoPage() {
  const mobile = useAppStore((state) => state.isMobile());
  if (mobile) {
    return <section className={styles.page} />;
  }
  return <VideoWorkspace />;
}

function VideoWorkspace() {
  const { t } = useTranslation('video');
  const queryClient = useQueryClient();
  const [op, setOp] = useState<VideoOp>('convert');
  const [mode, setMode] = useState<'default' | 'custom'>('default');
  const [draft, setDraft] = useState(initialEncodeDraft);
  const [files, setFiles] = useState<VideoFile[]>([]);
  const [output, setOutput] = useState('');
  const [outputEdited, setOutputEdited] = useState(false);
  const [drawFilename, setDrawFilename] = useState(false);
  const [formError, setFormError] = useState<FormError | null>(null);
  const [tab, setTab] = useState<VideoTab>('run');
  const extension = outputExtension(mode, draft.container, draft.codec);
  const perFile = op === 'convert' || op === 'cut';
  const anchor = files.find((file) => file.checked)?.input ?? files[0]?.input;
  const tracks =
    files.find((file) => file.checked && file.info)?.info?.audioTracks ??
    files.find((file) => file.info)?.info?.audioTracks ??
    [];

  useEffect(() => {
    if (!perFile) {
      return;
    }
    const suffix = op === 'cut' ? '_cut' : '_converted';
    setFiles((current) => {
      let changed = false;
      const next = current.map((file) => {
        if (file.edited) {
          return file;
        }
        const nextOutput = videoOutput(file.input, suffix, extension);
        if (nextOutput === file.output) {
          return file;
        }
        changed = true;
        return { ...file, output: nextOutput };
      });
      return changed ? next : current;
    });
  }, [extension, op, perFile]);

  useEffect(() => {
    if (perFile || outputEdited || !anchor) {
      return;
    }
    const suffix = op === 'merge' ? '_merged' : '_appended';
    setOutput(videoOutput(anchor, suffix, extension));
  }, [anchor, extension, op, outputEdited, perFile]);

  function watch(file: VideoFile) {
    void ffmpegProbe(file.input)
      .then((info) => {
        setFiles((current) =>
          current.map((item) => (item.key === file.key ? { ...item, info, probeError: null } : item))
        );
      })
      .catch((error: unknown) => {
        setFiles((current) =>
          current.map((item) => (item.key === file.key ? { ...item, probeError: errorMessage(error) } : item))
        );
      });
  }

  function rememberSizes(inputs: string[]) {
    void fileSizes(inputs)
      .then((found) => {
        const sizes = new Map(found.map((item) => [item.path, item.size]));
        setFiles((current) =>
          current.map((file) => (sizes.has(file.input) ? { ...file, size: sizes.get(file.input) ?? null } : file))
        );
      })
      .catch(() => undefined);
  }

  function fileFrom(input: string, outputPath: string, segments: TimeSegment[]): VideoFile {
    const clips = segments.map((segment) => {
      const start = parseTime(segment.start);
      const clip = nextClip();
      return { ...clip, start, end: start + parseTime(segment.duration) };
    });
    return {
      key: nextFileKey(),
      input,
      output: outputPath,
      edited: true,
      checked: true,
      size: null,
      info: null,
      probeError: null,
      segments: clips.length > 0 ? clips : [nextClip()]
    };
  }

  function restore(task: Task) {
    const saved = task.draft;
    if (!saved) {
      return;
    }
    if (
      saved.kind !== 'convertVideo' &&
      saved.kind !== 'cutVideo' &&
      saved.kind !== 'mergeVideo' &&
      saved.kind !== 'appendVideo'
    ) {
      return;
    }
    const form = formFromEncode(saved.encode);
    setMode(form.mode);
    setDraft(form.draft);
    setDrawFilename(form.drawFilename);
    setFormError(null);
    if (saved.kind === 'convertVideo' || saved.kind === 'cutVideo') {
      setOp(saved.kind === 'cutVideo' ? 'cut' : 'convert');
      const file = fileFrom(saved.input, saved.output, saved.kind === 'cutVideo' ? saved.segments : []);
      setFiles([file]);
      setOutput('');
      setOutputEdited(false);
      watch(file);
      rememberSizes([saved.input]);
      return;
    }
    const inputs = saved.kind === 'mergeVideo' ? saved.inputs : [saved.base, ...saved.inputs];
    const next = inputs.map((input) => fileFrom(input, '', []));
    setOp(saved.kind === 'mergeVideo' ? 'merge' : 'append');
    setFiles(next);
    setOutput(saved.output);
    setOutputEdited(true);
    next.forEach(watch);
    rememberSizes(inputs);
  }

  async function addFiles(paths: string[]) {
    const known = new Set(files.map((file) => file.input));
    const fresh = paths.filter((path) => path.length > 0 && !known.has(path));
    if (fresh.length === 0) {
      return;
    }
    const sizes = new Map<string, number>();
    try {
      const found = await fileSizes(fresh);
      for (const item of found) {
        sizes.set(item.path, item.size);
      }
    } catch {
      sizes.clear();
    }
    const suffix = op === 'cut' ? '_cut' : '_converted';
    const added = fresh.map((input) => ({
      key: nextFileKey(),
      input,
      output: videoOutput(input, suffix, extension),
      edited: false,
      checked: true,
      size: sizes.get(input) ?? null,
      info: null,
      probeError: null,
      segments: [nextClip()]
    }));
    setFiles((current) => {
      const present = new Set(current.map((file) => file.input));
      const next = added.filter((file) => !present.has(file.input));
      return next.length === 0 ? current : [...current, ...next];
    });
    setFormError(null);
    added.forEach(watch);
  }

  function encode(): VideoEncode | undefined {
    if (mode === 'default') {
      if (!perFile && drawFilename) {
        return { title: { source: 'filename', text: '', anchor: 'topStart', fontSize: 0 } };
      }
      return undefined;
    }
    return buildEncode(draft, perFile);
  }

  const enqueue = useMutation({
    mutationFn: async () => {
      const selected = files.filter((file) => file.checked);
      if (files.length === 0) {
        throw new ValidationError('page.errors.noFiles');
      }
      if (selected.length === 0) {
        throw new ValidationError('page.errors.noSelection');
      }
      const profile = encode();
      if (op === 'convert') {
        for (const file of selected) {
          if (!file.output.trim()) {
            throw new ValidationError('page.errors.outputRequired');
          }
          await taskEnqueue({ kind: 'convertVideo', input: file.input, output: file.output.trim(), encode: profile });
        }
        return;
      }
      if (op === 'cut') {
        for (const file of selected) {
          if (!file.output.trim() || file.segments.length === 0) {
            throw new ValidationError('page.errors.clipOutputRequired');
          }
          if (file.segments.some((segment) => segment.end <= segment.start)) {
            throw new ValidationError('page.errors.clipRange');
          }
          const segments: TimeSegment[] = file.segments.map(({ start, end }) => ({
            start: formatTime(start),
            duration: formatTime(end - start)
          }));
          await taskEnqueue({
            kind: 'cutVideo',
            input: file.input,
            output: file.output.trim(),
            segments,
            encode: profile
          });
        }
        return;
      }
      if (selected.length < 2) {
        throw new ValidationError(op === 'merge' ? 'page.errors.mergeMin' : 'page.errors.appendMin');
      }
      if (!output.trim()) {
        throw new ValidationError('page.errors.outputRequired');
      }
      if (op === 'merge') {
        await taskEnqueue({
          kind: 'mergeVideo',
          inputs: selected.map((file) => file.input),
          output: output.trim(),
          encode: profile
        });
        return;
      }
      const [base, ...rest] = selected;
      if (!base) {
        throw new ValidationError('page.errors.appendMin');
      }
      await taskEnqueue({
        kind: 'appendVideo',
        base: base.input,
        inputs: rest.map((file) => file.input),
        output: output.trim(),
        encode: profile
      });
    },
    onSuccess: async () => {
      setFiles((current) => current.filter((file) => !file.checked));
      setOutput('');
      setOutputEdited(false);
      setFormError(null);
      setTab('wait');
      await refreshTasks(queryClient);
    },
    onError: (error) => {
      setFormError(error instanceof ValidationError ? { key: error.key } : { text: errorMessage(error) });
    }
  });

  function moveFile(index: number, delta: number) {
    setFiles((current) => {
      const target = index + delta;
      if (target < 0 || target >= current.length) {
        return current;
      }
      const next = current.slice();
      const [item] = next.splice(index, 1);
      if (!item) {
        return current;
      }
      next.splice(target, 0, item);
      return next;
    });
  }

  return (
    <section className={styles.page}>
      <div className={styles.workspace}>
        <div className={styles.column}>
          <article className={styles.formatCard}>
            <div className={styles.formatRow}>
              <div className={styles.group} role="tablist" aria-label={t('page.processingMode')}>
                {operations.map((item) => (
                  <SegmentButton key={item.id} selected={op === item.id} onClick={() => setOp(item.id)}>
                    {t(item.label)}
                  </SegmentButton>
                ))}
              </div>
              <InfoTip label={t('page.processingHint')} title={t(hint)} />
              <div className={styles.group} role="tablist" aria-label={t('page.encodeMode')}>
                <SegmentButton selected={mode === 'default'} onClick={() => setMode('default')}>
                  {t('page.modeDefault')}
                </SegmentButton>
                <SegmentButton selected={mode === 'custom'} onClick={() => setMode('custom')}>
                  {t('page.modeCustom')}
                </SegmentButton>
              </div>
            </div>
            <Scroll className={styles.formatRest} axis="vertical" gutter>
              {mode === 'default' ? (
                <p className={styles.defaults}>{t('page.defaultSummary', { summary: defaultEncodeSummary() })}</p>
              ) : (
                <EncodePanel draft={draft} perFile={perFile} tracks={tracks} onChange={setDraft} />
              )}
              {op === 'convert' || op === 'cut' ? (
                <QualityPreview
                  path={files.find((file) => file.checked)?.input ?? files[0]?.input ?? null}
                  encode={encode()}
                />
              ) : null}
            </Scroll>
          </article>
          <VideoQueue tab={tab} onTab={setTab} onEdit={restore} />
        </div>
        <article className={styles.fileCard}>
          <div className={styles.fileTools}>
            <FileDrop accept={videoExtensions} onFiles={(paths) => void addFiles(paths)} />
          </div>
          {perFile ? null : (
            <div className={styles.output}>
              {mode === 'default' ? (
                <Checkbox
                  label={t('page.drawFilename')}
                  checked={drawFilename}
                  onChange={(event) => setDrawFilename(event.target.checked)}
                />
              ) : null}
              <TextField
                label={t('page.outputPath')}
                value={output}
                required
                onChange={(event) => {
                  setOutput(event.target.value);
                  setOutputEdited(true);
                  if (formError && 'key' in formError && formError.key === 'page.errors.outputRequired') {
                    setFormError(null);
                  }
                }}
              />
              <p className={styles.hint}>{op === 'append' ? t('page.appendHint') : t('page.mergeHint')}</p>
            </div>
          )}
          <FileList
            files={files}
            op={op}
            perFile={perFile}
            onChecked={(key, checked) => {
              setFiles((current) => current.map((file) => (file.key === key ? { ...file, checked } : file)));
              if (formError && 'key' in formError && formError.key === 'page.errors.noSelection') {
                setFormError(null);
              }
            }}
            onChange={(next) => setFiles((current) => current.map((file) => (file.key === next.key ? next : file)))}
            onMove={moveFile}
            onRemove={(key) => setFiles((current) => current.filter((file) => file.key !== key))}
            onSelectAll={() => {
              setFiles((current) => current.map((file) => ({ ...file, checked: true })));
              setFormError(null);
            }}
            onInvert={() => setFiles((current) => current.map((file) => ({ ...file, checked: !file.checked })))}
            onClear={() => setFiles([])}
            onDeleteSelected={() => setFiles((current) => current.filter((file) => !file.checked))}
            start={
              <>
                {formError ? (
                  <p className={styles.error}>{'key' in formError ? t(formError.key) : formError.text}</p>
                ) : null}
                <Button disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
                  {t(startLabel[op])}
                </Button>
              </>
            }
          />
        </article>
      </div>
    </section>
  );
}
