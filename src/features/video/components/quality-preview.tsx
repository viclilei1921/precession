import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ffmpegPreviewCancel, ffmpegPreviewQuality } from '@/bridge';
import type { QualityPreview as QualityPreviewResult, VideoEncode } from '@/bridge/task';
import { Button } from '@/components/button';
import { formatTime } from '@/components/time-field/format';
import { formatBytes } from '@/features/settings/bytes';
import { errorMessage, rawErrorMessage } from '@/utils/error';
import styles from './quality-preview.module.css';

type QualityPreviewProps = {
  path: string | null;
  encode: VideoEncode | undefined;
};

function fileName(path: string) {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

function shortSeconds(value: number) {
  const rounded = Math.round(value * 10) / 10;
  return Number.isInteger(rounded) ? String(rounded) : rounded.toFixed(1);
}

let stopQueue = Promise.resolve();

/** 等上一次取消落到后端之后，再允许开始新的预估。 */
function stopPreview() {
  stopQueue = stopQueue
    .then(() => ffmpegPreviewCancel())
    .then(
      () => undefined,
      () => undefined
    );
  return stopQueue;
}

function bandLabel(band: QualityPreviewResult['band'], t: ReturnType<typeof useTranslation<'video'>>['t']) {
  switch (band) {
    case 'indistinguishable':
      return t('preview.bands.indistinguishable');
    case 'same':
      return t('preview.bands.same');
    case 'slight':
      return t('preview.bands.slight');
    case 'visible':
      return t('preview.bands.visible');
    case 'degraded':
      return t('preview.bands.degraded');
  }
}

/** 用当前画面参数预估选中视频中间一段的画质。 */
export function QualityPreview({ path, encode }: QualityPreviewProps) {
  const { t } = useTranslation('video');
  const signature = JSON.stringify({ path, encode: encode ?? null });
  const seen = useRef(signature);
  const token = useRef(0);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<QualityPreviewResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (seen.current === signature) {
      return;
    }
    seen.current = signature;
    token.current += 1;
    setRunning(false);
    setResult(null);
    setError(null);
    void stopPreview();
  }, [signature]);

  useEffect(
    () => () => {
      void stopPreview();
    },
    []
  );

  async function onClick() {
    if (running) {
      token.current += 1;
      await stopPreview();
      setRunning(false);
      return;
    }
    if (!path) {
      return;
    }
    await stopQueue;
    const current = token.current + 1;
    token.current = current;
    setRunning(true);
    setError(null);
    try {
      const next = await ffmpegPreviewQuality(path, encode);
      if (token.current !== current) {
        return;
      }
      setResult(next);
    } catch (caught) {
      if (token.current !== current || rawErrorMessage(caught).trim() === '已取消') {
        return;
      }
      setError(errorMessage(caught));
    } finally {
      if (token.current === current) {
        setRunning(false);
      }
    }
  }

  return (
    <div className={styles.preview}>
      <Button variant="outlined" disabled={!path} onClick={() => void onClick()}>
        {running ? t('preview.cancel') : t('preview.action')}
      </Button>
      {error ? <p className={styles.error}>{error}</p> : null}
      {result && path ? (
        <div className={styles.result}>
          <p className={styles.lead}>{bandLabel(result.band, t)}</p>
          <p className={styles.meta}>
            {result.vmaf != null ? t('preview.vmaf', { score: result.vmaf.toFixed(1) }) : t('preview.vmafMissing')}
            {result.ssim != null ? ` · ${t('preview.ssim', { score: result.ssim.toFixed(3) })}` : ''}
          </p>
          <p className={styles.note}>{t('preview.file', { name: fileName(path) })}</p>
          <p className={styles.note}>
            {result.whole
              ? t('preview.whole')
              : t('preview.window', {
                  start: formatTime(result.start).slice(0, 8),
                  seconds: shortSeconds(result.seconds)
                })}
          </p>
          <p className={styles.note}>{t('preview.size', { size: formatBytes(result.estimatedBytes) })}</p>
          <p className={styles.note}>{t('preview.scope')}</p>
        </div>
      ) : null}
    </div>
  );
}
