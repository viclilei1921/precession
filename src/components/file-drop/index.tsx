import { FolderOpenIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { useEffect, useRef, useState } from 'react';
import { onFileDrop, pickFiles } from '@/bridge';
import { Button } from '@/components/button';
import styles from './index.module.css';

type FileDropProps = {
  /** 不带点的扩展名。空数组表示接受任意文件。 */
  accept?: string[];
  multiple?: boolean;
  className?: string;
  onFiles: (paths: string[]) => void;
};

function matches(path: string, accept: string[]) {
  if (accept.length === 0) {
    return true;
  }
  const lower = path.toLowerCase();
  return accept.some((ext) => lower.endsWith(`.${ext.toLowerCase()}`));
}

function takePaths(paths: string[], accept: string[], multiple: boolean) {
  const matched = paths.filter((path) => matches(path, accept));
  return multiple ? matched : matched.slice(0, 1);
}

/** 选择文件，或把文件拖进窗口。拖入时盖一层提示。 */
export function FileDrop({ accept = [], multiple = true, className, onFiles }: FileDropProps) {
  const [hovering, setHovering] = useState(false);
  const acceptRef = useRef(accept);
  const multipleRef = useRef(multiple);
  const onFilesRef = useRef(onFiles);
  acceptRef.current = accept;
  multipleRef.current = multiple;
  onFilesRef.current = onFiles;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void onFileDrop((event) => {
      if (event.phase === 'leave') {
        setHovering(false);
        return;
      }
      if (event.phase === 'drop') {
        setHovering(false);
        const paths = takePaths(event.paths, acceptRef.current, multipleRef.current);
        if (paths.length > 0) {
          onFilesRef.current(paths);
        }
        return;
      }
      setHovering(true);
    }).then((stop) => {
      if (cancelled) {
        stop();
        return;
      }
      unlisten = stop;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return (
    <div className={clsx(styles.root, className)}>
      <Button
        type="button"
        variant="outlined"
        onClick={() => {
          void pickFiles({ multiple, extensions: accept }).then((paths) => {
            const next = takePaths(paths, accept, multiple);
            if (next.length > 0) {
              onFiles(next);
            }
          });
        }}
      >
        <FolderOpenIcon weight="regular" />
        选择文件
      </Button>
      {hovering ? (
        <div className={styles.overlay}>
          <p className={styles.overlayLabel}>松开以添加文件</p>
        </div>
      ) : null}
    </div>
  );
}
