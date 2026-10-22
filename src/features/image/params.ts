import type { AvifEncodeParams, JxlEncodeParams } from '@/bridge/task';
import type { ImageParamsDraft } from './params-draft';
import { initialImageDraft } from './params-draft';

function numberOr(value: string, fallback: number, min: number, max: number) {
  const parsed = Number(value.trim());
  if (!Number.isFinite(parsed)) {
    return fallback;
  }
  return Math.min(max, Math.max(min, parsed));
}

function wholeOr(value: string, fallback: number, min: number, max: number) {
  return Math.round(numberOr(value, fallback, min, max));
}

function optionalWhole(value: string, min: number, max: number) {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed)) {
    return null;
  }
  return Math.min(max, Math.max(min, Math.round(parsed)));
}

function optionalPositive(value: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }
  return Math.min(Number.MAX_SAFE_INTEGER, Math.round(parsed));
}

function optionalFloat(value: string, min: number) {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed)) {
    return null;
  }
  return Math.max(min, parsed);
}

function blank(value: string | null | undefined) {
  return value == null || value.trim() === '';
}

function isDefaultAvif(params: AvifEncodeParams) {
  return (
    (params.qcolor ?? 60) === 60 &&
    (params.qalpha ?? 60) === 60 &&
    (params.speed ?? 6) === 6 &&
    !params.lossless &&
    params.jobs == null &&
    (params.depth == null || params.depth === 'eight') &&
    (params.yuv == null || params.yuv === 'auto') &&
    !params.premultiply &&
    !params.sharpyuv &&
    !params.ignoreExif &&
    !params.ignoreXmp &&
    !params.ignoreIcc &&
    (params.range == null || params.range === 'auto') &&
    params.autotiling !== false &&
    params.tilerowslog2 == null &&
    params.tilecolslog2 == null &&
    params.targetSize == null &&
    !params.progressive &&
    blank(params.cicp) &&
    blank(params.codec) &&
    blank(params.pasp) &&
    blank(params.crop) &&
    params.irot == null &&
    params.imir == null &&
    blank(params.clli) &&
    (params.advanced == null || params.advanced.length === 0)
  );
}

function isDefaultJxl(params: JxlEncodeParams) {
  return (
    !params.lossless &&
    (params.quality ?? 1) === 1 &&
    (params.effort ?? 7) === 7 &&
    !params.losslessJpeg &&
    !params.useContainer &&
    (params.decodingSpeed ?? 0) === 0 &&
    params.colorEncoding == null &&
    params.targetIntensity == null &&
    params.alphaDistance == null &&
    !params.progressive &&
    params.groupOrder == null &&
    params.compressBoxes == null &&
    params.brotliEffort == null &&
    params.modular == null &&
    params.numThreads == null &&
    params.photonNoiseIso == null &&
    params.allowJpegReconstruction == null &&
    params.codestreamLevel == null &&
    params.buffering == null &&
    params.premultiply == null &&
    params.keepInvisible == null &&
    !params.progressiveAc &&
    !params.qprogressiveAc &&
    params.progressiveDc == null &&
    params.resampling == null &&
    params.ecResampling == null &&
    !params.alreadyDownsampled &&
    params.upsamplingMode == null &&
    params.epf == null &&
    params.gaborish == null &&
    params.overrideBitdepth == null &&
    params.noise == null &&
    params.dots == null &&
    params.patches == null &&
    (params.extraHints == null || params.extraHints.length === 0)
  );
}

/** 把队列里记下的图片参数填回界面。和默认导出一致时回到默认模式。 */
export function formFromParams(
  kind: 'convertAvif' | 'convertJxl',
  params: AvifEncodeParams | JxlEncodeParams
): { mode: 'default' | 'custom'; draft: ImageParamsDraft } {
  const draft = initialImageDraft();
  if (kind === 'convertAvif') {
    const avif = params as AvifEncodeParams;
    if (isDefaultAvif(avif)) {
      return { mode: 'default', draft };
    }
    draft.qcolor = String(avif.qcolor ?? 60);
    draft.qalpha = String(avif.qalpha ?? 60);
    draft.speed = String(avif.speed ?? 6);
    draft.lossless = Boolean(avif.lossless);
    draft.targetSize = avif.targetSize == null ? '' : String(avif.targetSize);
    draft.depth = avif.depth ?? 'eight';
    draft.yuv = avif.yuv ?? 'auto';
    draft.range = avif.range ?? 'auto';
    draft.sharpyuv = Boolean(avif.sharpyuv);
    draft.premultiply = Boolean(avif.premultiply);
    draft.ignoreExif = Boolean(avif.ignoreExif);
    draft.ignoreXmp = Boolean(avif.ignoreXmp);
    draft.ignoreIcc = Boolean(avif.ignoreIcc);
    draft.autotiling = avif.autotiling !== false;
    draft.tilerowslog2 = avif.tilerowslog2 == null ? '' : String(avif.tilerowslog2);
    draft.tilecolslog2 = avif.tilecolslog2 == null ? '' : String(avif.tilecolslog2);
    draft.progressive = Boolean(avif.progressive);
    draft.jobs = avif.jobs == null ? '' : String(avif.jobs);
    return { mode: 'custom', draft };
  }
  const jxl = params as JxlEncodeParams;
  if (isDefaultJxl(jxl)) {
    return { mode: 'default', draft };
  }
  draft.lossless = Boolean(jxl.lossless);
  draft.quality = String(jxl.quality ?? 1);
  draft.effort = String(jxl.effort ?? 7);
  draft.decodingSpeed = String(jxl.decodingSpeed ?? 0);
  draft.alphaDistance = jxl.alphaDistance == null ? '' : String(jxl.alphaDistance);
  draft.losslessJpeg = Boolean(jxl.losslessJpeg);
  draft.colorEncoding = jxl.colorEncoding ?? 'source';
  draft.targetIntensity = jxl.targetIntensity == null ? '' : String(jxl.targetIntensity);
  draft.useContainer = Boolean(jxl.useContainer);
  draft.progressive = Boolean(jxl.progressive);
  draft.jxlModular = jxl.modular === 1 ? 'modular' : jxl.modular === 0 ? 'vardct' : 'auto';
  draft.jxlResampling = resamplingChoice(jxl.resampling);
  draft.jxlPremultiply = jxl.premultiply === 1 ? 'on' : jxl.premultiply === 0 ? 'off' : 'auto';
  draft.jxlBitDepth = bitDepthChoice(jxl.overrideBitdepth);
  draft.photonNoise = jxl.photonNoiseIso == null || jxl.photonNoiseIso <= 0 ? '' : String(jxl.photonNoiseIso);
  draft.jxlThreads = jxl.numThreads == null ? '' : String(jxl.numThreads);
  return { mode: 'custom', draft };
}

function bitDepthChoice(value: number | null | undefined): ImageParamsDraft['jxlBitDepth'] {
  if (value === 8) {
    return '8';
  }
  if (value === 10) {
    return '10';
  }
  if (value === 12) {
    return '12';
  }
  if (value === 16) {
    return '16';
  }
  return 'source';
}

function resamplingChoice(value: number | null | undefined): ImageParamsDraft['jxlResampling'] {
  if (value === 1) {
    return '1';
  }
  if (value === 2) {
    return '2';
  }
  if (value === 4) {
    return '4';
  }
  if (value === 8) {
    return '8';
  }
  return 'auto';
}

/** 自定义模式提交给后端的 AVIF 参数。空着的可选数字不传。 */
export function buildAvifParams(draft: ImageParamsDraft): AvifEncodeParams {
  return {
    qcolor: wholeOr(draft.qcolor, 60, 0, 100),
    qalpha: wholeOr(draft.qalpha, 60, 0, 100),
    speed: wholeOr(draft.speed, 6, 0, 10),
    lossless: draft.lossless,
    jobs: optionalWhole(draft.jobs, 1, 255),
    depth: draft.depth,
    yuv: draft.yuv,
    premultiply: draft.premultiply,
    sharpyuv: draft.sharpyuv,
    ignoreExif: draft.ignoreExif,
    ignoreXmp: draft.ignoreXmp,
    ignoreIcc: draft.ignoreIcc,
    range: draft.range,
    autotiling: draft.autotiling,
    tilerowslog2: draft.autotiling ? null : optionalWhole(draft.tilerowslog2, 0, 6),
    tilecolslog2: draft.autotiling ? null : optionalWhole(draft.tilecolslog2, 0, 6),
    targetSize: optionalPositive(draft.targetSize),
    progressive: draft.progressive
  };
}

/** 自定义模式提交给后端的 JXL 参数。空着的可选数字不传。 */
export function buildJxlParams(draft: ImageParamsDraft): JxlEncodeParams {
  return {
    lossless: draft.lossless,
    quality: numberOr(draft.quality, 1, 0, 25),
    effort: wholeOr(draft.effort, 7, 1, 10),
    losslessJpeg: draft.losslessJpeg,
    useContainer: draft.useContainer,
    decodingSpeed: wholeOr(draft.decodingSpeed, 0, 0, 4),
    colorEncoding: draft.colorEncoding === 'source' ? null : draft.colorEncoding,
    targetIntensity: optionalFloat(draft.targetIntensity, 0),
    alphaDistance: boundedDistance(draft.alphaDistance),
    progressive: draft.progressive,
    modular: draft.jxlModular === 'modular' ? 1 : draft.jxlModular === 'vardct' ? 0 : null,
    resampling: draft.jxlResampling === 'auto' ? null : Number(draft.jxlResampling),
    premultiply: draft.jxlPremultiply === 'on' ? 1 : draft.jxlPremultiply === 'off' ? 0 : null,
    overrideBitdepth: draft.jxlBitDepth === 'source' ? null : Number(draft.jxlBitDepth),
    photonNoiseIso: optionalPositive(draft.photonNoise),
    numThreads: optionalWhole(draft.jxlThreads, 1, 256)
  };
}

function boundedDistance(value: string) {
  const distance = optionalFloat(value, 0);
  return distance == null ? null : Math.min(25, distance);
}
