<script setup lang="ts">
import { Menu, X, FolderOpen, Save, FileText, FileCode } from 'lucide-vue-next';

interface Props {
    isSidebarOpen: boolean;
    currentFileName: string;
    isModified: boolean;
    currentPath: string;
}

interface Emits {
    (e: 'toggle-sidebar'): void;
    (e: 'open-file'): void;
    (e: 'save-file'): void;
    (e: 'save-as'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const toggleSidebar = () => {
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
</script>

<template>
    <div class="macos-toolbar  h-[30px] flex items-center px-4 select-none flex-row-reverse"
        data-tauri-drag-region>
        <!-- App Title (Leftmost) -->
        <!-- <span class="text-sm font-medium text-gray-900 hide-on-tiny">TypoDown</span> -->
        <!-- Sidebar Toggle (Rightmost) -->
        
        
        <!-- Current Path -->
        <span v-if="currentPath" class="text-xs text-gray-500 hide-on-small font-medium truncate max-w-[200px]">{{
            currentPath }}</span>

        <!-- File Operations -->
        <div class="flex items-center gap-1.5">
            <!-- Current File Info -->
            <div v-if="currentFileName" class="file-info-badge">
                <FileText :size="12" class="text-gray-500" />
                <span class="font-medium">{{ currentFileName }}</span>
                <div v-if="isModified" class="modified-dot"></div>
            </div>

            <!-- Save As Button -->
            <button v-if="!currentFileName" @click="handleSaveAs" class="macos-btn w-8 h-8"
                title="Save As (Cmd+Shift+S)">
                <FileCode :size="14" class="macos-icon" />
            </button>
            

            <div class="macos-separator mx-1"></div>
            <button @click="handleOpenFile" class="macos-btn w-8 h-8" title="Open File (Cmd+O)">
                <FolderOpen :size="16" class="macos-icon" />
            </button>

            <button @click="handleSaveFile" class="macos-btn w-8 h-8" title="Save File (Cmd+S)">
                <Save :size="16" class="macos-icon" />
            </button>

            <button @click="toggleSidebar" class="macos-btn w-8 h-8" title="Toggle Sidebar">
                <Menu v-if="!isSidebarOpen" :size="16" class="macos-icon" />
                <X v-else :size="16" class="macos-icon" />
            </button>
            
        </div>

        
    </div>
</template>

<style scoped>
@import '../../styles/macos-theme.css';

.macos-toolbar {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
    font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Helvetica Neue', sans-serif;
    padding: 0 16px;
}

.macos-btn {
    border-radius: 8px;
    transition: all 0.15s ease;
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
}

.macos-btn:hover {
    background: rgba(0, 0, 0, 0.06);
}

.macos-btn:active {
    transform: scale(0.97);
    background: rgba(0, 0, 0, 0.1);
}

.macos-icon {
    color: rgba(0, 0, 0, 0.75);
    stroke-width: 1.5;
}

.macos-separator {
    width: 1px;
    height: 16px;
    background: rgba(0, 0, 0, 0.1);
}

.file-info-badge {
    background: rgba(0, 0, 0, 0.04);
    /* border-radius: 6px; */
    padding: 4px 8px;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: rgba(0, 0, 0, 0.85);
    font-weight: 500;
}

.file-info-badge .modified-dot {
    width: 6px;
    height: 6px;
    background: #007AFF;
    /* border-radius: 50%; */
    animation: pulse 2s infinite;
}

@keyframes pulse {

    0%,
    100% {
        opacity: 1;
    }

    50% {
        opacity: 0.5;
    }
}

@media (max-width: 800px) {
    .hide-on-small {
        display: none;
    }
}

@media (max-width: 600px) {
    .hide-on-tiny {
        display: none;
    }
}

button {
    cursor: pointer;
}
</style>
