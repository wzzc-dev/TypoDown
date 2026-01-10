<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { 
  ChevronRight, 
  FileText, 
  FolderOpen, 
  Folder,
  List,
  Type
} from 'lucide-vue-next';
import type { FileNode } from '../../types/file';
import DocumentOutline from './DocumentOutline.vue';

type ViewMode = 'files' | 'outline';

interface Props {
  currentPath?: string | null;
  content?: string;
}

interface Emits {
  (e: 'select-file', path: string, name: string): void;
  (e: 'change-mode', mode: ViewMode): void;
  (e: 'select-heading', index: number): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const viewMode = ref<ViewMode>('files');
const fileTree = ref<FileNode[]>([]);
const expandedNodes = ref<Set<string>>(new Set());
const loading = ref(false);

const loadFileTree = async () => {
  loading.value = true;
  try {
    const homeDir = await invoke<string>('home_dir');
    const documentsDir = await invoke<string>('documents_dir');

    const startDir = documentsDir + "/TypoDown" || homeDir;
    
    fileTree.value = await scanDirectory(startDir);
  } catch (error) {
    console.error('Failed to load file tree:', error);
  } finally {
    loading.value = false;
  }
};

const scanDirectory = async (path: string): Promise<FileNode[]> => {
  try {
    const entries = await invoke<string[]>('read_dir', { path });
    
    const nodes: FileNode[] = [];
    
    for (const entry of entries) {
      const name = entry.split(/[/\\]/).pop() || entry;
      const isMarkdownFile = name.endsWith('.md') || name.endsWith('.markdown');
      
      const node: FileNode = {
        id: entry,
        name,
        path: entry,
        type: 'directory'
      };

      if (isMarkdownFile) {
        node.type = 'file';
        nodes.push(node);
      } else if (!name.startsWith('.')) {
        nodes.push(node);
      }
    }
    
    nodes.sort((a, b) => {
      if (a.type !== b.type) {
        return a.type === 'directory' ? -1 : 1;
      }
      return a.name.localeCompare(b.name);
    });

    return nodes;
  } catch (error) {
    console.error('Failed to scan directory:', error);
    return [];
  }
};

const toggleExpand = async (node: FileNode) => {
  if (node.type !== 'directory') return;

  if (expandedNodes.value.has(node.id)) {
    expandedNodes.value.delete(node.id);
    if (node.children) {
      delete node.children;
    }
  } else {
    expandedNodes.value.add(node.id);
    if (!node.children && node.type === 'directory') {
      node.children = await scanDirectory(node.path);
    }
  }
};

const selectFile = (node: FileNode) => {
  if (node.type === 'file') {
    emit('select-file', node.path, node.name);
  }
};

const getFileIcon = (node: FileNode) => {
  if (node.type === 'file') {
    return FileText;
  }
  return expandedNodes.value.has(node.id) ? FolderOpen : Folder;
};

const switchMode = (mode: ViewMode) => {
  viewMode.value = mode;
  emit('change-mode', mode);
};

const selectHeading = (index: number) => {
  emit('select-heading', index);
};

onMounted(() => {
  loadFileTree();
});
</script>

<template>
  <div class="h-full flex-1 flex-col bg-white">
    <!-- Header with View Mode Switcher -->
    <div class="h-10 flex-1 items-center justify-between border-gray-100 py-1 px-1">
      <!-- <div class="flex items-center gap-2">
        <Home :size="15" class="text-gray-400 mr-2" />
        <span class="text-xs font-medium text-gray-600 uppercase tracking-wider">Files</span>
      </div> -->
      
      <!-- View Mode Switcher -->
      <div class="flex items-center rounded overflow-hidden">
        <button
          @click="switchMode('files')"
          class="flex-1 py-1.5 text-xs font-medium transition-colors"
          :class="{
            'text-blue-600 bg-blue-50': viewMode === 'files',
            'text-gray-600 hover:bg-gray-200': viewMode !== 'files'
          }"
        >
          <List :size="14" class="mr-1.5" />
          Files
        </button>
        <button
          @click="switchMode('outline')"
          class="flex-1 py-1.5 text-xs font-medium transition-colors"
          :class="{
            'text-blue-600 bg-blue-50': viewMode === 'outline',
            'text-gray-600 hover:bg-gray-200': viewMode !== 'outline'
          }"
        >
          <Type :size="14" class="mr-1.5" />
          Outline
        </button>
      </div>
    </div>

    <!-- Files View -->
    <div v-if="viewMode === 'files'" class="flex-1 overflow-y-auto scrollbar-thin">
      <div v-if="loading" class="p-4 text-center text-sm text-gray-400">
        Loading...
      </div>
      
      <div v-else-if="fileTree.length === 0" class="p-4 text-center text-sm text-gray-400">
        No Markdown files
      </div>

      <div v-else class="py-2">
        <!-- Simple Flat File List -->
        <div
          v-for="node in fileTree"
          :key="node.id"
          class="tree-node"
        >
          <div
            class="flex items-center px-3 py-1.5 cursor-pointer hover:bg-gray-200/50 transition-colors group"
            :class="{
              'bg-blue-50': props.currentPath === node.path
            }"
            @click="node.type === 'file' ? selectFile(node) : toggleExpand(node)"
          >
            <!-- Expand Icon -->
            <span class="w-4 mr-1 flex justify-center">
              <ChevronRight
                v-if="node.type === 'directory'"
                :size="12"
                class="text-gray-400 transition-transform"
                :class="{ 'rotate-90': expandedNodes.has(node.id) }"
              />
            </span>

            <!-- File/Folder Icon -->
            <component
              :is="getFileIcon(node)"
              :size="15"
              class="mr-2 flex-shrink-0"
              :class="{
                'text-blue-400': node.type === 'directory',
                'text-gray-500': node.type === 'file'
              }"
            />

            <!-- File/Folder Name -->
            <span
              class="text-sm truncate flex-1"
              :class="{
                'text-gray-600': node.type === 'directory',
                'text-gray-500': node.type === 'file',
                'font-medium text-blue-600': props.currentPath === node.path
              }"
            >
              {{ node.name }}
            </span>
          </div>

          <!-- Children (Flat) -->
          <div v-if="node.type === 'directory' && expandedNodes.has(node.id) && node.children" class="ml-4">
            <div
              v-for="child in node.children"
              :key="child.id"
              class="flex items-center px-3 py-1.5 cursor-pointer hover:bg-gray-200/50 transition-colors group"
              :class="{
                'bg-blue-50': props.currentPath === child.path
              }"
              @click="child.type === 'file' ? selectFile(child) : undefined"
            >
              <span class="w-4 mr-1"></span>

              <component
                :is="child.type === 'file' ? FileText : Folder"
                :size="15"
                class="mr-2 flex-shrink-0"
                :class="{
                  'text-blue-400': child.type === 'directory',
                  'text-gray-500': child.type === 'file'
                }"
              />

              <span
                class="text-sm truncate flex-1"
                :class="{
                  'text-gray-600': child.type === 'directory',
                  'text-gray-500': child.type === 'file',
                  'font-medium text-blue-600': props.currentPath === child.path
                }"
              >
                {{ child.name }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Outline View -->
    <div v-else class="flex-1 overflow-y-auto scrollbar-thin">
      <DocumentOutline
        v-if="content"
        :content="content"
        @select-heading="selectHeading"
      />
      <div v-else class="text-center text-sm text-gray-400 p-4">
        Open a file to view outline
      </div>
    </div>
  </div>
</template>

<style scoped>
.tree-node {
  user-select: none;
}

.tree-node:hover > div {
  background-color: rgba(0, 0, 0, 0.05);
}

.scrollbar-thin::-webkit-scrollbar {
  width: 4px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: transparent;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  background: #d1d5db;
  border-radius: 2px;
}

.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}
</style>
