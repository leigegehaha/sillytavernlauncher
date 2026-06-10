# 🍺 Tavern Deepseek Launcher

> SillyTavern 酒馆桌面启动器 — 零配置、一键启动、中世纪暗黑奇幻风  
> Tauri v2 + Vue 3 · 跨平台 · 内置酒馆 & Node.js

<p align="center">
  <img src="assets/logo.png" alt="Tavern Deepseek" width="220" />
</p>

<p align="center">
  <a href="https://github.com/leigegehaha/sillytavernlauncher/releases"><img src="https://img.shields.io/badge/下载-v2.0.0-blue?style=flat-square" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/framework-Tauri%20v2%20%2B%20Vue%203-brightgreen?style=flat-square" alt="Framework">
  <img src="https://img.shields.io/badge/status-active-success?style=flat-square" alt="Status">
</p>

---

## 📸 界面预览

<p align="center">
  <img src="assets/media/启动页.png" alt="启动页" width="80%" />
</p>

<details>
<summary>更多截图 👈 点击展开</summary>

### 角色卡管理
<img src="assets/media/角色卡资源.png" alt="角色卡管理" width="80%" />

### 配置教程
<img src="assets/media/教程.png" alt="教程页面" width="80%" />

</details>

---

## 🎬 快速演示

<video src="assets/media/demo视频.mp4" controls width="100%"></video>

---

## 🍺 酒馆大模型 — DeepSeek Tavern

Tavern Deepseek Launcher 内置了**酒馆专属大模型服务**，为 SillyTavern 角色扮演场景深度定制：

| 特性 | 说明 |
|------|------|
| 🧠 **v2-pro** | 旗舰模型，极致角色扮演 + 复杂剧情推理 |
| ⚡ **v2-turbo** | 快速响应，适合日常对话 |
| 🎭 **角色扮演优化** | 专为 SillyTavern 调优的 prompt 和输出风格 |
| 🔑 **一键集成** | 在酒馆大模型页面获取 API Key，自动配置到 API 连接 |
| 💰 **按量计费** | 按 token 计费，低门槛使用 |

### 获取 Key

1. 点击启动器左侧 **🍺 酒馆大模型**
2. 点击 **"酒馆专属大模型网站"** 跳转到 [deepseektavern.com](https://deepseektavern.com)
3. 注册/登录 → 创建 API Key
4. 复制 Key 粘贴到启动器的 **API 连接** 页面
5. 🍺 大功告成！

---

## ✨ 功能一览

| 模块 | 功能 |
|------|------|
| 🍺 **酒馆大模型** | DeepSeek Tavern 模型服务，API Key 管理 |
| 🔑 **API 连接** | 多 Provider 密钥管理，OpenAI 兼容端点，连接测试 |
| ▶️ **一键启动** | SillyTavern 零配置安装和启动 |
| 🃏 **角色卡管理** | PNG 角色卡读取/导入/删除，3000+ 预设角色库 |
| 🔌 **拓展管理** | 拓展安装、Git 更新、启停控制 |
| ⚙️ **酒馆选项** | config.yaml 可视化编辑，配置迁移 |
| 📦 **版本管理** | SillyTavern 多版本安装/切换 |
| 🖥️ **控制台** | 内置酒馆桌面窗口模式 |
| 🛠️ **教程** | 配置修复、依赖检测、网络诊断 |

---

## 🏗️ 技术栈

- **桌面框架**: [Tauri v2](https://v2.tauri.app/) — Rust 驱动，原生性能，小巧体积
- **前端**: Vue 3 + TypeScript + Vite
- **样式**: Tailwind CSS + 中世纪酒馆手工暗黑主题
- **图标**: Phosphor Icons
- **Rust 后端**: tokio, reqwest, serde, tracing

---

## 📦 下载安装

### macOS (Apple Silicon)

[![Download macOS](https://img.shields.io/badge/macOS-DMG-333?style=for-the-badge&logo=apple)](https://github.com/leigegehaha/sillytavernlauncher/releases/latest)

下载 `.dmg` 文件，双击挂载，拖入 `Applications` 即可。

### Windows

[![Download Windows](https://img.shields.io/badge/Windows-EXE-0078D6?style=for-the-badge&logo=windows)](https://github.com/leigegehaha/sillytavernlauncher/releases/latest)

下载 `.msi` 安装包，双击安装。

---

## 🛠️ 从源码构建

### 前置要求

- [Rust](https://www.rust-lang.org/) (最新 stable)
- [Node.js](https://nodejs.org/) 18+

### 开发模式

```bash
git clone https://github.com/leigegehaha/sillytavernlauncher.git
cd sillytavernlauncher
npm install
npm run tauri dev
```

### 生产构建

```bash
npm install
npm run tauri build
```

构建产物在 `src-tauri/target/release/bundle/`。

---

## 📂 项目结构

```
sillytavern-launcher/
├── src/                        # Vue 3 前端
│   ├── views/                  # 页面组件
│   │   ├── DeepSeek.vue        # 酒馆大模型
│   │   ├── ApiConfig.vue       # API 连接
│   │   ├── Home.vue            # 一键启动
│   │   ├── Resources.vue       # 角色卡管理
│   │   ├── Extensions.vue      # 拓展管理
│   │   ├── Tavern.vue          # 酒馆选项
│   │   ├── Versions.vue        # 版本管理
│   │   ├── Tools.vue           # 教程
│   │   ├── Console.vue         # 控制台
│   │   └── Settings.vue        # 设置
│   ├── components/             # 通用组件
│   │   ├── TavernAccount.vue   # 酒馆大模型账户
│   │   ├── BackgroundVideo.vue # 动态背景
│   │   └── ...
│   ├── layouts/Oheader.vue     # 中世纪酒馆侧边栏
│   └── router/
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # 应用入口 & 命令注册
│   │   ├── sillytavern.rs      # SillyTavern 管理
│   │   ├── secrets.rs          # API 密钥管理
│   │   ├── tavern_api.rs       # 酒馆 API 客户端
│   │   ├── config.rs           # 配置管理
│   │   ├── character.rs        # 角色卡处理
│   │   └── ...
│   └── tauri.conf.json
├── assets/
│   ├── logo.png
│   └── media/                  # 截图和演示视频
└── README.md
```

---

## 🎨 设计风格

中世纪酒馆羊皮卷轴暗黑奇幻风：

```
背景 #120c08    金辉高亮 #e6b422
边框 #b48c64    文字 #d4a574
玻璃拟态侧边栏   动态火焰背景
```

---

## 🧪 开发中功能

- [ ] **DeepTavern** 内置聊天客户端（独立窗口，直接与角色卡对话）
- [ ] TTS 语音合成集成
- [ ] 更多模型 Provider 支持

---

## 📄 致谢 & 许可

本项目基于 [al01cn/sillyTavern-launcher](https://github.com/al01cn/sillyTavern-launcher) 开发。

由 **磊哥哥** 维护 · [「磊哥哥科技拆解室」](https://space.bilibili.com/) · MIT License

---

*"来酒馆坐坐，喝一杯，聊聊天。" 🍻*
