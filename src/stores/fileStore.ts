import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FileNode } from '../types/file';

export function useFileStore() {
  const currentFile = ref<{ path: string; name: string } | null>(null);
  const fileContent = ref('');
  const isModified = ref(false);

  const readFile = async (path: string, name?: string) => {
    try {
      const content = await invoke<string>('read_file', { path });
      currentFile.value = {
        path,
        name: name || path.split(/[/\\]/).pop() || 'Untitled'
      };
      fileContent.value = content;
      isModified.value = false;
      return content;
    } catch (error) {
      console.error('Failed to read file:', error);
      throw error;
    }
  };

  const writeFile = async (path: string, content: string) => {
    try {
      await invoke('write_file', { path, content });
      isModified.value = false;
    } catch (error) {
      console.error('Failed to write file:', error);
      throw error;
    }
  };

  const openFileDialog = async () => {
    try {
      const path = await invoke<string | null>('open_file_dialog');
      if (path) {
        currentFile.value = {
          path,
          name: path.split(/[/\\]/).pop() || 'Untitled'
        };
        await readFile(path);
      }
      return path;
    } catch (error) {
      console.error('Failed to open file dialog:', error);
      throw error;
    }
  };

  const saveFileDialog = async (defaultName: string = 'untitled.md') => {
    try {
      const path = await invoke<string | null>('save_file_dialog', { defaultName });
      if (path) {
        currentFile.value = {
          path,
          name: path.split(/[/\\]/).pop() || defaultName
        };
        await writeFile(path, fileContent.value);
      }
      return path;
    } catch (error) {
      console.error('Failed to save file dialog:', error);
      throw error;
    }
  };

  const updateContent = (content: string) => {
    fileContent.value = content;
    isModified.value = true;
  };

  return {
    currentFile: computed(() => currentFile.value),
    fileContent: computed(() => fileContent.value),
    isModified: computed(() => isModified.value),
    readFile,
    writeFile,
    openFileDialog,
    saveFileDialog,
    updateContent
  };
}
