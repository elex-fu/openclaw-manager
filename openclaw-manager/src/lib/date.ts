/**
 * 日期格式化工具
 */

/**
 * 格式化日期为指定格式
 * @param date - 日期对象或时间戳
 * @param format - 格式字符串 (yyyy-MM-dd HH:mm:ss.SSS)
 * @returns 格式化后的字符串
 */
export function format(date: Date | number, format: string): string {
  const d = typeof date === 'number' ? new Date(date) : date;

  const pad = (n: number, len: number = 2) => n.toString().padStart(len, '0');

  const tokens: Record<string, () => string> = {
    yyyy: () => d.getFullYear().toString(),
    MM: () => pad(d.getMonth() + 1),
    dd: () => pad(d.getDate()),
    HH: () => pad(d.getHours()),
    mm: () => pad(d.getMinutes()),
    ss: () => pad(d.getSeconds()),
    SSS: () => pad(d.getMilliseconds(), 3),
  };

  return format.replace(/yyyy|MM|dd|HH|mm|ss|SSS/g, (match) => tokens[match]?.() || match);
}

/**
 * 格式化相对时间
 * @param date - 日期对象或时间戳
 * @returns 相对时间字符串 (如: "刚刚", "5分钟前")
 */
export function formatRelative(date: Date | number): string {
  const d = typeof date === 'number' ? new Date(date) : date;
  const now = new Date();
  const diff = now.getTime() - d.getTime();

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 10) return '刚刚';
  if (seconds < 60) return `${seconds}秒前`;
  if (minutes < 60) return `${minutes}分钟前`;
  if (hours < 24) return `${hours}小时前`;
  if (days < 30) return `${days}天前`;

  return format(d, 'yyyy-MM-dd');
}

/**
 * 格式化日期范围为字符串
 * @param start - 开始日期
 * @param end - 结束日期
 * @returns 格式化后的范围字符串
 */
export function formatRange(start: Date | number, end: Date | number): string {
  const s = typeof start === 'number' ? new Date(start) : start;
  const e = typeof end === 'number' ? new Date(end) : end;

  const sameDay = s.toDateString() === e.toDateString();

  if (sameDay) {
    return `${format(s, 'yyyy-MM-dd')} ${format(s, 'HH:mm')} - ${format(e, 'HH:mm')}`;
  }

  return `${format(s, 'yyyy-MM-dd HH:mm')} - ${format(e, 'yyyy-MM-dd HH:mm')}`;
}

/**
 * 解析日期字符串
 * @param str - 日期字符串
 * @returns Date对象或null
 */
export function parse(str: string): Date | null {
  const date = new Date(str);
  return isNaN(date.getTime()) ? null : date;
}

/**
 * 获取今天的开始时间
 */
export function startOfDay(date: Date = new Date()): Date {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  return d;
}

/**
 * 获取今天的结束时间
 */
export function endOfDay(date: Date = new Date()): Date {
  const d = new Date(date);
  d.setHours(23, 59, 59, 999);
  return d;
}
