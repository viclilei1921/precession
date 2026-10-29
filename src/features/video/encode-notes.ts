export type SectionNote = {
  name: string;
  text: string;
};

type Codec = 'av1' | 'hevc' | 'vp9';

function crfText(codec: Codec) {
  if (codec === 'hevc') {
    return '0 最清晰，51 最小。默认 26。';
  }
  if (codec === 'vp9') {
    return '0 最清晰，63 最小。默认 33。';
  }
  return '0 最清晰，63 最小。默认 35。';
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
        name: '速度',
        text: codec === 'vp9' ? '数字越小越慢。' : '数字越小越慢，同样质量下文件更小。'
      },
      { name: '质量 CRF', text: crfText(codec) }
    ];
    if (codec !== 'vp9') {
      notes.push({ name: '位深', text: '10-bit 渐变更干净，播放器支持更少。' });
    }
    if (perFile) {
      notes.push({ name: '帧率', text: '留空保留源帧率。' });
    }
    notes.push({ name: '宽度', text: '留空跟源。合并取最大，追加跟基准。' });
    return notes;
  }
  if (section === 'title') {
    return [
      { name: '字号', text: '留空为 24，按画面宽度缩放。' },
      { name: '标题开始（秒）', text: '留空从片头开始。' },
      { name: '标题结束（秒）', text: '留空一直显示。' }
    ];
  }
  if (section === 'subtitle') {
    return [
      { name: '字幕流序号', text: 'Stream #0:N 的 N。' },
      { name: '字幕样式', text: '只对 SRT、WebVTT 生效，可以留空。' }
    ];
  }
  return [];
}
