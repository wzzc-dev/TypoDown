<script setup lang="ts">
import { ref, watch } from 'vue';
import EditorLayout from './components/layout/EditorLayout.vue';
import FileTree from './components/sidebar/FileTree.vue';
import MilkdownEditor from './components/editor/MilkdownEditor.vue';
import { useFileStore } from './stores/fileStore';

const {
  currentFile,
  fileContent,
  isModified,
  readFile,
  writeFile,
  openFileDialog,
  saveFileDialog,
  updateContent
} = useFileStore();

const editorLayoutRef = ref<InstanceType<typeof EditorLayout>>();
const sidebarOpen = ref(false);
const showWelcome = ref(true);
const showEditor = ref(false);

const handleFileSelect = async (path: string, name: string) => {
  try {
    showEditor.value = false;
    await new Promise(resolve => setTimeout(resolve, 50));
    await readFile(path, name);
    showWelcome.value = false;
    await new Promise(resolve => setTimeout(resolve, 50));
    showEditor.value = true;
  } catch (error) {
    console.error('Failed to load file:', error);
  }
};

const handleOpenFile = async () => {
  try {
    showEditor.value = false;
    await openFileDialog();
    showWelcome.value = false;
    await new Promise(resolve => setTimeout(resolve, 50));
    showEditor.value = true;
  } catch (error) {
    console.error('Failed to open file:', error);
  }
};

const handleSaveFile = async () => {
  if (currentFile.value) {
    try {
      await writeFile(currentFile.value.path, fileContent.value);
    } catch (error) {
      console.error('Failed to save file:', error);
    }
  } else {
    await handleSaveAs();
  }
};

const handleSaveAs = async () => {
  try {
    const defaultName = 'untitled.md';
    await saveFileDialog(defaultName);
  } catch (error) {
    console.error('Failed to save file as:', error);
  }
};

const scrollToHeading = (index: number) => {
  // 滚动编辑器到指定标题位置
  console.log('Scroll to heading:', index);
  // 这里可以添加实际的滚动逻辑
  // 现在只是简单处理，用户点击标题时可以滚动到相应位置
};

// Watch for file changes to update layout
watch([currentFile, isModified], () => {
  if (editorLayoutRef.value) {
    const fileName = currentFile.value?.name || '';
    const modified = isModified.value || false;
    editorLayoutRef.value.handleFileNameChange(fileName, modified);
  }
});
</script>

<template>
  <EditorLayout 
    ref="editorLayoutRef"
    @toggle-sidebar="sidebarOpen = !sidebarOpen"
    @open-file="handleOpenFile"
    @save-file="handleSaveFile"
    @save-as="handleSaveAs"
  >
    <template #sidebar>
      <FileTree 
        :current-path="currentFile?.path"
        :content="fileContent"
        @select-file="handleFileSelect"
        @select-heading="(index: number) => scrollToHeading(index)"
        @path-change="(path: string) => editorLayoutRef?.handlePathChange(path)"
      />
    </template>

    <template #editor>
      <!-- Editor Area -->
      <div class="h-full flex flex-col">
        <!-- Welcome Screen -->
        <div
          v-if="showWelcome"
          class="flex-1 flex flex-col items-center justify-center text-gray-400"
        >
          <div class="text-6xl mb-8 opacity-30">📝</div>
          <h1 class="text-2xl font-semibold text-gray-600 mb-2 tracking-tight">Welcome to TypoDown</h1>
          <p class="text-gray-400 mb-6">A minimalist Markdown editor</p>
          <button
            @click="handleOpenFile"
            class="flex items-center gap-2 px-5 py-2.5 bg-gray-700 text-white rounded hover:bg-gray-800 transition-colors shadow-sm"
          >
            <span>Open File</span>
          </button>
          <p class="mt-4 text-xs text-gray-400">
            Shortcut: <kbd class="px-1.5 py-0.5 bg-gray-100 rounded text-gray-500 border border-gray-200">⌘O</kbd> to open file
          </p>
        </div>

        <!-- Milkdown Editor -->
        <MilkdownEditor
          ref="editorRef"
          v-if="!showWelcome && showEditor"
          v-model="fileContent"
          @update:model-value="updateContent"
        />
      </div>
    </template>
  </EditorLayout>
</template>

<style scoped>
/* Remove scrollbar styling for cleaner look */
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
