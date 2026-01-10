<script setup lang="ts">
import { ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();
const isMaximized = ref(false);

const minimize = async () => {
  await appWindow.minimize();
};

const toggleMaximize = async () => {
  await appWindow.toggleMaximize();
};

const closeWindow = async () => {
  await appWindow.close();
};
</script>

<template>
  <div class="window-controls">
    <!-- Close Button -->
    <button
      @click="closeWindow"
      class="window-control-btn close"
      title="Close"
    >
      <span class="icon">×</span>
    </button>

    <!-- Minimize Button -->
    <button
      @click="minimize"
      class="window-control-btn minimize"
      title="Minimize"
    >
      <span class="icon">−</span>
    </button>

    <!-- Maximize/Restore Button -->
    <button
      @click="toggleMaximize"
      class="window-control-btn maximize"
      :title="isMaximized ? 'Restore' : 'Maximize'"
    >
      <span class="icon">{{ isMaximized ? '−' : '+' }}</span>
    </button>
  </div>
</template>

<style scoped>
.window-controls {
  -webkit-app-region: no-drag;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-left: 12px;
}

.window-control-btn {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  flex-shrink: 0;
}

.window-control-btn .icon {
  opacity: 0;
  transition: opacity 0.15s ease;
  font-size: 10px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
}

.window-control-btn:hover .icon {
  opacity: 1;
}

.window-control-btn.close {
  background: #FF5F57;
}

.window-control-btn.close:hover {
  background: #FF453A;
}

.window-control-btn.minimize {
  background: #FEBC2E;
}

.window-control-btn.minimize:hover {
  background: #FEA71E;
}

.window-control-btn.maximize {
  background: #28C840;
}

.window-control-btn.maximize:hover {
  background: #20A832;
}
</style>
