import { invoke } from '@tauri-apps/api';
import type {
  ApiResponse,
  FileItem,
  FileScanRequest,
  FileScanResult,
  Group,
  GroupWithFiles,
  Plugin,
} from '@/types';

// Config API
export const configApi = {
  get: (key: string) =>
    invoke<ApiResponse<{ key: string; value: string } | null>>('get_config', { key }),
  set: (key: string, value: string, description?: string) =>
    invoke<ApiResponse<{ key: string; value: string }>>('set_config', {
      req: { key, value, description },
    }),
  delete: (key: string) =>
    invoke<ApiResponse<boolean>>('delete_config', { key }),
};

// File API
export const fileApi = {
  scan: (req: FileScanRequest) =>
    invoke<ApiResponse<FileScanResult>>('scan_files', { req }),
  getAll: (params?: {
    file_type?: string;
    is_collected?: boolean;
    is_classified?: boolean;
    limit?: number;
    offset?: number;
  }) => invoke<ApiResponse<FileItem[]>>('get_files', params || {}),
  getById: (id: string) =>
    invoke<ApiResponse<FileItem | null>>('get_file_by_id', { id }),
  update: (id: string, data: Partial<FileItem>) =>
    invoke<ApiResponse<FileItem>>('update_file', {
      req: { id, ...data },
    }),
  delete: (id: string) =>
    invoke<ApiResponse<boolean>>('delete_file', { id }),
  parse: (fileName: string) =>
    invoke<ApiResponse<{ file_name: string; parsed_data: unknown }>>('parse_file_info', {
      fileName,
    }),
};

// Group API
export const groupApi = {
  getAll: (withFiles?: boolean) =>
    invoke<ApiResponse<GroupWithFiles[]>>('get_groups', { withFiles }),
  create: (name: string, description?: string, icon?: string, color?: string) =>
    invoke<ApiResponse<Group>>('create_group', {
      req: { name, description, icon, color },
    }),
  update: (id: string, data: Partial<Group>) =>
    invoke<ApiResponse<Group>>('update_group', {
      req: { id, ...data },
    }),
  delete: (id: string) =>
    invoke<ApiResponse<boolean>>('delete_group', { id }),
  addFile: (groupId: string, fileId: string) =>
    invoke<ApiResponse<boolean>>('add_file_to_group', {
      req: { group_id: groupId, file_id: fileId },
    }),
  removeFile: (groupId: string, fileId: string) =>
    invoke<ApiResponse<boolean>>('remove_file_from_group', {
      groupId,
      fileId,
    }),
};

// Plugin API
export const pluginApi = {
  getAll: () => invoke<ApiResponse<Plugin[]>>('get_plugins'),
  install: (marketItemId: string, downloadUrl: string) =>
    invoke<ApiResponse<Plugin>>('install_plugin', {
      req: { market_item_id: marketItemId, download_url: downloadUrl },
    }),
  uninstall: (id: string) =>
    invoke<ApiResponse<boolean>>('uninstall_plugin', { id }),
  enable: (id: string) =>
    invoke<ApiResponse<Plugin>>('enable_plugin', { id }),
  disable: (id: string) =>
    invoke<ApiResponse<Plugin>>('disable_plugin', { id }),
};
