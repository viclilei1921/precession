import { FileImageIcon, SlidersHorizontalIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { taskEnqueue } from '@/bridge';
import type { Task } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Card } from '@/components/card';
import { FileDrop } from '@/components/file-drop';
import { Scroll } from '@/components/scroll';
import { TaskQueue } from '@/features/task/task-queue';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import type { ImageFile } from './components/file-list';
import { FileList } from './components/file-list';
import { ParamsPanel } from './components/params-panel';
import styles from './page.module.css';
import { buildAvifParams, buildJxlParams, formFromParams } from './params';
import { defaultAvifSummary, defaultJxlSummary, initialImageDraft } from './params-draft';
import { imageOutput } from './path';

const imageExtensions = ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp', 'tif', 'tiff'];

type ImageFormat = 'avif' | 'jxl';

const hints: Record<ImageFormat, string> = {
  avif: '默认质量 60、速度 6、8-bit。质量越大越清楚，100 是无损。',
  jxl: '默认距离 1、effort 7。距离越小越接近原图，1 大约看不出差别，日常用 0.5 到 3。'
};

let fileKey = 0;

function nextFileKey() {
  fileKey += 1;
  return `image-${fileKey}`;
}

export function ImagePage() {
  const mobile = useAppStore((state) => state.isMobile());
  if (mobile) {
    return (
      <section className={styles.page}>
        <header className={styles.head}>
          <h1 className={styles.title}>图片转码</h1>
          <p className={styles.hint}>图片转码只在桌面使用。</p>
        </header>
      </section>
    );
  }
  return <ImageWorkspace />;
}

function ImageWorkspace() {
  const queryClient = useQueryClient();
  const [format, setFormat] = useState<ImageFormat>('avif');
  const [mode, setMode] = useState<'default' | 'custom'>('default');
  const [draft, setDraft] = useState(initialImageDraft);
  const [files, setFiles] = useState<ImageFile[]>([]);
  const [formError, setFormError] = useState('');
  const extension = format === 'avif' ? '.avif' : '.jxl';

  useEffect(() => {
    setFiles((current) => {
      let changed = false;
      const next = current.map((file) => {
        if (file.edited) {
          return file;
        }
        const output = imageOutput(file.input, extension);
        if (output === file.output) {
          return file;
        }
        changed = true;
        return { ...file, output };
      });
      return changed ? next : current;
    });
  }, [extension]);

  function restore(task: Task) {
    const saved = task.draft;
    if (!saved || (saved.kind !== 'convertAvif' && saved.kind !== 'convertJxl')) {
      return;
    }
    const form = formFromParams(saved.kind, saved.params);
    setFormat(saved.kind === 'convertAvif' ? 'avif' : 'jxl');
    setMode(form.mode);
    setDraft(form.draft);
    setFiles([{ key: nextFileKey(), input: saved.input, output: saved.output, edited: true }]);
    setFormError('');
  }

  function addFiles(paths: string[]) {
    const known = new Set(files.map((file) => file.input));
    const added = paths
      .filter((path) => !known.has(path))
      .map((input) => ({
        key: nextFileKey(),
        input,
        output: imageOutput(input, extension),
        edited: false
      }));
    if (added.length === 0) {
      return;
    }
    setFiles((current) => [...current, ...added]);
    setFormError('');
  }

  const enqueue = useMutation({
    mutationFn: async () => {
      if (files.length === 0) {
        throw new Error('请先添加图片。');
      }
      for (const file of files) {
        if (!file.output.trim()) {
          throw new Error('请填写输出路径。');
        }
        if (format === 'avif') {
          await taskEnqueue({
            kind: 'convertAvif',
            input: file.input,
            output: file.output.trim(),
            ...(mode === 'custom' ? { params: buildAvifParams(draft) } : {})
          });
        } else {
          await taskEnqueue({
            kind: 'convertJxl',
            input: file.input,
            output: file.output.trim(),
            ...(mode === 'custom' ? { params: buildJxlParams(draft) } : {})
          });
        }
      }
    },
    onSuccess: async () => {
      setFiles([]);
      setFormError('');
      await refreshTasks(queryClient);
    },
    onError: (error) => {
      setFormError(errorMessage(error));
    }
  });

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>图片转码</h1>
        <p className={styles.hint}>{hints[format]}</p>
        <Button disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
          加入队列
        </Button>
      </header>
      <div className={styles.workspace}>
        <div className={styles.main}>
          <Scroll className={styles.scroll}>
            <div className={styles.stage}>
              <div className={styles.segments}>
                <div className={styles.group} role="tablist" aria-label="图片格式">
                  <SegmentButton selected={format === 'avif'} onClick={() => setFormat('avif')}>
                    <FileImageIcon weight="regular" />
                    AVIF
                  </SegmentButton>
                  <SegmentButton selected={format === 'jxl'} onClick={() => setFormat('jxl')}>
                    <FileImageIcon weight="regular" />
                    JXL
                  </SegmentButton>
                </div>
                <div className={styles.group} role="tablist" aria-label="编码模式">
                  <SegmentButton selected={mode === 'default'} onClick={() => setMode('default')}>
                    默认
                  </SegmentButton>
                  <SegmentButton selected={mode === 'custom'} onClick={() => setMode('custom')}>
                    <SlidersHorizontalIcon weight="regular" />
                    自定义
                  </SegmentButton>
                </div>
              </div>
              <Card>
                <FileDrop accept={imageExtensions} onFiles={addFiles} />
              </Card>
              {mode === 'default' ? (
                <Card title="编码">
                  <p className={styles.summary}>{format === 'avif' ? defaultAvifSummary : defaultJxlSummary}</p>
                </Card>
              ) : (
                <ParamsPanel format={format} draft={draft} onChange={setDraft} />
              )}
              {formError ? <p className={styles.error}>{formError}</p> : null}
              <FileList
                files={files}
                onOutput={(key, value) =>
                  setFiles((current) =>
                    current.map((item) => (item.key === key ? { ...item, output: value, edited: true } : item))
                  )
                }
                onRemove={(key) => setFiles((current) => current.filter((item) => item.key !== key))}
              />
            </div>
          </Scroll>
        </div>
        <aside className={styles.rail}>
          <Scroll className={styles.scroll}>
            <TaskQueue kinds={['convertAvif', 'convertJxl']} onEdit={restore} />
          </Scroll>
        </aside>
      </div>
    </section>
  );
}
