import type { Editor } from '@milkdown/core';

declare module '@milkdown/vue' {
  export interface MilkdownEditorInstance {
    editor: Editor;
    undo: () => void;
    redo: () => void;
    toggleFullscreen: () => void;
  }
  
  export interface MilkdownEditor extends MilkdownEditorInstance {
    $el: HTMLElement;
  }
}