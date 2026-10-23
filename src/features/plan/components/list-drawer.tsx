import type { CSSProperties, FormEvent } from 'react';
import { useState } from 'react';
import type { Plan, PlanGroup } from '@/bridge/plan';
import { planGroupCreate } from '@/bridge/plan';
import { TextField } from '@/components/text-field';
import styles from '../drawer.module.css';
import { groupHue } from '../layout';

const HUES = [42, 305, 200, 20, 140];

type ListDrawerProps = {
  groups: PlanGroup[];
  plans: Plan[];
  systemId?: string;
  selectedId: string | null;
  onPick: (id: string) => void;
  onCreated: () => void;
};

/** 清单。点一次过滤，再点一次看全部 */
export function ListDrawer({ groups, plans, systemId, selectedId, onPick, onCreated }: ListDrawerProps) {
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState('');

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
    <div className={styles.body}>
      <div className={styles.list}>
        {groups.map((group) => {
          const count = plans.filter(
            (plan) => plan.groupId === group.id || (group.id === systemId && plan.groupId == null)
          ).length;
          return (
            <button
              key={group.id}
              type="button"
              className={styles.row}
              data-on={selectedId === group.id ? 'true' : undefined}
              style={{ '--hue': groupHue(group.color) } as CSSProperties}
              onClick={() => onPick(group.id)}
            >
              <i className={styles.dot} />
              <span className={styles.name}>
                <span>{group.name}</span>
                {group.system ? <i className={styles.tag}>默认</i> : null}
              </span>
              <span className={styles.count}>{count}</span>
            </button>
          );
        })}
      </div>
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
        <button type="button" className={styles.add} onClick={() => setAdding(true)}>
          + 清单
        </button>
      )}
    </div>
  );
}
