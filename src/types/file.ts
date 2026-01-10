export interface FileNode {
  id: string;
  name: string;
  path: string;
  type: 'file' | 'directory';
  level?: number;
  children?: FileNode[];
  isExpanded?: boolean;
}
