import { useEffect, useState } from 'react';
import type { OwnerKind } from '@/bridge/upload';
import { uploadDisplayUrl } from './image';

type UploadThumbProps = {
  id: string;
  owner: OwnerKind;
  ownerId: string;
  className?: string;
};

function showable(url: string) {
  return url.startsWith('blob:') || url.startsWith('http://') || url.startsWith('https://');
}

/** 卡片上的一张已导入图片。还在读取或读不到时留一块底 */
export function UploadThumb({ id, owner, ownerId, className }: UploadThumbProps) {
  const [src, setSrc] = useState('');

  useEffect(() => {
    let alive = true;
    setSrc('');
    void uploadDisplayUrl(`upload:${id}`, { owner, ownerId }).then((url) => {
      if (alive) {
        setSrc(showable(url) ? url : '');
      }
    });
    return () => {
      alive = false;
    };
  }, [id, owner, ownerId]);

  if (!src) {
    return <span className={className} />;
  }
  return <img className={className} src={src} alt="" />;
}
