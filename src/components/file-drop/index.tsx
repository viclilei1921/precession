import { FileArrowDownIcon, FileIcon, FolderOpenIcon } from '@phosphor-icons/react';
import clsx from 'clsx';
import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
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

/** 选择文件或文件夹，或把它们拖进窗口。文件夹会连同子文件夹里符合的文件一起加入。拖入时在标题栏下盖一层提示。 */
export function FileDrop({ accept = [], multiple = true, className, onFiles }: FileDropProps) {
  const { t } = useTranslation('ui');
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
              setNotice(t('fileDrop.noMatch'));
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
  }, [t]);

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
        setNotice(t('fileDrop.noMatch'));
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
          {t('fileDrop.chooseFiles')}
        </Button>
        <Button variant="outlined" disabled={pending} onClick={() => void addDirectories()}>
          <FolderOpenIcon weight="regular" />
          {t('fileDrop.chooseFolder')}
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
                <p className={styles.lead}>{t('fileDrop.release')}</p>
                <p className={styles.caption}>
                  {dragCount > 1 ? t('fileDrop.caption', { count: dragCount }) : t('fileDrop.captionGeneric')}
                </p>
              </div>
            </div>,
            document.body
          )
        : null}
    </div>
  );
}
