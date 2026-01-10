<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Editor, rootCtx, defaultValueCtx, editorViewCtx } from '@milkdown/core';
import { commonmark } from '@milkdown/preset-commonmark';
import { gfm } from '@milkdown/preset-gfm';
import { history } from '@milkdown/plugin-history';
import { listener, listenerCtx } from '@milkdown/plugin-listener';

interface Props {
  modelValue: string;
}

interface Emits {
  (e: 'update:modelValue', value: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const containerRef = ref<HTMLDivElement>();
let editorInstance: Editor | null = null;

onMounted(async () => {
  if (!containerRef.value) return;

  try {
    editorInstance = await Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, containerRef.value!);
        ctx.set(defaultValueCtx, props.modelValue);
        ctx.get(listenerCtx).markdownUpdated((ctx, markdown) => {
          emit('update:modelValue', markdown);
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(history)
      .use(listener)
      .create();
  } catch (error) {
    console.error('Failed to create editor:', error);
  }
});

onUnmounted(() => {
  if (editorInstance) {
    editorInstance.destroy();
    editorInstance = null;
  }
});
</script>

<template>
  <div ref="containerRef" class="milkdown-editor h-full"></div>
</template>

<style scoped>
.milkdown-editor {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 30px 40px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
  font-size: 15px;
  line-height: 1.75;
  color: #2c3e50;
  background: #ffffff;
}

.milkdown-editor :deep(.milkdown) {
  max-width: 800px;
  margin: 0 auto;
  outline: none;
}

.milkdown-editor :deep(.ProseMirror) {
  min-height: 100%;
  outline: none;
  padding: 15px 0;
}

.milkdown-editor :deep(.ProseMirror p) {
  margin: 0.8em 0;
}

.milkdown-editor :deep(.ProseMirror h1) {
  font-size: 2.25rem;
  font-weight: 700;
  margin-top: 1.8rem;
  margin-bottom: 0.8rem;
  line-height: 1.2;
  color: #1a1a1a;
  letter-spacing: -0.02em;
}

.milkdown-editor :deep(.ProseMirror h2) {
  font-size: 1.75rem;
  font-weight: 600;
  margin-top: 1.5rem;
  margin-bottom: 0.8rem;
  line-height: 1.3;
  color: #1a1a1a;
  letter-spacing: -0.01em;
}

.milkdown-editor :deep(.ProseMirror h3) {
  font-size: 1.4rem;
  font-weight: 600;
  margin-top: 1.25rem;
  margin-bottom: 0.7rem;
  line-height: 1.4;
  color: #2c3e50;
}

.milkdown-editor :deep(.ProseMirror code) {
  background-color: rgba(0, 0, 0, 0.05);
  padding: 0.15rem 0.35rem;
  border-radius: 0.25rem;
  font-family: 'SF Mono', 'Fira Code', 'Monaco', 'Consolas', monospace;
  font-size: 0.9em;
  color: #c7254e;
}

.milkdown-editor :deep(.ProseMirror pre) {
  background-color: #f8f8f8;
  color: #2c3e50;
  padding: 1.2rem;
  border-radius: 0.4rem;
  overflow-x: auto;
  margin: 1.2rem 0;
  border: 1px solid #e1e4e8;
  font-size: 0.95em;
}

.milkdown-editor :deep(.ProseMirror pre code) {
  background-color: transparent;
  color: inherit;
  padding: 0;
}

.milkdown-editor :deep(.ProseMirror blockquote) {
  border-left: 3px solid #d1d5db;
  padding-left: 1.2rem;
  margin: 1.2rem 0;
  color: #6b7280;
  font-style: italic;
  background-color: #f9fafb;
  border-radius: 0.25rem;
  padding: 0.8rem 1.2rem 0.8rem 1.5rem;
}

.milkdown-editor :deep(.ProseMirror ul),
.milkdown-editor :deep(.ProseMirror ol) {
  padding-left: 2rem;
  margin: 0.9em 0;
}

.milkdown-editor :deep(.ProseMirror li) {
  margin: 0.3em 0;
  line-height: 1.75;
}

.milkdown-editor :deep(.ProseMirror li::marker) {
  color: #6b7280;
}

.milkdown-editor :deep(.ProseMirror a) {
  color: #3b82f6;
  text-decoration: none;
  transition: color 0.15s ease;
}

.milkdown-editor :deep(.ProseMirror a:hover) {
  color: #2563eb;
  text-decoration: underline;
}

.milkdown-editor :deep(.ProseMirror table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1.2rem 0;
  border: 1px solid #e1e4e8;
}

.milkdown-editor :deep(.ProseMirror th),
.milkdown-editor :deep(.ProseMirror td) {
  border: 1px solid #e1e4e8;
  padding: 0.6rem 0.8rem;
  text-align: left;
}

.milkdown-editor :deep(.ProseMirror th) {
  background-color: #f8f9fa;
  font-weight: 600;
  color: #1a1a1a;
}

.milkdown-editor :deep(.ProseMirror tr:hover td) {
  background-color: #f8f9fa;
}

.milkdown-editor :deep(.ProseMirror img) {
  max-width: 100%;
  height: auto;
  border-radius: 0.25rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.milkdown-editor :deep(.ProseMirror hr) {
  border: none;
  border-top: 1px solid #e1e4e8;
  margin: 1.8rem 0;
}

.milkdown-editor :deep(.ProseMirror strong) {
  font-weight: 600;
  color: #1a1a1a;
}

.milkdown-editor :deep(.ProseMirror em) {
  font-style: italic;
  color: #4b5563;
}

.milkdown-editor :deep(.ProseMirror-selectednode) {
  outline: 1px solid #3b82f6;
  outline-offset: 2px;
}

.milkdown-editor :deep(.ProseMirror-focus:focus) {
  outline: none;
}
</style>
