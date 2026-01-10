<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { FileText, List, ChevronRight } from 'lucide-vue-next';

interface OutlineNode {
  id: string;
  level: number;
  title: string;
  children?: OutlineNode[];
}

interface Props {
  content: string;
  currentHeadingIndex?: number;
}

interface Emits {
  (e: 'select-heading', index: number): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const expandedSections = ref<Set<number>>(new Set());
const outline = ref<OutlineNode[]>([]);

// 解析 Markdown 内容生成大纲
const parseMarkdownOutline = (content: string): OutlineNode[] => {
  const lines = content.split('\n');
  const nodes: OutlineNode[] = [];
  
  lines.forEach((line, index) => {
    const trimmed = line.trim();
    
    // 匹配标题（# 开头）
    const headingMatch = trimmed.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      const level = headingMatch[1].length; // #, ##, ### 等的长度
      const title = headingMatch[2].trim();
      
      const node: OutlineNode = {
        id: `heading-${index}`,
        level,
        title,
      };
      
      nodes.push(node);
    }
  });
  
  return nodes;
};

const loadOutline = () => {
  outline.value = parseMarkdownOutline(props.content);
};

const toggleSection = (id: number) => {
  if (expandedSections.value.has(id)) {
    expandedSections.value.delete(id);
  } else {
    expandedSections.value.add(id);
  }
};

const selectHeading = (index: number) => {
  emit('select-heading', index);
};

const getIndent = (level: number) => {
  return (level - 1) * 16;
};

const getHeadingStyle = (level: number) => {
  const styles: Record<number, string> = {
    1: 'text-lg font-bold text-gray-900',
    2: 'text-base font-semibold text-gray-800',
    3: 'text-sm font-medium text-gray-700',
    4: 'text-xs font-medium text-gray-600',
    5: 'text-xs font-normal text-gray-600',
    6: 'text-xs font-normal text-gray-600',
  };
  return styles[level] || styles[6];
};

onMounted(() => {
  loadOutline();
});
</script>

<template>
  <div class="h-full flex flex-col bg-white">
    <!-- Header -->
    <div class="h-10 flex items-center px-3 border-b border-gray-100">
      <List :size="15" class="text-gray-400 mr-2" />
      <span class="text-xs font-medium text-gray-600 uppercase tracking-wider">Outline</span>
    </div>

    <!-- Outline Content -->
    <div class="flex-1 overflow-y-auto scrollbar-thin p-2">
      <div v-if="outline.length === 0" class="p-4 text-center text-sm text-gray-400">
        No headings found
      </div>

      <div v-else class="space-y-1">
        <div
          v-for="(node, index) in outline"
          :key="node.id"
          class="outline-item"
        >
          <div
            class="flex items-center py-1.5 cursor-pointer hover:bg-gray-100/50 transition-colors group"
            :class="{
              'bg-blue-50': currentHeadingIndex === index
            }"
            @click="selectHeading(index)"
          >
            <!-- Toggle Icon for Parent Headings -->
            <span
              v-if="index < outline.length - 1"
              class="w-4 mr-1 flex justify-center"
              @click.stop="toggleSection(index)"
            >
              <ChevronRight
                :size="10"
                class="text-gray-400 transition-transform"
                :class="{ 'rotate-90': expandedSections.has(index) }"
              />
            </span>

            <!-- Spacer for Hierarchy -->
            <span
              v-else
              class="w-4 mr-1"
            ></span>

            <!-- Heading Text -->
            <span
              class="truncate flex-1"
              :class="getHeadingStyle(node.level)"
            >
              {{ node.title }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.outline-item {
  user-select: none;
}

.outline-item:hover > div {
  background-color: rgba(0, 0, 0, 0.05);
}

.rotate-90 {
  transform: rotate(90deg);
}
</style>
