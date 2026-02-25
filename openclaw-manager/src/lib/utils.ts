import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function getFileIcon(fileType: string): string {
  const type = fileType.toLowerCase();
  if (['mp4', 'avi', 'mkv', 'mov', 'wmv'].includes(type)) return 'video';
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp'].includes(type)) return 'image';
  if (['mp3', 'wav', 'flac', 'aac', 'ogg'].includes(type)) return 'audio';
  if (['txt', 'md', 'doc', 'docx', 'pdf'].includes(type)) return 'document';
  return 'file';
}
