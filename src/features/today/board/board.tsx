import type { ReactNode } from 'react';
import { createContext, useCallback, useContext, useLayoutEffect, useRef, useState } from 'react';
import styles from './board.module.css';
import type { Box } from './pack';
import { columnSpec, GAP, groupWidthFor, shortestColumn, yearColumns } from './pack';

type Relayout = (id: string, signature: string) => void;

const RelayoutContext = createContext<Relayout>(() => {});

/** 内容高度变化时通知瀑布流重排 */
export function useCardLayout(id: string, signature: string) {
  const report = useContext(RelayoutContext);

  useLayoutEffect(() => {
    report(id, signature);
  }, [id, signature, report]);
}

type TileItem = {
  id: string;
  node: ReactNode;
  /** 整张卡不随按下位移，例如里面还有自己的按钮 */
  static?: boolean;
};

type BoardProps = {
  /** 日历组最小宽度。年视图用更宽的下限。 */
  calendarMin: number;
  /** 列数按日历组将得到的宽度算好再交给年历 */
  group: (columns: number) => ReactNode;
  tiles: TileItem[];
};

function writeBox(element: HTMLElement, box: Box) {
  element.style.left = `${box.left}px`;
  element.style.top = `${box.top}px`;
  element.style.width = `${box.width}px`;
  element.style.height = `${box.height}px`;
}

/** 先按目标宽度量高，高度仍是 auto，方便读到真实内容高度。 */
function measureBox(element: HTMLElement, left: number, top: number, width: number): Box {
  element.style.transition = 'none';
  element.style.left = `${left}px`;
  element.style.top = `${top}px`;
  element.style.width = `${width}px`;
  element.style.height = 'auto';
  return { left, top, width, height: element.offsetHeight };
}

function readBox(element: HTMLElement): Box {
  return {
    left: element.offsetLeft,
    top: element.offsetTop,
    width: element.offsetWidth,
    height: element.offsetHeight
  };
}

/**
 * 日历组占一块，模块卡按最短列往下排。
 *
 * 首次摆好之后才打开过渡，避免进页面时从角落飞入。
 */
export function Board({ calendarMin, group, tiles }: BoardProps) {
  const boardRef = useRef<HTMLDivElement>(null);
  const groupRef = useRef<HTMLDivElement>(null);
  const tileRefs = useRef<(HTMLDivElement | null)[]>([]);
  const signatures = useRef(new Map<string, string>());
  const ticketRef = useRef(0);
  const readyRef = useRef(false);
  const [width, setWidth] = useState(0);
  const [stamp, setStamp] = useState('');

  const report = useCallback((id: string, signature: string) => {
    if (signatures.current.get(id) === signature) {
      return;
    }
    signatures.current.set(id, signature);
    const next = [...signatures.current.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, value]) => `${key}:${value}`)
      .join('|');
    setStamp(next);
  }, []);

  useLayoutEffect(() => {
    const node = boardRef.current;
    if (!node) {
      return;
    }
    const observer = new ResizeObserver(() => {
      const next = node.clientWidth;
      setWidth((current) => (current === next ? current : next));
    });
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useLayoutEffect(() => {
    const board = boardRef.current;
    const groupNode = groupRef.current;
    if (!board || !groupNode) {
      return;
    }
    if (board.clientWidth !== width) {
      setWidth(board.clientWidth);
      return;
    }
    if (width <= 0) {
      return;
    }
    // 把内容签名记在节点上，卡片变高时这个效果才会重跑。
    board.dataset.layout = stamp;

    const ticket = ++ticketRef.current;
    let frame = 0;
    const tileNodes = tileRefs.current.filter((node): node is HTMLDivElement => node != null);
    const nodes = [groupNode, ...tileNodes];
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const origin = readyRef.current && !reduce ? new Map(nodes.map((element) => [element, readBox(element)])) : null;
    const originBoard = board.offsetHeight;
    const spec = columnSpec(width, calendarMin);

    const groupBox = measureBox(groupNode, 0, 0, spec.calendarWidth);
    const tops = spec.columns.map(() => (spec.sideBySide ? 0 : groupBox.height + GAP));
    const tileBoxes = tileNodes.map((tile) => {
      const column = shortestColumn(tops);
      const place = spec.columns[column];
      const box = measureBox(tile, place.left, tops[column], place.width);
      tops[column] += box.height + GAP;
      return box;
    });
    const bottoms = tops.map((value) => value - GAP);
    const boardHeight = spec.sideBySide
      ? Math.max(groupBox.height, ...bottoms, 0)
      : Math.max(...bottoms, groupBox.height);

    writeBox(groupNode, groupBox);
    for (let index = 0; index < tileNodes.length; index += 1) {
      writeBox(tileNodes[index], tileBoxes[index]);
    }
    board.style.height = `${boardHeight}px`;

    const destination = new Map<HTMLElement, Box>([[groupNode, groupBox]]);
    for (let index = 0; index < tileNodes.length; index += 1) {
      destination.set(tileNodes[index], tileBoxes[index]);
    }

    if (!origin) {
      readyRef.current = true;
      for (const element of nodes) {
        element.style.transition = '';
      }
      board.style.transition = '';
      board.classList.add(styles.ready);
      return;
    }

    for (const element of nodes) {
      const box = origin.get(element);
      if (!box) {
        continue;
      }
      element.style.transition = 'none';
      writeBox(element, box);
    }
    board.style.transition = 'none';
    board.style.height = `${originBoard}px`;
    board.getBoundingClientRect();
    frame = requestAnimationFrame(() => {
      if (ticket !== ticketRef.current) {
        return;
      }
      for (const element of nodes) {
        const box = destination.get(element);
        element.style.transition = '';
        if (box) {
          writeBox(element, box);
        }
      }
      board.style.transition = '';
      board.style.height = `${boardHeight}px`;
      board.classList.add(styles.ready);
    });

    return () => {
      if (frame !== 0) {
        cancelAnimationFrame(frame);
      }
    };
  }, [width, calendarMin, stamp]);

  const columns = yearColumns(groupWidthFor(width, calendarMin));

  return (
    <RelayoutContext.Provider value={report}>
      <div ref={boardRef} className={styles.board}>
        <div ref={groupRef} className={styles.group}>
          {group(columns)}
        </div>
        {tiles.map((tile, index) => (
          <div
            key={tile.id}
            ref={(node) => {
              tileRefs.current[index] = node;
            }}
            className={styles.tile}
            data-static={tile.static ? 'true' : undefined}
          >
            {tile.node}
          </div>
        ))}
      </div>
    </RelayoutContext.Provider>
  );
}
