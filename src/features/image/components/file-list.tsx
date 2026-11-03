import { FileImageIcon, TrashIcon } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/button';
import { IconButton } from '@/components/button/icon';
import { Checkbox } from '@/components/checkbox';
import { Scroll } from '@/components/scroll';
import { formatBytes } from '@/features/files/text';
import { splitPath } from '@/utils/path';
import styles from './file-list.module.css';

export type ImageFile = {
  key: string;
  input: string;
  size: number | null;
  checked: boolean;
};

type FileListProps = {
  files: ImageFile[];
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
  const { t } = useTranslation('image');
  const selected = files.filter((file) => file.checked);
  const total = selected.reduce((sum, file) => sum + (file.size ?? 0), 0);
  const empty = files.length === 0;

  return (
    <>
      <Scroll className={styles.sheet} axis="vertical" gutter>
        <div className={styles.head}>
          <span />
          <span>{t('files.fileName')}</span>
          <span className={styles.size}>{t('files.size')}</span>
          <span>{t('files.path')}</span>
          <span className={styles.action}>{t('files.action')}</span>
        </div>
        <div className={styles.body}>
          {empty ? <p className={styles.note}>{t('files.empty')}</p> : null}
          {files.map((file) => {
            const { name } = splitPath(file.input);
            return (
              <div className={styles.row} key={file.key}>
                <Checkbox
                  checked={file.checked}
                  aria-label={t('files.selectAria', { name })}
                  onChange={(event) => onChecked(file.key, event.target.checked)}
                />
                <span className={styles.name} title={name}>
                  <FileImageIcon weight="regular" />
                  <span className={styles.label}>{name}</span>
                </span>
                <span className={styles.size}>{file.size == null ? '—' : formatBytes(file.size)}</span>
                <span className={styles.path} title={file.input}>
                  {file.input}
                </span>
                <IconButton
                  className={styles.remove}
                  label={t('files.remove')}
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
            {t('files.selectAll')}
          </Button>
          <Button variant="text" disabled={empty} onClick={onInvert}>
            {t('files.invert')}
          </Button>
          <Button variant="text" disabled={empty} onClick={onClear}>
            {t('files.clear')}
          </Button>
          <Button variant="text" tone="destructive" disabled={selected.length === 0} onClick={onDeleteSelected}>
            {t('files.delete')}
          </Button>
        </div>
        <p className={styles.total}>{t('files.selected', { count: selected.length, size: formatBytes(total) })}</p>
      </div>
    </>
  );
}
