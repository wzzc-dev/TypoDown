<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import Toolbar from '../toolbar/Toolbar.vue';

const emit = defineEmits<{
  'toggle-sidebar': []
  'open-file': []
  'save-file': []
  'save-as': []
}>();

const sidebarWidth = ref(240);
const isSidebarOpen = ref(false);
const currentPath = ref('');
const currentFileName = ref('');
const isModified = ref(false);
const showToolbar = ref(false);
const toolbarTimeout = ref<number | null>(null);
const resizerRef = ref<HTMLDivElement | null>(null);
const isResizing = ref(false);
const startX = ref(0);
const startWidth = ref(0);

// 侧边栏宽度限制
const MIN_SIDEBAR_WIDTH = 150;
const MAX_SIDEBAR_WIDTH = 500;

const handlePathChange = (path: string) => {
  currentPath.value = path;
};

const handleFileNameChange = (fileName: string, modified: boolean) => {
  currentFileName.value = fileName;
  isModified.value = modified;
};

const toggleSidebar = () => {
  isSidebarOpen.value = !isSidebarOpen.value;
  emit('toggle-sidebar');
};

const handleOpenFile = () => {
  emit('open-file');
};

const handleSaveFile = () => {
  emit('save-file');
};

const handleSaveAs = () => {
  emit('save-as');
};

const handleMouseMove = (e: MouseEvent) => {
  const mouseY = e.clientY;
  const threshold = 20; // 顶部20像素区域

  if (mouseY < threshold) {
    showToolbar.value = true;
    
    // 清除之前的隐藏定时器
    if (toolbarTimeout.value) {
      clearTimeout(toolbarTimeout.value);
      toolbarTimeout.value = null;
    }
  } else {
    // 鼠标移开顶部区域后延迟隐藏
    if (toolbarTimeout.value) {
      clearTimeout(toolbarTimeout.value);
    }
    
    toolbarTimeout.value = window.setTimeout(() => {
      showToolbar.value = false;
      toolbarTimeout.value = null;
    }, 500);
  }

  // 处理拖动调整大小
  if (isResizing.value) {
    const deltaX = e.clientX - startX.value;
    const newWidth = startWidth.value + deltaX;
    
    // 限制宽度范围
    if (newWidth >= MIN_SIDEBAR_WIDTH && newWidth <= MAX_SIDEBAR_WIDTH) {
      sidebarWidth.value = newWidth;
    }
  }
};

const handleMouseDown = (e: MouseEvent) => {
  if (!resizerRef.value || !isSidebarOpen.value) return;
  
  isResizing.value = true;
  startX.value = e.clientX;
  startWidth.value = sidebarWidth.value;
  
  // 防止选中文字
  e.preventDefault();
  e.stopPropagation();
  
  // 添加全局鼠标移动和释放事件
  document.addEventListener('mousemove', handleDocumentMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

const handleDocumentMouseMove = (e: MouseEvent) => {
  if (!isResizing.value) return;
  
  const deltaX = e.clientX - startX.value;
  const newWidth = startWidth.value + deltaX;
  
  // 限制宽度范围
  if (newWidth >= MIN_SIDEBAR_WIDTH && newWidth <= MAX_SIDEBAR_WIDTH) {
    sidebarWidth.value = newWidth;
  }
};

const handleMouseUp = () => {
  isResizing.value = false;
  
  // 移除全局事件
  document.removeEventListener('mousemove', handleDocumentMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
};

onMounted(() => {
  window.addEventListener('mousemove', handleMouseMove);
});

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMouseMove);
  if (toolbarTimeout.value) {
    clearTimeout(toolbarTimeout.value);
  }
  // 清理拖动事件
  document.removeEventListener('mousemove', handleDocumentMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
});

defineExpose({
  handleFileNameChange,
  handlePathChange
});
</script>

<template>
  <div class="editor-layout h-screen w-screen flex flex-col bg-gray-50">
    <!-- Unified Toolbar -->
    <Transition name="toolbar">
      <Toolbar
        v-show="true"
        :is-sidebar-open="isSidebarOpen"
        :current-file-name="currentFileName"
        :is-modified="isModified"
        :current-path="currentPath"
        @toggle-sidebar="toggleSidebar"
        @open-file="handleOpenFile"
        @save-file="handleSaveFile"
        @save-as="handleSaveAs"
      />
    </Transition>

    <!-- Main Content Area -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar -->
      <Transition name="sidebar">
        <aside 
          v-if="isSidebarOpen"
          class="h-full bg-gray-100 border-r border-gray-200 flex-shrink-0"
          :style="{ width: `${sidebarWidth}px` }"
        >
          <slot name="sidebar" :path="currentPath" @path-change="handlePathChange"></slot>
        </aside>
      </Transition>

      <!-- Resizer -->
      <Transition name="resizer">
        <div
          v-if="isSidebarOpen"
          ref="resizerRef"
          class="resizer w-0.5 bg-gray-200 hover:bg-gray-300 cursor-col-resize flex-shrink-0"
          :class="{ 'is-resizing': isResizing }"
          @mousedown="handleMouseDown"
        ></div>
      </Transition>

      <!-- Editor Area -->
      <main class="flex-1 h-full bg-white">
        <slot name="editor"></slot>
      </main>
    </div>
  </div>
</template>

<style scoped>
.toolbar-enter-active,
.toolbar-leave-active {
  transition: all 0.2s ease-in-out;
}

.toolbar-enter-from,
.toolbar-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}

.resizer-enter-active,
.resizer-leave-active {
  transition: all 0.3s ease;
}

.resizer-enter-from,
.resizer-leave-to {
  opacity: 0;
}

.sidebar-enter-active,
.sidebar-leave-active {
  transition: all 0.3s ease;
}

.sidebar-enter-from,
.sidebar-leave-to {
  width: 0;
  opacity: 0;
}

.resizer {
  transition: background-color 0.15s ease;
}

.resizer:hover {
  background-color: #9ca3af;
}

.resizer.is-resizing {
  background-color: #3b82f6;
  cursor: col-resize;
  user-select: none;
}

.editor-layout {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
}
</style>
