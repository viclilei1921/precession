import i18n from '@/i18n';

export type SectionNote = {
  name: string;
  text: string;
};

type Codec = 'av1' | 'hevc' | 'vp9';

function crfText(codec: Codec) {
  if (codec === 'hevc') {
    return i18n.t('video:notes.text.crfHevc');
  }
  if (codec === 'vp9') {
    return i18n.t('video:notes.text.crfVp9');
  }
  return i18n.t('video:notes.text.crfAv1');
}

/** 当前分组里需要说明的参数。分组切换时整段一起换。 */
export function sectionNotes(
  section: 'picture' | 'audio' | 'title' | 'subtitle',
  codec: Codec,
  perFile: boolean
): SectionNote[] {
  if (section === 'picture') {
    const notes: SectionNote[] = [
      {
        name: i18n.t('video:notes.name.speed'),
        text: i18n.t(codec === 'vp9' ? 'video:notes.text.speedVp9' : 'video:notes.text.speedAv1')
      },
      { name: i18n.t('video:notes.name.crf'), text: crfText(codec) }
    ];
    if (codec !== 'vp9') {
      notes.push({ name: i18n.t('video:notes.name.bitDepth'), text: i18n.t('video:notes.text.bitDepth') });
    }
    if (perFile) {
      notes.push({ name: i18n.t('video:notes.name.frameRate'), text: i18n.t('video:notes.text.frameRate') });
    }
    notes.push({ name: i18n.t('video:notes.name.width'), text: i18n.t('video:notes.text.width') });
    return notes;
  }
  if (section === 'title') {
    return [
      { name: i18n.t('video:notes.name.fontSize'), text: i18n.t('video:notes.text.fontSize') },
      { name: i18n.t('video:notes.name.titleStart'), text: i18n.t('video:notes.text.titleStart') },
      { name: i18n.t('video:notes.name.titleEnd'), text: i18n.t('video:notes.text.titleEnd') }
    ];
  }
  if (section === 'subtitle') {
    return [
      { name: i18n.t('video:notes.name.subtitleTrack'), text: i18n.t('video:notes.text.subtitleTrack') },
      { name: i18n.t('video:notes.name.subtitleStyle'), text: i18n.t('video:notes.text.subtitleStyle') }
    ];
  }
  return [];
}
