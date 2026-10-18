import { useState } from 'react';
import type { AudioTrack } from '@/bridge/task';
import { SegmentButton } from '@/components/button/segment';
import { Card } from '@/components/card';
import type { EncodeDraft } from '../encode-draft';
import { encodeSummary } from '../encode-draft';
import styles from './encode.module.css';
import { EncodeAudio } from './encode-audio';
import { EncodePicture } from './encode-picture';
import { EncodeSubtitle } from './encode-subtitle';
import { EncodeTitle } from './encode-title';

const sections = [
  { id: 'picture', label: '画面' },
  { id: 'audio', label: '音频' },
  { id: 'title', label: '标题' },
  { id: 'subtitle', label: '字幕' }
] as const;

type EncodeSection = (typeof sections)[number]['id'];

type EncodePanelProps = {
  draft: EncodeDraft;
  perFile: boolean;
  tracks: AudioTrack[];
  onChange: (draft: EncodeDraft) => void;
};

/** 自定义编码一次只展开一组参数。 */
export function EncodePanel({ draft, perFile, tracks, onChange }: EncodePanelProps) {
  const [section, setSection] = useState<EncodeSection>('picture');

  return (
    <Card title="自定义">
      <div className={styles.segments} role="tablist" aria-label="编码分组">
        {sections.map((item) => (
          <SegmentButton key={item.id} selected={section === item.id} onClick={() => setSection(item.id)}>
            {item.label}
          </SegmentButton>
        ))}
      </div>
      <p className={styles.summary}>{encodeSummary(draft)}</p>
      {section === 'picture' ? <EncodePicture draft={draft} perFile={perFile} onChange={onChange} /> : null}
      {section === 'audio' ? <EncodeAudio draft={draft} tracks={tracks} onChange={onChange} /> : null}
      {section === 'title' ? <EncodeTitle draft={draft} onChange={onChange} /> : null}
      {section === 'subtitle' ? <EncodeSubtitle draft={draft} onChange={onChange} /> : null}
    </Card>
  );
}
