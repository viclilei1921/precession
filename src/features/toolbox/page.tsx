import styles from './page.module.css';

export function ToolboxPage() {
  return (
    <section className={styles.page}>
      <h1 className={styles.title}>工具箱</h1>
      <p className={styles.hint}>图片视频的处理和单件加解密，以后从这里进入。</p>
    </section>
  );
}
