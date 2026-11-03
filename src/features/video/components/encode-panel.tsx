import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { AudioTrack } from '@/bridge/task';
import { SegmentButton } from '@/components/button/segment';
import { Scroll } from '@/components/scroll';
import { InfoTip } from '@/components/tooltip';
import type { EncodeDraft } from '../encode-draft';
import { encodeSummary } from '../encode-draft';
import { sectionNotes } from '../encode-notes';
import styles from './encode.module.css';
import { EncodeAudio } from './encode-audio';
import { EncodePicture } from './encode-picture';
import { EncodeSubtitle } from './encode-subtitle';
import { EncodeTitle } from './encode-title';

const sections = [
  { id: 'picture', labelKey: 'encode.sections.picture' },
  { id: 'audio', labelKey: 'encode.sections.audio' },
  { id: 'title', labelKey: 'encode.sections.title' },
  { id: 'subtitle', labelKey: 'encode.sections.subtitle' }
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
  const { t } = useTranslation('video');
  const [section, setSection] = useState<EncodeSection>('picture');
  const current = sections.find((item) => item.id === section) ?? sections[0];
  const currentLabel = t(current.labelKey);
  const notes = sectionNotes(section, draft.codec, perFile);

  return (
    <div className={styles.panel}>
      <div className={styles.bar}>
        <div className={styles.segments} role="tablist" aria-label={t('encode.sectionGroup')}>
          {sections.map((item) => (
            <SegmentButton key={item.id} selected={section === item.id} onClick={() => setSection(item.id)}>
              {t(item.labelKey)}
            </SegmentButton>
          ))}
        </div>
        {notes.length > 0 ? (
          <InfoTip
            label={t('encode.sectionHint', { section: currentLabel })}
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
        ) : null}
      </div>
      <p className={styles.summary}>{encodeSummary(draft)}</p>
      {section === 'picture' ? <EncodePicture draft={draft} perFile={perFile} onChange={onChange} /> : null}
      {section === 'audio' ? <EncodeAudio draft={draft} tracks={tracks} onChange={onChange} /> : null}
      {section === 'title' ? <EncodeTitle draft={draft} onChange={onChange} /> : null}
      {section === 'subtitle' ? <EncodeSubtitle draft={draft} onChange={onChange} /> : null}
    </div>
  );
}
