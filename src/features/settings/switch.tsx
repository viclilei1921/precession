import styles from './switch.module.css';

type SwitchProps = {
  label: string;
  checked: boolean;
  disabled?: boolean;
  onClick: () => void;
};

/** 设置行右侧的开关。 */
export function Switch({ label, checked, disabled, onClick }: SwitchProps) {
  return (
    <button
      type="button"
      className={styles.switch}
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
    >
      <span />
    </button>
  );
}
