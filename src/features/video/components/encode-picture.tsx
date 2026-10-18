import type { BitDepth, Fit, HevcPreset, HevcTune, VideoContainer, Vp9Content } from '@/bridge/task';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import type { EncodeDraft } from '../encode-draft';
import styles from './encode.module.css';

type EncodePictureProps = {
  draft: EncodeDraft;
  perFile: boolean;
  onChange: (draft: EncodeDraft) => void;
};

export function EncodePicture({ draft, perFile, onChange }: EncodePictureProps) {
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  const crfHint =
    draft.codec === 'av1'
      ? '0 最清晰，63 最小。默认 35'
      : draft.codec === 'hevc'
        ? '0 最清晰，51 最小。默认 26'
        : '0 最清晰，63 最小。默认 33';

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <Select
          label="输出格式"
          value={draft.container}
          onChange={(event) => patch({ container: event.target.value as VideoContainer })}
        >
          <option value="mp4">MP4</option>
          <option value="mov">MOV</option>
          {draft.codec === 'vp9' ? null : <option value="mpegTs">MPEG-TS</option>}
        </Select>
        <Select
          label="视频编码"
          value={draft.codec}
          onChange={(event) => {
            const codec = event.target.value as EncodeDraft['codec'];
            patch({
              codec,
              container: codec === 'vp9' && draft.container === 'mpegTs' ? 'mp4' : draft.container
            });
          }}
        >
          <option value="av1">AV1</option>
          <option value="hevc">HEVC</option>
          <option value="vp9">VP9</option>
        </Select>
      </div>
      {draft.codec === 'av1' ? (
        <div className={styles.pair}>
          <Select
            label="速度"
            hint="数字越小越慢，同样质量下文件更小"
            value={draft.av1Preset}
            onChange={(event) => patch({ av1Preset: event.target.value })}
          >
            <option value="4">4 · 更小</option>
            <option value="6">6 · 默认</option>
            <option value="8">8 · 较快</option>
            <option value="10">10 · 很快</option>
          </Select>
          <TextField
            label="质量 CRF"
            hint={crfHint}
            value={draft.av1Crf}
            inputMode="numeric"
            onChange={(event) => patch({ av1Crf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'hevc' ? (
        <div className={styles.pair}>
          <Select
            label="速度"
            value={draft.hevcPreset}
            onChange={(event) => patch({ hevcPreset: event.target.value as HevcPreset })}
          >
            <option value="veryFast">veryfast · 很快</option>
            <option value="fast">fast · 较快</option>
            <option value="medium">medium · 默认</option>
            <option value="slow">slow · 更小</option>
            <option value="slower">slower · 更慢</option>
          </Select>
          <TextField
            label="质量 CRF"
            hint={crfHint}
            value={draft.hevcCrf}
            inputMode="numeric"
            onChange={(event) => patch({ hevcCrf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'vp9' ? (
        <div className={styles.pair}>
          <Select
            label="速度"
            hint="数字越小越慢"
            value={draft.vp9Speed}
            onChange={(event) => patch({ vp9Speed: event.target.value })}
          >
            <option value="1">1 · 更小</option>
            <option value="2">2 · 默认</option>
            <option value="4">4 · 较快</option>
            <option value="5">5 · 很快</option>
          </Select>
          <TextField
            label="质量 CRF"
            hint={crfHint}
            value={draft.vp9Crf}
            inputMode="numeric"
            onChange={(event) => patch({ vp9Crf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'av1' ? (
        <Select
          label="位深"
          hint="10-bit 渐变更干净，播放器支持更少"
          value={draft.av1Bit}
          onChange={(event) => patch({ av1Bit: event.target.value as BitDepth })}
        >
          <option value="eight">8-bit</option>
          <option value="ten">10-bit</option>
        </Select>
      ) : null}
      {draft.codec === 'hevc' ? (
        <div className={styles.pair}>
          <Select
            label="调优"
            value={draft.hevcTune}
            onChange={(event) => patch({ hevcTune: event.target.value as HevcTune })}
          >
            <option value="none">不调优</option>
            <option value="animation">动画</option>
            <option value="grain">保留颗粒</option>
          </Select>
          <Select
            label="位深"
            value={draft.hevcBit}
            onChange={(event) => patch({ hevcBit: event.target.value as BitDepth })}
          >
            <option value="eight">8-bit</option>
            <option value="ten">10-bit</option>
          </Select>
        </div>
      ) : null}
      {draft.codec === 'vp9' ? (
        <Select
          label="画面"
          value={draft.vp9Content}
          onChange={(event) => patch({ vp9Content: event.target.value as Vp9Content })}
        >
          <option value="general">普通画面</option>
          <option value="screen">录屏</option>
          <option value="film">胶片颗粒</option>
        </Select>
      ) : null}
      {perFile ? (
        <TextField
          label="帧率"
          hint="留空保留源帧率"
          value={draft.fps}
          inputMode="numeric"
          onChange={(event) => patch({ fps: event.target.value })}
        />
      ) : null}
      <div className={styles.pair}>
        <TextField
          label="宽度"
          hint="留空跟源。合并取最大，追加跟基准"
          value={draft.width}
          inputMode="numeric"
          onChange={(event) => patch({ width: event.target.value })}
        />
        <TextField
          label="高度"
          value={draft.height}
          inputMode="numeric"
          onChange={(event) => patch({ height: event.target.value })}
        />
      </div>
      <Select label="画面适配" value={draft.fit} onChange={(event) => patch({ fit: event.target.value as Fit })}>
        <option value="contain">缩进画面，不足处补边</option>
        <option value="fill">拉伸到目标尺寸</option>
        <option value="cover">铺满后裁切（当前不可用）</option>
      </Select>
    </div>
  );
}
