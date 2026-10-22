import { useState } from 'react';
import { SegmentButton } from '@/components/button/segment';
import { Card } from '@/components/card';
import type { ImageParamsDraft } from '../params-draft';
import { avifSummary, jxlSummary } from '../params-draft';
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

/** 自定义编码一次只展开一组参数。 */
export function ParamsPanel({ format, draft, onChange }: ParamsPanelProps) {
  const [section, setSection] = useState<SectionId>('quality');
  const sections = format === 'avif' ? avifSections : jxlSections;
  const visible = sections.some((item) => item.id === section) ? section : 'quality';

  return (
    <Card title="自定义">
      <div className={styles.segments} role="tablist" aria-label="编码分组">
        {sections.map((item) => (
          <SegmentButton key={item.id} selected={visible === item.id} onClick={() => setSection(item.id)}>
            {item.label}
          </SegmentButton>
        ))}
      </div>
      <p className={styles.summary}>{format === 'avif' ? avifSummary(draft) : jxlSummary(draft)}</p>
      {visible === 'quality' ? <ParamsQuality format={format} draft={draft} onChange={onChange} /> : null}
      {visible === 'pixel' && format === 'avif' ? <ParamsPixel draft={draft} onChange={onChange} /> : null}
      {visible === 'pixel' && format === 'jxl' ? <ParamsJxlPixel draft={draft} onChange={onChange} /> : null}
      {visible === 'file' ? <ParamsFile draft={draft} onChange={onChange} /> : null}
      {visible === 'color' ? <ParamsColor draft={draft} onChange={onChange} /> : null}
      {visible === 'container' ? <ParamsContainer draft={draft} onChange={onChange} /> : null}
    </Card>
  );
}
