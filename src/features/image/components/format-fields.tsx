import { Card } from '@/components/card';
import { TextField } from '@/components/text-field';
import styles from './format-fields.module.css';

type ImageFormat = 'avif' | 'jxl';

type FormatFieldsProps = {
  format: ImageFormat;
  quality: string;
  speed: string;
  onQuality: (value: string) => void;
  onSpeed: (value: string) => void;
};

/** AVIF 用质量，JXL 用距离。 */
export function FormatFields({ format, quality, speed, onQuality, onSpeed }: FormatFieldsProps) {
  return (
    <Card title={format === 'avif' ? 'AVIF' : 'JPEG XL'}>
      <div className={styles.pair}>
        <TextField
          label={format === 'avif' ? '质量' : '距离'}
          value={quality}
          inputMode="decimal"
          onChange={(event) => onQuality(event.target.value)}
        />
        <TextField
          label={format === 'avif' ? '速度' : 'effort'}
          value={speed}
          inputMode="numeric"
          hint={format === 'avif' ? '0 到 10' : '1 到 10'}
          onChange={(event) => onSpeed(event.target.value)}
        />
      </div>
    </Card>
  );
}
