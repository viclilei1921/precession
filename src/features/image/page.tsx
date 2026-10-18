import { FileImageIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
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
import { FormatFields } from './components/format-fields';
import styles from './page.module.css';
import { imageOutput } from './path';

const imageExtensions = ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp', 'tif', 'tiff'];

type ImageFormat = 'avif' | 'jxl';

let fileKey = 0;

function nextFileKey() {
  fileKey += 1;
  return `image-${fileKey}`;
}

function numberOr(value: string, fallback: number) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
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
  const [quality, setQuality] = useState('60');
  const [speed, setSpeed] = useState('6');
  const [files, setFiles] = useState<ImageFile[]>([]);
  const [formError, setFormError] = useState('');
  const keepFormat = useRef(false);
  const extension = format === 'avif' ? '.avif' : '.jxl';

  useEffect(() => {
    if (keepFormat.current) {
      keepFormat.current = false;
      return;
    }
    setQuality(format === 'avif' ? '60' : '1');
    setSpeed(format === 'avif' ? '6' : '7');
  }, [format]);

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
    const draft = task.draft;
    if (!draft || (draft.kind !== 'convertAvif' && draft.kind !== 'convertJxl')) {
      return;
    }
    const next = draft.kind === 'convertAvif' ? 'avif' : 'jxl';
    if (next !== format) {
      keepFormat.current = true;
    }
    setFormat(next);
    if (draft.kind === 'convertAvif') {
      setQuality(String(draft.params.qcolor ?? 60));
      setSpeed(String(draft.params.speed ?? 6));
    } else {
      setQuality(String(draft.params.quality ?? 1));
      setSpeed(String(draft.params.effort ?? 7));
    }
    setFiles([{ key: nextFileKey(), input: draft.input, output: draft.output, edited: true }]);
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
          const qcolor = Math.min(100, Math.max(0, Math.round(numberOr(quality, 60))));
          await taskEnqueue({
            kind: 'convertAvif',
            input: file.input,
            output: file.output.trim(),
            params: {
              qcolor,
              qalpha: qcolor,
              speed: Math.min(10, Math.max(0, Math.round(numberOr(speed, 6))))
            }
          });
        } else {
          await taskEnqueue({
            kind: 'convertJxl',
            input: file.input,
            output: file.output.trim(),
            params: {
              quality: Math.max(0, numberOr(quality, 1)),
              effort: Math.min(10, Math.max(1, Math.round(numberOr(speed, 7))))
            }
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
        <p className={styles.hint}>
          {format === 'avif'
            ? '质量 60、速度 6 是常用起点。速度越高编码越快。'
            : '距离 1、effort 7 是常用起点。距离越大画面越糊。'}
        </p>
        <Button disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
          加入队列
        </Button>
      </header>
      <div className={styles.workspace}>
        <div className={styles.main}>
          <Scroll className={styles.scroll}>
            <div className={styles.stage}>
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
              <Card>
                <FileDrop accept={imageExtensions} onFiles={addFiles} />
              </Card>
              <FormatFields format={format} quality={quality} speed={speed} onQuality={setQuality} onSpeed={setSpeed} />
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
