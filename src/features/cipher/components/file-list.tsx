import { FileIcon, TrashIcon } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Checkbox } from '@/components/checkbox';
import { Scroll } from '@/components/scroll';
import { formatBytes } from '@/features/files/text';
import { splitPath } from '@/utils/path';
import styles from './file-list.module.css';

export type CipherFile = {
  key: string;
  input: string;
  size: number | null;
  checked: boolean;
};

type FileListProps = {
  files: CipherFile[];
  onChecked: (key: string, checked: boolean) => void;
  onRemove: (key: string) => void;
  onSelectAll: () => void;
  onInvert: () => void;
  onClear: () => void;
  onDeleteSelected: () => void;
};

export function FileList({
  files,
  onChecked,
  onRemove,
  onSelectAll,
  onInvert,
  onClear,
  onDeleteSelected
}: FileListProps) {
  const { t } = useTranslation(['cipher', 'common']);
  const selected = files.filter((file) => file.checked);
  const total = selected.reduce((sum, file) => sum + (file.size ?? 0), 0);
  const empty = files.length === 0;

  return (
    <>
      <Scroll className={styles.sheet} axis="vertical" gutter>
        <div className={styles.head}>
          <span />
          <span>{t('list.name')}</span>
          <span className={styles.size}>{t('list.size')}</span>
          <span>{t('list.path')}</span>
          <span className={styles.action}>{t('list.actions')}</span>
        </div>
        <div className={styles.body}>
          {empty ? <p className={styles.note}>{t('list.empty')}</p> : null}
          {files.map((file) => {
            const { name } = splitPath(file.input);
            return (
              <div className={styles.row} key={file.key}>
                <Checkbox
                  checked={file.checked}
                  aria-label={t('list.selectFile', { name })}
                  onChange={(event) => onChecked(file.key, event.target.checked)}
                />
                <span className={styles.name} title={name}>
                  <FileIcon weight="regular" />
                  <span className={styles.label}>{name}</span>
                </span>
                <span className={styles.size}>{file.size == null ? '—' : formatBytes(file.size)}</span>
                <span className={styles.path} title={file.input}>
                  {file.input}
                </span>
                <IconButton
                  className={styles.remove}
                  label={t('list.remove')}
                  tone="destructive"
                  size="compact"
                  onClick={() => onRemove(file.key)}
                >
                  <TrashIcon weight="regular" />
                </IconButton>
              </div>
            );
          })}
        </div>
      </Scroll>
      <div className={styles.foot}>
        <div className={styles.actions}>
          <Button variant="text" disabled={empty} onClick={onSelectAll}>
            {t('list.selectAll')}
          </Button>
          <Button variant="text" disabled={empty} onClick={onInvert}>
            {t('list.invert')}
          </Button>
          <Button variant="text" disabled={empty} onClick={onClear}>
            {t('list.clear')}
          </Button>
          <Button variant="text" tone="destructive" disabled={selected.length === 0} onClick={onDeleteSelected}>
            {t('common:button.delete')}
          </Button>
        </div>
        <p className={styles.total}>{t('list.summary', { count: selected.length, size: formatBytes(total) })}</p>
      </div>
    </>
  );
}
