import { FileArrowDownIcon, FileIcon, FolderOpenIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { collectFiles, onFileDrop, pickDirectories, pickFiles } from '@/bridge';
import { Button } from '@/components/button';
import { errorMessage } from '@/utils/error';
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

function dropCaption(count: number) {
  if (count > 1) {
    return `${count} 项，包含子文件夹里的文件`;
  }
  return '文件，以及文件夹里的文件';
}

/** 选择文件或文件夹，或把它们拖进窗口。文件夹会连同子文件夹里符合的文件一起加入。拖入时在标题栏下盖一层提示。 */
export function FileDrop({ accept = [], multiple = true, className, onFiles }: FileDropProps) {
  const [hovering, setHovering] = useState(false);
  const [dragCount, setDragCount] = useState(0);
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState('');
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
        void collectFiles(event.paths, acceptRef.current)
          .then((found) => {
            const next = takePaths(found.files, acceptRef.current, multipleRef.current);
            if (next.length > 0) {
              setNotice('');
              onFilesRef.current(next);
              return;
            }
            if (found.scannedDirectory) {
              setNotice('这个文件夹里没有符合的文件。');
            }
          })
          .catch((error: unknown) => {
            setNotice(errorMessage(error));
          });
        return;
      }
      if (event.phase === 'enter') {
        setDragCount(event.paths.length);
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

  async function addDirectories() {
    const directories = await pickDirectories();
    if (directories.length === 0) {
      return;
    }
    setNotice('');
    setPending(true);
    try {
      const found = await collectFiles(directories, accept);
      const next = takePaths(found.files, accept, multiple);
      if (next.length === 0) {
        setNotice('这个文件夹里没有符合的文件。');
        return;
      }
      onFiles(next);
    } catch (error: unknown) {
      setNotice(errorMessage(error));
    } finally {
      setPending(false);
    }
  }

  return (
    <div className={clsx(styles.root, className)}>
      <div className={styles.actions}>
        <Button
          variant="outlined"
          disabled={pending}
          onClick={() => {
            setNotice('');
            void pickFiles({ multiple, extensions: accept })
              .then((paths) => {
                const next = takePaths(paths, accept, multiple);
                if (next.length > 0) {
                  onFiles(next);
                }
              })
              .catch((error: unknown) => {
                setNotice(errorMessage(error));
              });
          }}
        >
          <FileIcon weight="regular" />
          选择文件
        </Button>
        <Button variant="outlined" disabled={pending} onClick={() => void addDirectories()}>
          <FolderOpenIcon weight="regular" />
          选择文件夹
        </Button>
      </div>
      {notice ? <p className={styles.notice}>{notice}</p> : null}
      {hovering
        ? createPortal(
            <div className={styles.overlay} role="status" aria-live="polite">
              <div className={styles.callout}>
                <span className={styles.mark} aria-hidden="true">
                  <FileArrowDownIcon className={styles.glyph} weight="regular" />
                </span>
                <p className={styles.lead}>松开以添加</p>
                <p className={styles.caption}>{dropCaption(dragCount)}</p>
              </div>
            </div>,
            document.body
          )
        : null}
    </div>
  );
}
