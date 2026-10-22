import type { AvifDepth, AvifRange, AvifYuv, JxlColorEncoding } from '@/bridge/task';

export type ImageParamsDraft = {
  qcolor: string;
  qalpha: string;
  speed: string;
  lossless: boolean;
  targetSize: string;
  depth: AvifDepth;
  yuv: AvifYuv;
  range: AvifRange;
  sharpyuv: boolean;
  premultiply: boolean;
  ignoreExif: boolean;
  ignoreXmp: boolean;
  ignoreIcc: boolean;
  autotiling: boolean;
  tilerowslog2: string;
  tilecolslog2: string;
  progressive: boolean;
  jobs: string;
  quality: string;
  effort: string;
  decodingSpeed: string;
  alphaDistance: string;
  losslessJpeg: boolean;
  colorEncoding: 'source' | JxlColorEncoding;
  targetIntensity: string;
  useContainer: boolean;
  jxlModular: 'auto' | 'vardct' | 'modular';
  jxlResampling: 'auto' | '1' | '2' | '4' | '8';
  jxlPremultiply: 'auto' | 'off' | 'on';
  jxlBitDepth: 'source' | '8' | '10' | '12' | '16';
  photonNoise: string;
  jxlThreads: string;
};

export function initialImageDraft(): ImageParamsDraft {
  return {
    qcolor: '60',
    qalpha: '60',
    speed: '6',
    lossless: false,
    targetSize: '',
    depth: 'eight',
    yuv: 'auto',
    range: 'auto',
    sharpyuv: false,
    premultiply: false,
    ignoreExif: false,
    ignoreXmp: false,
    ignoreIcc: false,
    autotiling: true,
    tilerowslog2: '',
    tilecolslog2: '',
    progressive: false,
    jobs: '',
    quality: '1',
    effort: '7',
    decodingSpeed: '0',
    alphaDistance: '',
    losslessJpeg: false,
    colorEncoding: 'source',
    targetIntensity: '',
    useContainer: false,
    jxlModular: 'auto',
    jxlResampling: 'auto',
    jxlPremultiply: 'auto',
    jxlBitDepth: 'source',
    photonNoise: '',
    jxlThreads: ''
  };
}

const depthLabel: Record<AvifDepth, string> = {
  eight: '8-bit',
  ten: '10-bit',
  twelve: '12-bit'
};

/** 默认模式固定使用的 AVIF 参数。 */
export const defaultAvifSummary = 'AVIF · 质量 60 · 速度 6 · 8-bit';

/** 默认模式固定使用的 JXL 参数。 */
export const defaultJxlSummary = 'JPEG XL · 距离 1 · effort 7';

/** 自定义模式当前会提交的 AVIF 参数。 */
export function avifSummary(draft: ImageParamsDraft) {
  const loss = draft.lossless ? ' · 无损' : '';
  return `AVIF · 质量 ${draft.qcolor} · 速度 ${draft.speed} · ${depthLabel[draft.depth]}${loss}`;
}

/** 自定义模式当前会提交的 JXL 参数。 */
export function jxlSummary(draft: ImageParamsDraft) {
  const loss = draft.lossless ? ' · 无损' : '';
  const mode = draft.jxlModular === 'modular' ? ' · 模块化' : draft.jxlModular === 'vardct' ? ' · VarDCT' : '';
  const sample =
    draft.jxlResampling === 'auto' ? '' : draft.jxlResampling === '1' ? ' · 不抽样' : ` · 抽样 ${draft.jxlResampling}×`;
  const depth = draft.jxlBitDepth === 'source' ? '' : ` · ${draft.jxlBitDepth}-bit`;
  return `JPEG XL · 距离 ${draft.quality} · effort ${draft.effort}${loss}${mode}${sample}${depth}`;
}
