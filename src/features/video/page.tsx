import { FilmStripIcon, PlusIcon, ScissorsIcon, SlidersHorizontalIcon, StackIcon } from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { ffmpegProbe, taskEnqueue } from '@/bridge';
import type { Task, TimeSegment, VideoEncode } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Card } from '@/components/card';
import { Checkbox } from '@/components/checkbox';
import { FileDrop } from '@/components/file-drop';
import { Scroll } from '@/components/scroll';
import { TextField } from '@/components/text-field';
import { formatTime, parseTime } from '@/components/time-field/format';
import { TaskQueue } from '@/features/task/task-queue';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { EncodePanel } from './components/encode-panel';
import { VideoCard } from './components/video-card';
import { buildEncode, formFromEncode } from './encode';
import { defaultEncodeSummary, initialEncodeDraft } from './encode-draft';
import styles from './page.module.css';
import { outputExtension, videoOutput } from './path';
import type { VideoFile, VideoOp } from './types';
import { nextClip, nextFileKey } from './types';

const videoExtensions = ['mp4', 'm4v', 'mov', 'mkv', 'webm', 'ts', 'm2ts', 'mpeg', 'mpg'];

const operations = [
  { id: 'convert', label: '转码', Icon: FilmStripIcon },
  { id: 'cut', label: '裁剪', Icon: ScissorsIcon },
  { id: 'merge', label: '合并', Icon: StackIcon },
  { id: 'append', label: '追加', Icon: PlusIcon }
] as const;

const hints: Record<VideoOp, string> = {
  convert: '默认导出 MP4，AV1，速度 6，质量 35，10-bit。',
  cut: '按开始和结束时间裁出片段，再合成一个视频。',
  merge: '合成一个视频。尺寸取最大，帧率取中位数。',
  append: '第一段作为基准。编码对不齐时，基准也会重编码。'
};

export function VideoPage() {
  const mobile = useAppStore((state) => state.isMobile());
  if (mobile) {
    return (
      <section className={styles.page}>
        <header className={styles.head}>
          <h1 className={styles.title}>视频处理</h1>
          <p className={styles.hint}>视频处理只在桌面使用。</p>
        </header>
      </section>
    );
  }
  return <VideoWorkspace />;
}

function VideoWorkspace() {
  const queryClient = useQueryClient();
  const [op, setOp] = useState<VideoOp>('convert');
  const [mode, setMode] = useState<'default' | 'custom'>('default');
  const [draft, setDraft] = useState(initialEncodeDraft);
  const [files, setFiles] = useState<VideoFile[]>([]);
  const [output, setOutput] = useState('');
  const [outputEdited, setOutputEdited] = useState(false);
  const [drawFilename, setDrawFilename] = useState(false);
  const [formError, setFormError] = useState('');
  const extension = outputExtension(mode, draft.container, draft.codec);
  const firstInput = files[0]?.input;
  const perFile = op === 'convert' || op === 'cut';
  const tracks = files.find((file) => file.info)?.info?.audioTracks ?? [];

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
    if (perFile || outputEdited || !firstInput) {
      return;
    }
    const suffix = op === 'merge' ? '_merged' : '_appended';
    setOutput(videoOutput(firstInput, suffix, extension));
  }, [extension, firstInput, op, outputEdited, perFile]);

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

  function fileFrom(input: string, output: string, segments: TimeSegment[]): VideoFile {
    const clips = segments.map((segment) => {
      const start = parseTime(segment.start);
      const clip = nextClip();
      return { ...clip, start, end: start + parseTime(segment.duration) };
    });
    return {
      key: nextFileKey(),
      input,
      output,
      edited: true,
      info: null,
      probeError: null,
      segments: clips.length > 0 ? clips : [nextClip()]
    };
  }

  function restore(task: Task) {
    const draft = task.draft;
    if (!draft) {
      return;
    }
    if (
      draft.kind !== 'convertVideo' &&
      draft.kind !== 'cutVideo' &&
      draft.kind !== 'mergeVideo' &&
      draft.kind !== 'appendVideo'
    ) {
      return;
    }
    const form = formFromEncode(draft.encode);
    setMode(form.mode);
    setDraft(form.draft);
    setDrawFilename(form.drawFilename);
    setFormError('');
    if (draft.kind === 'convertVideo' || draft.kind === 'cutVideo') {
      setOp(draft.kind === 'cutVideo' ? 'cut' : 'convert');
      const file = fileFrom(draft.input, draft.output, draft.kind === 'cutVideo' ? draft.segments : []);
      setFiles([file]);
      setOutput('');
      setOutputEdited(false);
      watch(file);
      return;
    }
    const inputs = draft.kind === 'mergeVideo' ? draft.inputs : [draft.base, ...draft.inputs];
    const next = inputs.map((input) => fileFrom(input, '', []));
    setOp(draft.kind === 'mergeVideo' ? 'merge' : 'append');
    setFiles(next);
    setOutput(draft.output);
    setOutputEdited(true);
    next.forEach(watch);
  }

  function addFiles(paths: string[]) {
    const known = new Set(files.map((file) => file.input));
    const fresh = paths.filter((path) => !known.has(path));
    if (fresh.length === 0) {
      return;
    }
    const suffix = op === 'cut' ? '_cut' : '_converted';
    const added = fresh.map((input) => ({
      key: nextFileKey(),
      input,
      output: videoOutput(input, suffix, extension),
      edited: false,
      info: null,
      probeError: null,
      segments: [nextClip()]
    }));
    setFiles((current) => [...current, ...added]);
    setFormError('');
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
      if (files.length === 0) {
        throw new Error('请先添加视频。');
      }
      const profile = encode();
      if (op === 'convert') {
        for (const file of files) {
          if (!file.output.trim()) {
            throw new Error('请填写输出路径。');
          }
          await taskEnqueue({ kind: 'convertVideo', input: file.input, output: file.output.trim(), encode: profile });
        }
        return;
      }
      if (op === 'cut') {
        for (const file of files) {
          if (!file.output.trim() || file.segments.length === 0) {
            throw new Error('每个视频都要有输出路径和至少一个片段。');
          }
          if (file.segments.some((segment) => segment.end <= segment.start)) {
            throw new Error('片段的结束时间要晚于开始时间。');
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
      if (files.length < 2) {
        throw new Error(op === 'merge' ? '合并至少需要两个视频。' : '追加至少需要基准和一段新视频。');
      }
      if (!output.trim()) {
        throw new Error('请填写输出路径。');
      }
      if (op === 'merge') {
        await taskEnqueue({
          kind: 'mergeVideo',
          inputs: files.map((file) => file.input),
          output: output.trim(),
          encode: profile
        });
        return;
      }
      const [base, ...rest] = files;
      if (!base) {
        throw new Error('追加至少需要基准和一段新视频。');
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
      setFiles([]);
      setOutput('');
      setOutputEdited(false);
      setFormError('');
      await refreshTasks(queryClient);
    },
    onError: (error) => {
      setFormError(errorMessage(error));
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
      <header className={styles.head}>
        <h1 className={styles.title}>视频处理</h1>
        <p className={styles.hint}>{hints[op]}</p>
        <Button disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
          加入队列
        </Button>
      </header>
      <div className={styles.workspace}>
        <div className={styles.main}>
          <Scroll className={styles.scroll}>
            <div className={styles.stage}>
              <div className={styles.segments}>
                <div className={styles.group} role="tablist" aria-label="处理方式">
                  {operations.map((item) => (
                    <SegmentButton key={item.id} selected={op === item.id} onClick={() => setOp(item.id)}>
                      <item.Icon weight="regular" />
                      {item.label}
                    </SegmentButton>
                  ))}
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
                <FileDrop accept={videoExtensions} onFiles={addFiles} />
              </Card>
              {mode === 'default' ? (
                <Card title="编码">
                  <p className={styles.summary}>{defaultEncodeSummary}</p>
                </Card>
              ) : (
                <EncodePanel draft={draft} perFile={perFile} tracks={tracks} onChange={setDraft} />
              )}
              {perFile ? null : (
                <Card title="输出">
                  <div className={styles.stack}>
                    <TextField
                      label="输出路径"
                      value={output}
                      required
                      onChange={(event) => {
                        setOutput(event.target.value);
                        setOutputEdited(true);
                      }}
                    />
                    {mode === 'default' ? (
                      <Checkbox
                        label="在画面上写文件名"
                        checked={drawFilename}
                        onChange={(event) => setDrawFilename(event.target.checked)}
                      />
                    ) : null}
                  </div>
                </Card>
              )}
              {formError ? <p className={styles.error}>{formError}</p> : null}
              {files.map((file, index) => (
                <VideoCard
                  key={file.key}
                  file={file}
                  op={op}
                  index={index}
                  total={files.length}
                  perFile={perFile}
                  onChange={(next) =>
                    setFiles((current) => current.map((item) => (item.key === file.key ? next : item)))
                  }
                  onMove={(delta) => moveFile(index, delta)}
                  onRemove={() => setFiles((current) => current.filter((item) => item.key !== file.key))}
                />
              ))}
            </div>
          </Scroll>
        </div>
        <aside className={styles.rail}>
          <Scroll className={styles.scroll}>
            <TaskQueue kinds={['convertVideo', 'cutVideo', 'mergeVideo', 'appendVideo']} onEdit={restore} />
          </Scroll>
        </aside>
      </div>
    </section>
  );
}
