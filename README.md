# TypoDown

一个基于 Tauri + Vue 3 + Milkdown 构建的跨平台 Markdown 编辑器，采用所见即所得模式，界面风格模仿 Typora 的简约设计。

## 功能特性

- ✨ **所见即所得编辑** - 编辑和预览在同一区域实时同步
- 📁 **文件树导航** - 左侧文件树导航，支持文件选择
- 💾 **文件操作** - 支持打开本地 Markdown 文件、保存当前编辑内容
- 🎨 **简约界面** - 类似 Typora 的沉浸式编辑体验
- ⌨️ **快捷键支持** - 支持 Cmd+O 打开文件，Cmd+S 保存文件
- 🌓 **响应式布局** - 左侧导航栏可折叠，右侧编辑区自适应

## 技术栈

- **桌面框架**: Tauri 2 (Rust 后端 + Web 前端)
- **前端框架**: Vue 3 + TypeScript
- **编辑器核心**: Milkdown (基于 ProseMirror 的 WYSIWYG 编辑器)
- **构建工具**: Vite
- **样式框架**: Tailwind CSS

## 快速开始

### 前置要求

- Node.js 18+ 或 Bun
- Rust 1.70+
- 系统依赖：
  - macOS: Xcode Command Line Tools
  - Linux: `libwebkit2gtk-4.0-dev` `build-essential` `curl` `wget` `file` `libssl-dev`
  - Windows: Microsoft C++ Build Tools + WebView2

### 安装依赖

```bash
# 使用 bun (推荐)
bun install

# 或使用 npm
npm install
```

### 开发模式

```bash
# 启动开发服务器
bun run tauri dev
```

### 构建应用

```bash
# 构建生产版本
bun run tauri build
```

构建产物将生成在 `src-tauri/target/release/bundle/` 目录。

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Cmd/Ctrl + O` | 打开文件 |
| `Cmd/Ctrl + S` | 保存文件 |
| `Cmd/Ctrl + Shift + S` | 另存为 |

## 项目结构

```
typodown/
├── src/                          # Vue 前端源码
│   ├── components/
│   │   ├── layout/
│   │   │   └── EditorLayout.vue  # 主布局组件
│   │   ├── sidebar/
│   │   │   └── FileTree.vue      # 文件树组件
│   │   └── editor/
│   │       └── MilkdownEditor.vue # Milkdown 编辑器组件
│   ├── stores/
│   │   └── fileStore.ts          # 文件状态管理
│   ├── types/
│   │   └── file.ts               # 文件类型定义
│   ├── App.vue                   # 主应用组件
│   └── main.ts                   # 应用入口
│
├── src-tauri/                    # Rust 后端源码
│   ├── src/
│   │   ├── main.rs               # Rust 主入口
│   │   └── lib.rs                # Tauri 命令和运行逻辑
│   ├── Cargo.toml                # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 应用配置
│
├── package.json                  # Node.js 依赖
├── vite.config.ts                # Vite 配置
├── tailwind.config.js            # Tailwind CSS 配置
└── tsconfig.json                 # TypeScript 配置
```

## 开发说明

### 添加新功能

1. **前端组件**: 在 `src/components/` 下创建新组件
2. **状态管理**: 在 `src/stores/` 下创建新的 store
3. **Tauri 命令**: 在 `src-tauri/src/lib.rs` 中添加新的 `#[tauri::command]` 函数
4. **权限配置**: 在 `src-tauri/capabilities/default.json` 中添加必要的权限

### 样式定制

- 全局样式: `src/index.css`
- 组件样式: 使用 Tailwind CSS utility classes
- 编辑器样式: `src/components/editor/MilkdownEditor.vue`

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Milkdown](https://milkdown.dev/) - 插件驱动的 WYSIWYG Markdown 编辑器框架
- [Typora](https://typora.io/) - 灵感来源
