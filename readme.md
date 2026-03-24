# Wishing Room

[![version](https://img.shields.io/badge/version-0.0.1-blue)]() <img src="https://img.shields.io/badge/status-early%20development-orange"/> <img src="https://img.shields.io/badge/focus-Android%20first-1f6feb"/> <img src="https://img.shields.io/badge/Tiled-latest%20orthogonal%20subset-2ea44f"/> <br>
<img src="https://img.shields.io/badge/Rust-2024-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Dioxus-0.7.3-6f42c1?style=for-the-badge" />

> Current Status: 🚧 Early Development (the core format boundary is in place, while the Android-first UI is still being refined)

**Wishing Room** — an Android-first third-party implementation of the **Tiled** map editor, built for people who want a real mobile map editing workflow instead of a desktop UI squeezed into a phone screen.

| English | 简体中文 |
|---------|----------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

**Wishing Room** is an Android-first, community-facing third-party implementation of the **Tiled** map editor.

It is being built for a very specific use case: opening, reviewing, and eventually editing Tiled maps on a phone without treating mobile as a second-class target.

At the moment, the project is intentionally focused on a narrow and reliable path:

- the latest stable **Tiled** format
- **Orthogonal** maps only
- Android portrait workflows
- explicit support boundaries instead of vague “partial support”

Once the core editor loop is stable, Wishing Room is meant to grow into a friendlier tool for community map workflows, especially those around **Undertale / Deltarune** creation, remixing, and curation.

## Name Origin

The name **Wishing Room** comes from the location of the same name in **Undertale**.

That reference is intentional. This project is not only a generic map editor experiment. It is also shaped by the kind of community workflows that grow around **Undertale / Deltarune** derivative works, fangames, and map archives.

## Motivation

The starting point of Wishing Room is simple: we want the **Undertale / Deltarune** community to have a more convenient map editor.

That is why the project treats Android and mobile workflows seriously from the beginning, instead of leaving them as an afterthought.

In the future, Wishing Room is also intended to connect with the map resource library built around [`open-utdr-maps`](https://github.com/Bli-AIk/open-utdr-maps), so that mobile-side viewing and editing can tie into a broader open map ecosystem.

## Features

* Android-first screen flow with dedicated mobile layouts
* TMX / TSX loading built on the official [`mapeditor/rs-tiled`](https://github.com/mapeditor/rs-tiled) crate
* Shared editor core separated from the application shell
* Embedded TMX samples for fast review on web and Android builds
* Reference-driven UI review workflow based on runtime screenshots
* (In progress) tighter editing workflows for tiles, layers, objects, and properties

## How to Use

Right now, Wishing Room is primarily aimed at contributors and testers.

1. **Clone the repository**:

   ```bash
   git clone <your-fork-or-local-remote>
   cd wishing-room
   ```

2. **Run workspace checks**:

   ```bash
   just check
   just clippy
   just test
   just fmt-check
   ```

3. **Start a web preview**:

   ```bash
   dx serve -p wishing-editor --platform web
   ```

4. **Preview through SSH on another machine**:

   ```bash
   just ssh-preview
   ```

5. **Build an Android package**:

   ```bash
   dx build --android --target aarch64-linux-android -r -p wishing-editor
   ```

## How to Build

### Prerequisites

* Rust toolchain with the 2024 edition
* [`dioxus-cli`](https://dioxuslabs.com/learn/0.7/getting_started/)
* Android SDK / NDK if you want to build the mobile target

### Build Steps

1. **Check the whole workspace**:

   ```bash
   just check
   ```

2. **Lint with warnings denied**:

   ```bash
   just clippy
   ```

3. **Run tests**:

   ```bash
   just test
   ```

4. **Build the Android app**:

   ```bash
   dx build --android --target aarch64-linux-android -r -p wishing-editor
   ```

## Dependencies

The workspace currently leans on a small set of core crates:

| Crate | Version | Role |
|-------|---------|------|
| [`dioxus`](https://crates.io/crates/dioxus) | `0.7.3` | app shell for web, desktop, and Android |
| [`tiled`](https://crates.io/crates/tiled) | `0.15.0` | official TMX / TSX loading |
| [`quick-xml`](https://crates.io/crates/quick-xml) | `0.38.4` | XML writing and supporting utilities |
| [`roxmltree`](https://crates.io/crates/roxmltree) | `0.20.0` | XML inspection and preprocessing |
| [`thiserror`](https://crates.io/crates/thiserror) | `2.0.17` | structured editor errors |

## Project Structure

```text
apps/wishing-editor/   Dioxus application for web, desktop, and Android
crates/wishing-core/   shared editor model, loading, saving, and session logic
assets/                embedded TMX samples and image assets
TASK.csv               structured Tiled feature/task stack
```

## Contributing

Contributions are welcome, especially in places where correctness and UX meet:

* format compatibility and save reliability
* Android-first interaction design
* UI parity work backed by runtime screenshots
* documentation and sample-map coverage

## References

* [Tiled documentation](https://doc.mapeditor.org/en/stable/)
* [Tiled source code](https://github.com/mapeditor/tiled)
* [`mapeditor/rs-tiled`](https://github.com/mapeditor/rs-tiled)
