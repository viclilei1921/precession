import { FileImageIcon, TrashIcon } from '@phosphor-icons/react';
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
  const selected = files.filter((file) => file.checked);
  const total = selected.reduce((sum, file) => sum + (file.size ?? 0), 0);
  const empty = files.length === 0;

  return (
    <>
      <Scroll className={styles.sheet} axis="vertical" gutter>
        <div className={styles.head}>
          <span />
          <span>文件名</span>
          <span className={styles.size}>大小</span>
          <span>路径</span>
          <span className={styles.action}>操作</span>
        </div>
        <div className={styles.body}>
          {empty ? <p className={styles.note}>还没有图片。选择文件或文件夹，也可以拖进来。</p> : null}
          {files.map((file) => {
            const { name } = splitPath(file.input);
            return (
              <div className={styles.row} key={file.key}>
                <Checkbox
                  checked={file.checked}
                  aria-label={`选择 ${name}`}
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
                  label="移除"
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
            全选
          </Button>
          <Button variant="text" disabled={empty} onClick={onInvert}>
            反选
          </Button>
          <Button variant="text" disabled={empty} onClick={onClear}>
            清空
          </Button>
          <Button variant="text" tone="destructive" disabled={selected.length === 0} onClick={onDeleteSelected}>
            删除
          </Button>
        </div>
        <p className={styles.total}>
          已选 {selected.length} 个 · 总大小 {formatBytes(total)}
        </p>
      </div>
    </>
  );
}
