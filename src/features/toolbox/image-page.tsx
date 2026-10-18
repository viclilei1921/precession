import { FileImageIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { taskEnqueue } from '@/bridge';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { FileDrop } from '@/components/file-drop';
import styles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { TaskQueue } from './components/task-queue';
import { imageOutput } from './path';

const imageExtensions = ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp', 'tif', 'tiff'];

type ImageFormat = 'avif' | 'jxl';

type ImageFile = {
  key: string;
  input: string;
  output: string;
  edited: boolean;
};

let fileKey = 0;

function nextFileKey() {
  fileKey += 1;
  return `image-${fileKey}`;
}

function numberOr(value: string, fallback: number) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}

export function ImageToolboxPage() {
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
  return <ImageToolbox />;
}

function ImageToolbox() {
  const queryClient = useQueryClient();
  const [format, setFormat] = useState<ImageFormat>('avif');
  const [quality, setQuality] = useState('60');
  const [speed, setSpeed] = useState('6');
  const [files, setFiles] = useState<ImageFile[]>([]);
  const [formError, setFormError] = useState('');
  const extension = format === 'avif' ? '.avif' : '.jxl';

  useEffect(() => {
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
        <div className={styles.segments} role="tablist" aria-label="图片格式">
          <SegmentButton selected={format === 'avif'} onClick={() => setFormat('avif')}>
            <FileImageIcon weight="regular" />
            AVIF
          </SegmentButton>
          <SegmentButton selected={format === 'jxl'} onClick={() => setFormat('jxl')}>
            <FileImageIcon weight="regular" />
            JXL
          </SegmentButton>
        </div>
        <Button className={styles.trailing} disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
          加入队列
        </Button>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          <div className={styles.card}>
            <FileDrop accept={imageExtensions} onFiles={addFiles} />
            <div className={styles.pair}>
              <TextField
                label={format === 'avif' ? '质量' : '距离'}
                value={quality}
                inputMode="decimal"
                onChange={(event) => setQuality(event.target.value)}
              />
              <TextField
                label={format === 'avif' ? '速度' : 'effort'}
                value={speed}
                inputMode="numeric"
                hint={format === 'avif' ? '0 到 10' : '1 到 10'}
                onChange={(event) => setSpeed(event.target.value)}
              />
            </div>
            {formError ? <p className={styles.error}>{formError}</p> : null}
          </div>
          {files.map((file) => (
            <div key={file.key} className={styles.card}>
              <h2 className={styles.cardTitle}>{file.input.split(/[/\\]/).pop()}</h2>
              <TextField
                label="输出路径"
                value={file.output}
                required
                onChange={(event) => {
                  const value = event.target.value;
                  setFiles((current) =>
                    current.map((item) => (item.key === file.key ? { ...item, output: value, edited: true } : item))
                  );
                }}
              />
              <Button
                variant="text"
                tone="destructive"
                onClick={() => setFiles((current) => current.filter((item) => item.key !== file.key))}
              >
                移除
              </Button>
            </div>
          ))}
          <TaskQueue kinds={['convertAvif', 'convertJxl']} />
        </div>
      </Scroll>
    </section>
  );
}
