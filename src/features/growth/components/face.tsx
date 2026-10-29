import { useEffect, useState } from 'react';
import type { MarkdownImageOwner } from '@/components/markdown';
import { uploadDisplayUrl } from '@/components/markdown/image';
import styles from './face.module.css';

type MemberFaceProps = {
  name: string;
  imageId: string;
  owner: MarkdownImageOwner;
  size?: 'md' | 'lg';
};

function showable(url: string) {
  return url.startsWith('blob:') || url.startsWith('http://') || url.startsWith('https://');
}

/** 家人头像。没有图片时用名字的第一个字 */
export function MemberFace({ name, imageId, owner, size = 'md' }: MemberFaceProps) {
  const [src, setSrc] = useState('');
  const mark = Array.from(name.trim())[0] ?? '人';
  const ownerKind = owner.owner;
  const ownerId = owner.ownerId;

  useEffect(() => {
    if (!imageId) {
      setSrc('');
      return;
    }
    let alive = true;
    setSrc('');
    void uploadDisplayUrl(`upload:${imageId}`, { owner: ownerKind, ownerId }).then((url) => {
      if (alive) {
        setSrc(showable(url) ? url : '');
      }
    });
    return () => {
      alive = false;
    };
  }, [imageId, ownerKind, ownerId]);

  return <span className={size === 'lg' ? styles.lg : styles.face}>{src ? <img src={src} alt="" /> : mark}</span>;
}
