import { Button } from '@/components/button';
import { Card } from '@/components/card';
import { TextField } from '@/components/text-field';
import { splitPath } from '@/utils/path';

export type CipherFile = {
  key: string;
  input: string;
  output: string;
  edited: boolean;
};

type FileListProps = {
  files: CipherFile[];
  onOutput: (key: string, value: string) => void;
  onRemove: (key: string) => void;
};

export function FileList({ files, onOutput, onRemove }: FileListProps) {
  return files.map((file) => (
    <Card key={file.key} title={splitPath(file.input).name}>
      <TextField
        label="输出路径"
        value={file.output}
        required
        onChange={(event) => onOutput(file.key, event.target.value)}
      />
      <Button variant="text" tone="destructive" onClick={() => onRemove(file.key)}>
        移除
      </Button>
    </Card>
  ));
}
