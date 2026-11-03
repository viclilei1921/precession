import { useTranslation } from 'react-i18next';
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
  const { t } = useTranslation('video');
  function patch(next: Partial<EncodeDraft>) {
    onChange({ ...draft, ...next });
  }

  return (
    <div className={styles.fields}>
      <div className={styles.pair}>
        <Select
          label={t('encode.picture.outputFormat')}
          value={draft.container}
          onChange={(event) => patch({ container: event.target.value as VideoContainer })}
        >
          <option value="mp4">MP4</option>
          <option value="mov">MOV</option>
          {draft.codec === 'vp9' ? null : <option value="mpegTs">MPEG-TS</option>}
        </Select>
        <Select
          label={t('encode.picture.videoCodec')}
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
            label={t('encode.picture.speed')}
            value={draft.av1Preset}
            onChange={(event) => patch({ av1Preset: event.target.value })}
          >
            <option value="4">{t('encode.picture.av1Preset4')}</option>
            <option value="6">{t('encode.picture.av1Preset6')}</option>
            <option value="8">{t('encode.picture.av1Preset8')}</option>
            <option value="10">{t('encode.picture.av1Preset10')}</option>
          </Select>
          <TextField
            label={t('encode.picture.qualityCrf')}
            value={draft.av1Crf}
            inputMode="numeric"
            onChange={(event) => patch({ av1Crf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'hevc' ? (
        <div className={styles.pair}>
          <Select
            label={t('encode.picture.speed')}
            value={draft.hevcPreset}
            onChange={(event) => patch({ hevcPreset: event.target.value as HevcPreset })}
          >
            <option value="veryFast">{t('encode.picture.hevcPresetVeryFast')}</option>
            <option value="fast">{t('encode.picture.hevcPresetFast')}</option>
            <option value="medium">{t('encode.picture.hevcPresetMedium')}</option>
            <option value="slow">{t('encode.picture.hevcPresetSlow')}</option>
            <option value="slower">{t('encode.picture.hevcPresetSlower')}</option>
          </Select>
          <TextField
            label={t('encode.picture.qualityCrf')}
            value={draft.hevcCrf}
            inputMode="numeric"
            onChange={(event) => patch({ hevcCrf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'vp9' ? (
        <div className={styles.pair}>
          <Select
            label={t('encode.picture.speed')}
            value={draft.vp9Speed}
            onChange={(event) => patch({ vp9Speed: event.target.value })}
          >
            <option value="1">{t('encode.picture.vp9Speed1')}</option>
            <option value="2">{t('encode.picture.vp9Speed2')}</option>
            <option value="4">{t('encode.picture.vp9Speed4')}</option>
            <option value="5">{t('encode.picture.vp9Speed5')}</option>
          </Select>
          <TextField
            label={t('encode.picture.qualityCrf')}
            value={draft.vp9Crf}
            inputMode="numeric"
            onChange={(event) => patch({ vp9Crf: event.target.value })}
          />
        </div>
      ) : null}
      {draft.codec === 'av1' ? (
        <Select
          label={t('encode.picture.bitDepth')}
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
            label={t('encode.picture.tune')}
            value={draft.hevcTune}
            onChange={(event) => patch({ hevcTune: event.target.value as HevcTune })}
          >
            <option value="none">{t('encode.picture.tuneNone')}</option>
            <option value="animation">{t('encode.picture.tuneAnimation')}</option>
            <option value="grain">{t('encode.picture.tuneGrain')}</option>
          </Select>
          <Select
            label={t('encode.picture.bitDepth')}
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
          label={t('encode.picture.content')}
          value={draft.vp9Content}
          onChange={(event) => patch({ vp9Content: event.target.value as Vp9Content })}
        >
          <option value="general">{t('encode.picture.contentGeneral')}</option>
          <option value="screen">{t('encode.picture.contentScreen')}</option>
          <option value="film">{t('encode.picture.contentFilm')}</option>
        </Select>
      ) : null}
      {perFile ? (
        <TextField
          label={t('encode.picture.frameRate')}
          value={draft.fps}
          inputMode="numeric"
          onChange={(event) => patch({ fps: event.target.value })}
        />
      ) : null}
      <div className={styles.pair}>
        <TextField
          label={t('encode.picture.width')}
          value={draft.width}
          inputMode="numeric"
          onChange={(event) => patch({ width: event.target.value })}
        />
        <TextField
          label={t('encode.picture.height')}
          value={draft.height}
          inputMode="numeric"
          onChange={(event) => patch({ height: event.target.value })}
        />
      </div>
      <Select
        label={t('encode.picture.fit')}
        value={draft.fit}
        onChange={(event) => patch({ fit: event.target.value as Fit })}
      >
        <option value="contain">{t('encode.picture.fitContain')}</option>
        <option value="fill">{t('encode.picture.fitFill')}</option>
        <option value="cover">{t('encode.picture.fitCover')}</option>
      </Select>
    </div>
  );
}
