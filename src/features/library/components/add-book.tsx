import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import { pickFiles } from '@/bridge/files';
import type { BookStatus } from '@/bridge/library';
import { bookCreate, bookPeek } from '@/bridge/library';
import { taskEnqueue } from '@/bridge/task';
import { Button } from '@/components/button';
import { SegmentButton } from '@/components/button/segment';
import recordStyles from '@/components/record.module.css';
import { Scroll } from '@/components/scroll';
import { Select } from '@/components/select';
import { TextField } from '@/components/text-field';
import drawerStyles from '@/features/plan/drawer.module.css';
import { refreshLibrary } from '@/query/library/query';
import { PATH } from '@/router/path';
import { errorMessage } from '@/utils/error';
import { bookStatusLabel } from '../book';

const EXTENSIONS = ['epub', 'mobi', 'txt', 'azw', 'azw3'];

function titleFromPath(path: string) {
  const name = path.split(/[/\\]/).pop() ?? path;
  return name.replace(/\.(epub|mobi|txt|azw3|azw)$/i, '') || name;
}

/** 从文件导入，或只建一本还没有正文的档案 */
export function AddBook({ onClose }: { onClose: () => void }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [mode, setMode] = useState<'file' | 'manual'>('file');
  const [path, setPath] = useState('');
  const [title, setTitle] = useState('');
  const [author, setAuthor] = useState('');
  const [status, setStatus] = useState<BookStatus>('want');
  const save = useMutation({
    mutationFn: async () => {
      if (mode === 'file') {
        if (!path) {
          throw new Error('先选择一个文件');
        }
        let nextTitle = titleFromPath(path);
        let nextAuthor = '';
        try {
          const meta = await bookPeek(path);
          if (meta.title.trim()) {
            nextTitle = meta.title.trim();
          }
          nextAuthor = meta.author.trim();
        } catch {
          nextTitle = titleFromPath(path);
        }
        const book = await bookCreate({
          title: nextTitle,
          author: nextAuthor,
          coverPath: '',
          status: 'reading',
          progress: 0,
          rating: null,
          startedAt: null,
          finishedAt: null
        });
        const task = await taskEnqueue({
          kind: 'importUpload',
          source: path,
          owner: 'book',
          ownerId: book.id,
          uploadKind: 'book',
          encrypt: false,
          password: ''
        });
        return { bookId: book.id, taskId: task.id };
      }
      if (!title.trim()) {
        throw new Error('先写上书名');
      }
      const book = await bookCreate({
        title: title.trim(),
        author: author.trim(),
        coverPath: '',
        status,
        progress: 0,
        rating: null,
        startedAt: null,
        finishedAt: null
      });
      return { bookId: book.id, taskId: '' };
    },
    onSuccess: async ({ bookId, taskId }) => {
      await refreshLibrary(queryClient);
      onClose();
      void navigate({
        to: PATH.bookReader,
        params: { bookId },
        search: taskId ? { task: taskId } : {}
      });
    }
  });

  return (
    <form
      className={drawerStyles.form}
      onSubmit={(event) => {
        event.preventDefault();
        save.mutate();
      }}
    >
      <Scroll className={drawerStyles.sheet} axis="vertical">
        <div className={drawerStyles.main}>
          <div className={recordStyles.segments}>
            <SegmentButton selected={mode === 'file'} onClick={() => setMode('file')}>
              从文件导入
            </SegmentButton>
            <SegmentButton selected={mode === 'manual'} onClick={() => setMode('manual')}>
              只建档案
            </SegmentButton>
          </div>
          {mode === 'file' ? (
            <>
              <Button
                type="button"
                variant="outlined"
                onClick={() => {
                  void pickFiles({ multiple: false, extensions: EXTENSIONS }).then((files) => {
                    setPath(files[0] ?? '');
                  });
                }}
              >
                选择 EPUB、MOBI 或 TXT
              </Button>
              <p className={recordStyles.note}>
                {path ? titleFromPath(path) : '还没有选择文件。书名会先用文件里的，读不到再用文件名。'}
              </p>
            </>
          ) : (
            <>
              <TextField label="书名" value={title} onChange={(event) => setTitle(event.target.value)} />
              <TextField label="作者" value={author} onChange={(event) => setAuthor(event.target.value)} />
              <Select label="状态" value={status} onChange={(event) => setStatus(event.target.value as BookStatus)}>
                {(Object.keys(bookStatusLabel) as BookStatus[]).map((item) => (
                  <option key={item} value={item}>
                    {bookStatusLabel[item]}
                  </option>
                ))}
              </Select>
            </>
          )}
          {save.error ? <p className={recordStyles.error}>{errorMessage(save.error)}</p> : null}
        </div>
      </Scroll>
      <div className={drawerStyles.foot}>
        <div className={recordStyles.actions}>
          <span className={recordStyles.spacer} />
          <Button type="button" variant="outlined" disabled={save.isPending} onClick={onClose}>
            取消
          </Button>
          <Button type="submit" disabled={save.isPending}>
            保存
          </Button>
        </div>
      </div>
    </form>
  );
}
