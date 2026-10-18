function pad(value: number, length = 2) {
  return String(value).padStart(length, '0');
}

/** 秒数拆成时、分、秒、毫秒。最长 99:59:59.999，保证输入框始终是 `HH:MM:SS.mmm`。 */
export function splitSeconds(total: number) {
  const safe = Number.isFinite(total) && total > 0 ? total : 0;
  const capped = Math.min(safe, 99 * 3600 + 59 * 60 + 59.999);
  const whole = Math.floor(capped);
  const hour = Math.floor(whole / 3600);
  const minute = Math.floor((whole % 3600) / 60);
  const second = whole % 60;
  const millisecond = Math.min(999, Math.floor((capped - whole) * 1000));
  return {
    hour: pad(hour),
    minute: pad(minute),
    second: pad(second),
    millisecond: pad(millisecond, 3)
  };
}

export function joinTime(parts: { hour: string; minute: string; second: string; millisecond: string }) {
  return `${parts.hour}:${parts.minute}:${parts.second}.${parts.millisecond}`;
}

export function secondsOf(parts: { hour: string; minute: string; second: string; millisecond: string }) {
  return (
    Number(parts.hour) * 3600 + Number(parts.minute) * 60 + Number(parts.second) + Number(parts.millisecond) / 1000
  );
}

/** `HH:MM:SS.mmm`，给裁剪任务的开始时间和时长用。 */
export function formatTime(total: number) {
  return joinTime(splitSeconds(total));
}

/** 把 `HH:MM:SS` 或 `HH:MM:SS.mmm` 还原成秒。无法识别时返回 0。 */
export function parseTime(value: string) {
  const match = /^(\d+):(\d{1,2}):(\d{1,2})(?:\.(\d{1,3}))?$/.exec(value.trim());
  if (!match) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }
  const fraction = (match[4] ?? '0').padEnd(3, '0');
  return Number(match[1]) * 3600 + Number(match[2]) * 60 + Number(match[3]) + Number(fraction) / 1000;
}
