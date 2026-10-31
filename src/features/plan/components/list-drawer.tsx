import { TrashIcon } from '@phosphor-icons/react';
import { useMutation } from '@tanstack/react-query';
import type { CSSProperties, FormEvent } from 'react';
import { useState } from 'react';
import type { Plan, PlanGroup } from '@/bridge/plan';
import { planGroupCreate, planGroupDelete } from '@/bridge/plan';
import { IconButton } from '@/components/button/icon';
import { TextField } from '@/components/text-field';
import { errorMessage } from '@/utils/error';
import styles from '../drawer.module.css';
import { groupHue } from '../layout';
import { DrawerAdd } from './drawer-frame';

const HUES = [42, 305, 200, 20, 140];

type ListDrawerProps = {
  groups: PlanGroup[];
  plans: Plan[];
  systemId?: string;
  selectedId: string | null;
  onPick: (id: string) => void;
  onCreated: () => void;
  onDeleted: (id: string) => void;
  /** 放进底部面板时去掉自带内边距 */
  flush?: boolean;
};

/** 清单。点一次过滤，再点一次看全部。默认清单不能删 */
export function ListDrawer({
  groups,
  plans,
  systemId,
  selectedId,
  onPick,
  onCreated,
  onDeleted,
  flush
}: ListDrawerProps) {
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState('');
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const remove = useMutation({
    mutationFn: (id: string) => planGroupDelete(id),
    onSuccess: (_result, id) => {
      setConfirmId(null);
      onDeleted(id);
    }
  });
  const confirming = groups.find((group) => group.id === confirmId);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const next = name.trim();
    if (!next) {
      return;
    }
    const used = new Set(groups.map((group) => Number(group.color)));
    const hue = HUES.find((item) => !used.has(item)) ?? (groups.length * 47) % 360;
    await planGroupCreate({ name: next, color: String(hue), sort: groups.length });
    setName('');
    setAdding(false);
    onCreated();
  }

  return (
    <div className={styles.body} data-flush={flush ? 'true' : undefined}>
      <div className={styles.list}>
        {groups.map((group) => {
          const locked = group.system || group.id === systemId;
          const count = plans.filter(
            (plan) => plan.groupId === group.id || (group.id === systemId && plan.groupId == null)
          ).length;
          const armed = confirmId === group.id;
          return (
            <div
              key={group.id}
              className={styles.row}
              data-on={selectedId === group.id ? 'true' : undefined}
              style={{ '--hue': groupHue(group.color) } as CSSProperties}
            >
              <button
                type="button"
                className={styles.pick}
                onClick={() => {
                  setConfirmId(null);
                  onPick(group.id);
                }}
              >
                <i className={styles.dot} />
                <span className={styles.name}>
                  <span>{group.name}</span>
                  {locked ? <i className={styles.tag}>默认</i> : null}
                </span>
                <span className={styles.count}>{count}</span>
              </button>
              {locked ? null : (
                <IconButton
                  className={styles.listRemove}
                  label={armed ? '确认删除' : '删除清单'}
                  size="compact"
                  tone="destructive"
                  disabled={remove.isPending}
                  onClick={() => {
                    if (!armed) {
                      setConfirmId(group.id);
                      return;
                    }
                    remove.mutate(group.id);
                  }}
                >
                  <TrashIcon weight="regular" />
                </IconButton>
              )}
            </div>
          );
        })}
      </div>
      {confirming ? (
        <p className={styles.hint}>再点一次就会删掉「{confirming.name}」。里面的计划会回到默认清单。</p>
      ) : null}
      {remove.error ? <p className={styles.error}>{errorMessage(remove.error)}</p> : null}
      {adding ? (
        <form className={styles.newList} onSubmit={(event) => void submit(event)}>
          <TextField
            label="清单名称"
            value={name}
            maxLength={16}
            onChange={(event) => setName(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Escape') {
                setAdding(false);
              }
            }}
          />
        </form>
      ) : (
        <DrawerAdd
          onClick={() => {
            setConfirmId(null);
            setAdding(true);
          }}
        >
          + 清单
        </DrawerAdd>
      )}
    </div>
  );
}
