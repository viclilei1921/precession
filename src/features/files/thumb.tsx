import { LockIcon } from '@phosphor-icons/react';
import { useQuery } from '@tanstack/react-query';
import type { RefObject } from 'react';
import { useEffect, useRef, useState } from 'react';
import type { UploadCatalog } from '@/bridge/upload';
import { uploadImage } from '@/bridge/upload';
import styles from './page.module.css';
import { bookFormatLabel, canThumb } from './text';

function imageSource(mime: string, data: string) {
  return `data:${mime};base64,${data}`;
}

function useFileImage(file: UploadCatalog, enabled: boolean) {
  return useQuery({
    queryKey: ['files', 'image', file.id],
    queryFn: () => uploadImage(file.id, file.owner, file.ownerId),
    enabled: enabled && canThumb(file),
    staleTime: Number.POSITIVE_INFINITY
  });
}

/** 列表缩略图。可预览的图片滚进视口才请求字节 */
export function FileThumb({ file }: { file: UploadCatalog }) {
  const ref = useRef<HTMLSpanElement>(null);
  const [seen, setSeen] = useState(false);

  useEffect(() => {
    if (!canThumb(file) || seen) {
      return;
    }
    const node = ref.current;
    if (!node) {
      return;
    }
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        setSeen(true);
        observer.disconnect();
      }
    });
    observer.observe(node);
    return () => observer.disconnect();
  }, [file, seen]);

  const image = useFileImage(file, seen);
  return <ThumbFace file={file} src={image.data ? imageSource(image.data.mime, image.data.data) : ''} nodeRef={ref} />;
}

function ThumbFace({
  file,
  src,
  nodeRef
}: {
  file: UploadCatalog;
  src: string;
  nodeRef: RefObject<HTMLSpanElement | null>;
}) {
  if (file.kind === 'book') {
    return (
      <span
        ref={nodeRef}
        className={styles.thumb}
        data-kind="book"
        data-plate={file.encrypted ? 'encrypted' : undefined}
      >
        {file.encrypted ? <LockIcon weight="regular" /> : bookFormatLabel(file.name)}
      </span>
    );
  }
  if (file.ownerSealed) {
    return (
      <span ref={nodeRef} className={styles.thumb} data-plate="sealed">
        <LockIcon weight="regular" />
      </span>
    );
  }
  if (file.encrypted) {
    return (
      <span ref={nodeRef} className={styles.thumb} data-plate="encrypted">
        <LockIcon weight="regular" />
      </span>
    );
  }
  if (file.kind !== 'image') {
    return (
      <span ref={nodeRef} className={styles.thumb}>
        {file.kind === 'video' ? '视频' : '文件'}
      </span>
    );
  }
  if (src) {
    return (
      <span ref={nodeRef} className={styles.thumb} data-kind="image">
        <img src={src} alt="" />
      </span>
    );
  }
  return <span ref={nodeRef} className={styles.thumb} data-kind="image" />;
}
