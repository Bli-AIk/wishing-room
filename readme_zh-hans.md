# Wishing Room

[![version](https://img.shields.io/badge/version-0.0.1-blue)]() <img src="https://img.shields.io/badge/status-early%20development-orange"/> <img src="https://img.shields.io/badge/focus-Android%20first-1f6feb"/> <img src="https://img.shields.io/badge/Tiled-latest%20orthogonal%20subset-2ea44f"/> <br>
<img src="https://img.shields.io/badge/Rust-2024-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Dioxus-0.7.3-6f42c1?style=for-the-badge" />

> **状态**：🚧 初始开发阶段（当前已建立核心格式边界，Android 主界面仍在持续收紧）

**Wishing Room** —— 一个面向 **Android** 的 **Tiled 地图编辑器第三方实现**，目标不是把桌面版强行压缩到手机里，而是认真做一套适合触控和竖屏使用的地图编辑工作流。

| English | 简体中文 |
|---------|----------|
| [English](./readme.md) | 简体中文 |

## 简介

**Wishing Room** 是一个面向 **Android**、面向社区使用场景的 **Tiled 地图编辑器第三方实现**。

它首先解决的是一个很具体的问题：  
让 `.tmx` 地图在手机上被打开、查看，并逐步进入可用的编辑流程，而不是把移动端当成桌面版的附属预览器。

当前阶段，项目刻意只聚焦一条收敛后的主路径：

- 对齐 **最新版稳定版 Tiled**
- 只做 **Orthogonal（正交）地图**
- 优先打磨 **Android 竖屏工作流**
- 明确支持范围，而不是模糊地宣称“部分兼容”

在核心编辑闭环站稳之后，Wishing Room 会逐步往更社区化的方向扩展，尤其是 **Undertale / Deltarune** 相关的地图制作、改图与整理流程。

## 名称来源

**Wishing Room** 这个名字来自 **Undertale** 中的同名地点。

这个命名不是随手取的。它在一开始就表明了项目的气质：Wishing Room 虽然是一个通用方向上的地图编辑器实现，但它也确实受到 **Undertale / Deltarune** 社区地图工作流的影响。

## 项目动机

Wishing Room 的出发点其实很简单：我们希望 **Undertale / Deltarune** 社区能有一个更便捷的地图编辑软件。

所以，这个项目从一开始就把 Android 和移动端工作流当成正经目标来做，而不是事后补一个“也能在手机上打开”的附属壳。

日后，Wishing Room 也会接入 [`open-utdr-maps`](https://github.com/Bli-AIk/open-utdr-maps) 的地图资源库，让移动端的地图查看与编辑能够接上更开放的地图生态。

## 功能特性

* 以 Android 竖屏为主场的移动端 screen 结构
* 基于官方 [`mapeditor/rs-tiled`](https://github.com/mapeditor/rs-tiled) 的 TMX / TSX 读取主链
* 核心编辑逻辑与应用壳分离，便于多端复用
* 内置 TMX 样例，便于 Web / Android 快速预览
* 基于真实运行截图的参考图对齐流程
* （持续完善中）瓦片、图层、对象、属性编辑流程

## 如何使用

当前阶段，Wishing Room 更偏向贡献者和测试者使用。

1. **克隆仓库**：

   ```bash
   git clone <你的 fork 或本地远端地址>
   cd wishing-room
   ```

2. **先跑基础检查**：

   ```bash
   just check
   just clippy
   just test
   just fmt-check
   ```

3. **启动 Web 预览**：

   ```bash
   dx serve -p wishing-editor --platform web
   ```

4. **如果你在远端机器上开发，需要经 SSH 转发到本机 / Android 浏览器预览**：

   ```bash
   just ssh-preview
   ```

5. **构建 Android 包**：

   ```bash
   dx build --android --target aarch64-linux-android -r -p wishing-editor
   ```

## 如何构建

### 前置要求

* Rust 工具链（2024 edition）
* [`dioxus-cli`](https://dioxuslabs.com/learn/0.7/getting_started/)
* 如果要构建 Android，需要安装 Android SDK / NDK

### 构建步骤

1. **检查整个 workspace**：

   ```bash
   just check
   ```

2. **运行 Clippy（警告即失败）**：

   ```bash
   just clippy
   ```

3. **运行测试**：

   ```bash
   just test
   ```

4. **构建 Android 应用**：

   ```bash
   dx build --android --target aarch64-linux-android -r -p wishing-editor
   ```

## 依赖

当前 workspace 主要依赖以下核心 crate：

| Crate | Version | 作用 |
|-------|---------|------|
| [`dioxus`](https://crates.io/crates/dioxus) | `0.7.3` | Web / Desktop / Android 应用壳 |
| [`tiled`](https://crates.io/crates/tiled) | `0.15.0` | 官方 TMX / TSX 读取 |
| [`quick-xml`](https://crates.io/crates/quick-xml) | `0.38.4` | XML 写出与辅助处理 |
| [`roxmltree`](https://crates.io/crates/roxmltree) | `0.20.0` | XML 检查与预处理 |
| [`thiserror`](https://crates.io/crates/thiserror) | `2.0.17` | 结构化错误定义 |

## 项目结构

```text
apps/wishing-editor/   Dioxus 应用，面向 Web / Desktop / Android
crates/wishing-core/   共享编辑器模型、TMX/TSX 读写、会话逻辑
assets/                内置样例地图与资源
TASK.csv               结构化 Tiled 功能 / 任务栈
```

## 参与贡献

欢迎参与，尤其欢迎以下方向的贡献：

* 格式兼容与保存可靠性
* Android 触控交互与信息架构
* 基于真实运行截图的 UI 对齐
* 文档与样例地图补充

## 参考资料

* [Tiled 官方文档](https://doc.mapeditor.org/en/stable/)
* [Tiled 源码仓库](https://github.com/mapeditor/tiled)
* [`mapeditor/rs-tiled`](https://github.com/mapeditor/rs-tiled)
