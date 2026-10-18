import {
  CaretDownIcon,
  CaretUpIcon,
  FilmStripIcon,
  PlusIcon,
  ScissorsIcon,
  SlidersHorizontalIcon,
  StackIcon,
  TrashIcon
} from '@phosphor-icons/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { ffmpegProbe, taskEnqueue } from '@/bridge';
import type { EncodeSpeed, TimeSegment, VideoCodec, VideoContainer, VideoEncode, VideoInfo } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import { Checkbox } from '@/components/checkbox';
import { FileDrop } from '@/components/file-drop';
import styles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import { TimeField } from '@/components/time-field';
import { formatTime } from '@/components/time-field/format';
import { refreshTasks } from '@/query/task/query';
import { useAppStore } from '@/store/app';
import { errorMessage } from '@/utils/error';
import { TaskQueue } from './components/task-queue';
import { containerExtension, withSuffix } from './path';

const videoExtensions = ['mp4', 'm4v', 'mov', 'mkv', 'webm', 'ts', 'm2ts', 'mpeg', 'mpg'];

const operations = [
  { id: 'convert', label: '转码', Icon: FilmStripIcon },
  { id: 'cut', label: '裁剪', Icon: ScissorsIcon },
  { id: 'merge', label: '合并', Icon: StackIcon },
  { id: 'append', label: '追加', Icon: PlusIcon }
] as const;

type VideoOp = (typeof operations)[number]['id'];

type Clip = {
  key: string;
  start: number;
  end: number;
};

type VideoFile = {
  key: string;
  input: string;
  output: string;
  edited: boolean;
  info: VideoInfo | null;
  probeError: string | null;
  segments: Clip[];
};

const hints: Record<VideoOp, string> = {
  convert: '默认转成 AV1 的 MP4。自定义可以选择容器、编码和速度。',
  cut: '按开始和结束时间裁出片段，再合成一个视频。',
  merge: '合成一个视频。尺寸取最大，帧率取中位数。',
  append: '第一段作为基准，后面的对齐后按顺序接上。'
};

let fileKey = 0;

function nextFileKey() {
  fileKey += 1;
  return `video-${fileKey}`;
}

function nextClip(previous?: Clip): Clip {
  fileKey += 1;
  const start = previous ? previous.end + 5 : 0;
  return { key: `clip-${fileKey}`, start, end: start + 5 };
}

function formatDuration(seconds: number) {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return '时长未知';
  }
  const total = Math.round(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remain = total % 60;
  const clock = `${String(minutes).padStart(2, '0')}:${String(remain).padStart(2, '0')}`;
  return hours > 0 ? `${hours}:${clock}` : clock;
}

function optionalInt(value: string, max: number) {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed)) {
    return null;
  }
  return Math.min(max, Math.max(0, Math.round(parsed)));
}

function positiveInt(value: string, max: number) {
  const next = optionalInt(value, max);
  return next != null && next > 0 ? next : null;
}

function outputExtension(mode: 'default' | 'custom', container: VideoContainer) {
  return mode === 'custom' ? containerExtension(container) : '.mp4';
}

export function VideoToolboxPage() {
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
  return <VideoToolbox />;
}

function VideoToolbox() {
  const queryClient = useQueryClient();
  const [op, setOp] = useState<VideoOp>('convert');
  const [mode, setMode] = useState<'default' | 'custom'>('default');
  const [container, setContainer] = useState<VideoContainer>('mp4');
  const [codec, setCodec] = useState<VideoCodec>('av1');
  const [speed, setSpeed] = useState<EncodeSpeed>('balanced');
  const [crf, setCrf] = useState('');
  const [fps, setFps] = useState('');
  const [files, setFiles] = useState<VideoFile[]>([]);
  const [output, setOutput] = useState('');
  const [outputEdited, setOutputEdited] = useState(false);
  const [drawFilename, setDrawFilename] = useState(false);
  const [formError, setFormError] = useState('');
  const extension = outputExtension(mode, container);
  const firstInput = files[0]?.input;
  const perFile = op === 'convert' || op === 'cut';

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
        const nextOutput = withSuffix(file.input, suffix, extension);
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
    setOutput(withSuffix(firstInput, suffix, extension));
  }, [extension, firstInput, op, outputEdited, perFile]);

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
      output: withSuffix(input, suffix, extension),
      edited: false,
      info: null,
      probeError: null,
      segments: [nextClip()]
    }));
    setFiles((current) => [...current, ...added]);
    setFormError('');
    for (const item of added) {
      void ffmpegProbe(item.input)
        .then((info) => {
          setFiles((current) =>
            current.map((file) => (file.key === item.key ? { ...file, info, probeError: null } : file))
          );
        })
        .catch((error: unknown) => {
          setFiles((current) =>
            current.map((file) => (file.key === item.key ? { ...file, probeError: errorMessage(error) } : file))
          );
        });
    }
  }

  function encode(): VideoEncode | undefined {
    if (mode === 'default') {
      return undefined;
    }
    return {
      container,
      codec,
      speed,
      crf: optionalInt(crf, 63),
      fps: perFile ? positiveInt(fps, 240) : null
    };
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
          drawFilename,
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
        drawFilename,
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
      next.splice(target, 0, item);
      return next;
    });
  }

  return (
    <section className={styles.page}>
      <header className={styles.head}>
        <h1 className={styles.title}>视频处理</h1>
        <p className={styles.hint}>{hints[op]}</p>
        <div className={styles.segments} role="tablist" aria-label="处理方式">
          {operations.map((item) => (
            <SegmentButton key={item.id} selected={op === item.id} onClick={() => setOp(item.id)}>
              <item.Icon weight="regular" />
              {item.label}
            </SegmentButton>
          ))}
        </div>
        <div className={styles.segments} role="tablist" aria-label="编码模式">
          <SegmentButton selected={mode === 'default'} onClick={() => setMode('default')}>
            默认
          </SegmentButton>
          <SegmentButton selected={mode === 'custom'} onClick={() => setMode('custom')}>
            <SlidersHorizontalIcon weight="regular" />
            自定义
          </SegmentButton>
        </div>
        <Button className={styles.trailing} disabled={enqueue.isPending} onClick={() => enqueue.mutate()}>
          加入队列
        </Button>
      </header>
      <Scroll className={styles.scroll}>
        <div className={styles.body}>
          <div className={styles.card}>
            <FileDrop accept={videoExtensions} onFiles={addFiles} />
            {mode === 'custom' ? (
              <>
                <div className={styles.pair}>
                  <Select
                    label="输出格式"
                    value={container}
                    onChange={(event) => setContainer(event.target.value as VideoContainer)}
                  >
                    <option value="mp4">MP4</option>
                    <option value="mov">MOV</option>
                    <option value="mpegTs">MPEG-TS</option>
                  </Select>
                  <Select
                    label="视频编码"
                    value={codec}
                    onChange={(event) => setCodec(event.target.value as VideoCodec)}
                  >
                    <option value="av1">AV1</option>
                    <option value="hevc">HEVC</option>
                    <option value="vp9">VP9</option>
                  </Select>
                </div>
                <div className={styles.pair}>
                  <Select
                    label="处理速度"
                    value={speed}
                    onChange={(event) => setSpeed(event.target.value as EncodeSpeed)}
                  >
                    <option value="fast">快</option>
                    <option value="balanced">均衡</option>
                    <option value="slow">慢</option>
                  </Select>
                  <TextField
                    label="质量 CRF"
                    value={crf}
                    inputMode="numeric"
                    hint="留空使用默认"
                    onChange={(event) => setCrf(event.target.value)}
                  />
                </div>
                {perFile ? (
                  <TextField
                    label="帧率"
                    value={fps}
                    inputMode="numeric"
                    hint="留空保留源帧率"
                    onChange={(event) => setFps(event.target.value)}
                  />
                ) : null}
              </>
            ) : null}
            {!perFile ? (
              <>
                <TextField
                  label="输出路径"
                  value={output}
                  required
                  onChange={(event) => {
                    setOutput(event.target.value);
                    setOutputEdited(true);
                  }}
                />
                <Checkbox
                  label="在画面上写文件名"
                  checked={drawFilename}
                  onChange={(event) => setDrawFilename(event.target.checked)}
                />
              </>
            ) : null}
            {formError ? <p className={styles.error}>{formError}</p> : null}
          </div>
          {files.map((file, index) => (
            <div key={file.key} className={styles.card}>
              <h2 className={styles.cardTitle}>
                {op === 'append' && index === 0 ? '基准' : file.input.split(/[/\\]/).pop()}
                <span className={styles.extra}>{file.info ? `${file.info.width}×${file.info.height}` : null}</span>
              </h2>
              <p className={styles.meta}>{file.info ? formatDuration(file.info.duration) : file.input}</p>
              {file.probeError ? <p className={styles.error}>{file.probeError}</p> : null}
              {perFile ? (
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
              ) : null}
              {op === 'cut'
                ? file.segments.map((segment) => (
                    <div key={segment.key} className={styles.stack}>
                      <div className={styles.pair}>
                        <TimeField
                          label="开始"
                          value={segment.start}
                          onChange={(start) => {
                            setFiles((current) =>
                              current.map((item) =>
                                item.key === file.key
                                  ? {
                                      ...item,
                                      segments: item.segments.map((part) =>
                                        part.key === segment.key ? { ...part, start } : part
                                      )
                                    }
                                  : item
                              )
                            );
                          }}
                        />
                        <TimeField
                          label="结束"
                          value={segment.end}
                          error={segment.end <= segment.start ? '结束要晚于开始' : undefined}
                          onChange={(end) => {
                            setFiles((current) =>
                              current.map((item) =>
                                item.key === file.key
                                  ? {
                                      ...item,
                                      segments: item.segments.map((part) =>
                                        part.key === segment.key ? { ...part, end } : part
                                      )
                                    }
                                  : item
                              )
                            );
                          }}
                        />
                      </div>
                      {file.segments.length > 1 ? (
                        <Button
                          variant="text"
                          onClick={() => {
                            setFiles((current) =>
                              current.map((item) =>
                                item.key === file.key
                                  ? {
                                      ...item,
                                      segments: item.segments.filter((part) => part.key !== segment.key)
                                    }
                                  : item
                              )
                            );
                          }}
                        >
                          删除这段
                        </Button>
                      ) : null}
                    </div>
                  ))
                : null}
              <div className={styles.actions}>
                {op === 'cut' ? (
                  <Button
                    variant="text"
                    onClick={() => {
                      setFiles((current) =>
                        current.map((item) =>
                          item.key === file.key
                            ? { ...item, segments: [...item.segments, nextClip(item.segments.at(-1))] }
                            : item
                        )
                      );
                    }}
                  >
                    添加片段
                  </Button>
                ) : null}
                {!perFile ? (
                  <>
                    <Button variant="text" disabled={index === 0} onClick={() => moveFile(index, -1)}>
                      <CaretUpIcon weight="regular" />
                      上移
                    </Button>
                    <Button variant="text" disabled={index === files.length - 1} onClick={() => moveFile(index, 1)}>
                      <CaretDownIcon weight="regular" />
                      下移
                    </Button>
                  </>
                ) : null}
                <Button
                  variant="text"
                  tone="destructive"
                  className={styles.trailing}
                  onClick={() => setFiles((current) => current.filter((item) => item.key !== file.key))}
                >
                  <TrashIcon weight="regular" />
                  移除
                </Button>
              </div>
            </div>
          ))}
          <TaskQueue kinds={['convertVideo', 'cutVideo', 'mergeVideo', 'appendVideo']} />
        </div>
      </Scroll>
    </section>
  );
}
