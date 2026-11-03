import i18n from '@/i18n';

export type SectionNote = {
  name: string;
  text: string;
};

type NoteKey =
  | 'avifQuality'
  | 'avifAlpha'
  | 'avifSpeed'
  | 'avifTargetSize'
  | 'avifLossless'
  | 'avifDepth'
  | 'avifYuv'
  | 'avifRange'
  | 'avifSharpYuv'
  | 'avifPremultiply'
  | 'avifExif'
  | 'avifXmp'
  | 'avifIcc'
  | 'avifTiles'
  | 'avifTileCount'
  | 'avifProgressive'
  | 'avifJobs'
  | 'jxlDistance'
  | 'jxlEffort'
  | 'jxlDecoding'
  | 'jxlAlphaDistance'
  | 'jxlLossless'
  | 'jxlLosslessJpeg'
  | 'jxlColor'
  | 'jxlIntensity'
  | 'jxlModular'
  | 'jxlResampling'
  | 'jxlBitDepth'
  | 'jxlPremultiply'
  | 'jxlPhoton'
  | 'jxlThreads'
  | 'jxlContainer'
  | 'jxlProgressive';

function note(key: NoteKey): SectionNote {
  return {
    name: i18n.t(`image:notes.${key}.name`),
    text: i18n.t(`image:notes.${key}.text`)
  };
}

/** 当前分组里每个参数的说明。分组切换时整段一起换。 */
export function sectionNotes(
  format: 'avif' | 'jxl',
  section: 'quality' | 'pixel' | 'file' | 'color' | 'container'
): SectionNote[] {
  if (format === 'avif' && section === 'quality') {
    return [note('avifQuality'), note('avifAlpha'), note('avifSpeed'), note('avifTargetSize'), note('avifLossless')];
  }
  if (format === 'avif' && section === 'pixel') {
    return [note('avifDepth'), note('avifYuv'), note('avifRange'), note('avifSharpYuv'), note('avifPremultiply')];
  }
  if (format === 'avif' && section === 'file') {
    return [
      note('avifJobs'),
      note('avifExif'),
      note('avifXmp'),
      note('avifIcc'),
      note('avifTiles'),
      note('avifTileCount'),
      note('avifProgressive')
    ];
  }
  if (format === 'jxl' && section === 'quality') {
    return [
      note('jxlDistance'),
      note('jxlEffort'),
      note('jxlDecoding'),
      note('jxlAlphaDistance'),
      note('jxlPhoton'),
      note('jxlLossless'),
      note('jxlLosslessJpeg')
    ];
  }
  if (format === 'jxl' && section === 'pixel') {
    return [note('jxlModular'), note('jxlResampling'), note('jxlPremultiply'), note('jxlBitDepth')];
  }
  if (format === 'jxl' && section === 'color') {
    return [note('jxlColor'), note('jxlIntensity')];
  }
  if (format === 'jxl' && section === 'container') {
    return [note('jxlThreads'), note('jxlContainer'), note('jxlProgressive')];
  }
  return [];
}
