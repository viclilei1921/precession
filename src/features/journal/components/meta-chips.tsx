import { PlusIcon } from '@phosphor-icons/react';
import { useState } from 'react';
import { ChipButton } from '@/components/button/chip';
import { IconButton } from '@/components/button/icon';
import { TextField } from '@/components/text-field';
import { errorMessage } from '@/utils/error';
import styles from './meta-chips.module.css';

type Named = {
  id: string;
  name: string;
};

type MetaChipsProps = {
  items: Named[];
  selected: string[];
  onChange: (ids: string[]) => void;
  /** 为真时才露出选择和新建 */
  adding: boolean;
  empty: string;
  addLabel?: string;
  onAdd?: (name: string) => Promise<void>;
  listError?: string;
};

/** 已选的名字直接摆出来。添加时才出现可选项和输入框。 */
export function MetaChips({ items, selected, onChange, adding, empty, addLabel, onAdd, listError }: MetaChipsProps) {
  const [name, setName] = useState('');
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  const chosen = items.filter((item) => selected.includes(item.id));
  const rest = items.filter((item) => !selected.includes(item.id));

  function toggle(id: string) {
    onChange(selected.includes(id) ? selected.filter((item) => item !== id) : [...selected, id]);
  }

  async function add() {
    const next = name.trim();
    if (!next || !onAdd) {
      return;
    }
    setPending(true);
    setError('');
    try {
      await onAdd(next);
      setName('');
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setPending(false);
    }
  }

  return (
    <div className={styles.block}>
      {chosen.length > 0 ? (
        <div className={styles.chips}>
          {chosen.map((item) => (
            <ChipButton key={item.id} selected onClick={() => toggle(item.id)}>
              {item.name}
            </ChipButton>
          ))}
        </div>
      ) : adding ? null : (
        <p className={styles.hint}>{empty}</p>
      )}
      {adding ? (
        <>
          {rest.length > 0 ? (
            <div className={styles.chips}>
              {rest.map((item) => (
                <ChipButton key={item.id} onClick={() => toggle(item.id)}>
                  {item.name}
                </ChipButton>
              ))}
            </div>
          ) : null}
          {onAdd ? (
            <div className={styles.add}>
              <TextField
                className={styles.addField}
                label={addLabel}
                value={name}
                disabled={pending}
                onChange={(event) => setName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key !== 'Enter') {
                    return;
                  }
                  event.preventDefault();
                  void add();
                }}
              />
              <IconButton label="添加" disabled={pending || name.trim() === ''} onClick={() => void add()}>
                <PlusIcon weight="regular" />
              </IconButton>
            </div>
          ) : rest.length === 0 ? (
            <p className={styles.hint}>没有可添加的。</p>
          ) : null}
        </>
      ) : null}
      {error ? <p className={styles.error}>{error}</p> : null}
      {listError ? <p className={styles.error}>{listError}</p> : null}
    </div>
  );
}
