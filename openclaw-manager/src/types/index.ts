export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export interface FileItem {
  id: string;
  file_name: string;
  file_path: string;
  file_type: string;
  file_size: number;
  description?: string;
  tags?: string;
  is_collected: boolean;
  is_classified: boolean;
  classification?: string;
  custom_attributes?: string;
  created_at: string;
  updated_at: string;
}

export interface Group {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  sort_order: number;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface GroupWithFiles extends Group {
  files: FileItem[];
  file_count: number;
}

export interface Plugin {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  plugin_type: string;
  entry_point: string;
  is_enabled: boolean;
  config_schema?: string;
  default_config?: string;
  created_at: string;
  updated_at: string;
}

export interface FileScanRequest {
  path: string;
  recursive: boolean;
  file_types?: string[];
}

export interface FileScanResult {
  files: FileItem[];
  total_count: number;
  total_size: number;
}
