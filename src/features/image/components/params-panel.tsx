import { useState } from 'react';
import { SegmentButton } from '@/components/button/segment';
import { Scroll } from '@/components/scroll';
import { InfoTip } from '@/components/tooltip';
import type { ImageParamsDraft } from '../params-draft';
import { sectionNotes } from '../params-notes';
import styles from './params.module.css';
import { ParamsColor } from './params-color';
import { ParamsContainer } from './params-container';
import { ParamsFile } from './params-file';
import { ParamsJxlPixel } from './params-jxl-pixel';
import { ParamsPixel } from './params-pixel';
import { ParamsQuality } from './params-quality';

const avifSections = [
  { id: 'quality', label: '质量' },
  { id: 'pixel', label: '像素' },
  { id: 'file', label: '文件' }
] as const;

const jxlSections = [
  { id: 'quality', label: '质量' },
  { id: 'pixel', label: '像素' },
  { id: 'color', label: '色彩' },
  { id: 'container', label: '容器' }
] as const;

type SectionId = (typeof avifSections)[number]['id'] | (typeof jxlSections)[number]['id'];

type ParamsPanelProps = {
  format: 'avif' | 'jxl';
  draft: ImageParamsDraft;
  onChange: (draft: ImageParamsDraft) => void;
};

/** 自定义编码一次只展开一组参数。两种格式各自记住当前分组。 */
export function ParamsPanel({ format, draft, onChange }: ParamsPanelProps) {
  const [picked, setPicked] = useState<Record<'avif' | 'jxl', SectionId>>({
    avif: 'quality',
    jxl: 'quality'
  });
  const sections = format === 'avif' ? avifSections : jxlSections;
  const visible = sections.some((item) => item.id === picked[format]) ? picked[format] : 'quality';
  const current = sections.find((item) => item.id === visible) ?? sections[0];
  const notes = sectionNotes(format, visible);

  return (
    <div className={styles.panel}>
      <div className={styles.bar}>
        <div className={styles.segments} role="tablist" aria-label="参数分组">
          {sections.map((item) => (
            <SegmentButton
              key={item.id}
              selected={visible === item.id}
              onClick={() => setPicked((current) => ({ ...current, [format]: item.id }))}
            >
              {item.label}
            </SegmentButton>
          ))}
        </div>
        <InfoTip
          label={`${current.label}说明`}
          title={
            <Scroll className={styles.tip} axis="vertical" edge>
              <div className={styles.tipBody}>
                {notes.map((note) => (
                  <p className={styles.tipItem} key={note.name}>
                    <span className={styles.tipName}>{note.name}</span>
                    {note.text}
                  </p>
                ))}
              </div>
            </Scroll>
          }
        />
      </div>
      {visible === 'quality' ? <ParamsQuality format={format} draft={draft} onChange={onChange} /> : null}
      {visible === 'pixel' && format === 'avif' ? <ParamsPixel draft={draft} onChange={onChange} /> : null}
      {visible === 'pixel' && format === 'jxl' ? <ParamsJxlPixel draft={draft} onChange={onChange} /> : null}
      {visible === 'file' ? <ParamsFile draft={draft} onChange={onChange} /> : null}
      {visible === 'color' ? <ParamsColor draft={draft} onChange={onChange} /> : null}
      {visible === 'container' ? <ParamsContainer draft={draft} onChange={onChange} /> : null}
    </div>
  );
}
