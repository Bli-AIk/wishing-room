pub(crate) const MOBILE_REVIEW_STYLES: &str = r#"
  .review-shell {
    display: none;
    height: 100dvh;
    min-height: 100dvh;
    overflow: hidden;
    background: #121212;
    color: #ffffff;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Inter", sans-serif;
  }
  .review-page {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    min-height: 100dvh;
    overflow: hidden;
    background: #121212;
  }
  .review-transition-horizontal-forward,
  .review-transition-horizontal-backward,
  .review-transition-vertical-forward,
  .review-transition-vertical-backward {
    will-change: transform, opacity;
    animation-duration: 240ms;
    animation-timing-function: cubic-bezier(0.22, 1, 0.36, 1);
    animation-fill-mode: both;
  }
  .review-transition-horizontal-forward {
    animation-name: review-slide-in-from-right;
  }
  .review-transition-horizontal-backward {
    animation-name: review-slide-in-from-left;
  }
  .review-transition-vertical-forward {
    animation-name: review-slide-in-from-bottom;
  }
  .review-transition-vertical-backward {
    animation-name: review-slide-in-from-top;
  }
  @keyframes review-slide-in-from-right {
    from { opacity: 0.7; transform: translate3d(36px, 0, 0); }
    to { opacity: 1; transform: translate3d(0, 0, 0); }
  }
  @keyframes review-slide-in-from-left {
    from { opacity: 0.7; transform: translate3d(-36px, 0, 0); }
    to { opacity: 1; transform: translate3d(0, 0, 0); }
  }
  @keyframes review-slide-in-from-bottom {
    from { opacity: 0.7; transform: translate3d(0, 42px, 0); }
    to { opacity: 1; transform: translate3d(0, 0, 0); }
  }
  @keyframes review-slide-in-from-top {
    from { opacity: 0.7; transform: translate3d(0, -42px, 0); }
    to { opacity: 1; transform: translate3d(0, 0, 0); }
  }
  .review-header {
    display: grid;
    grid-template-columns: 92px minmax(0, 1fr) 92px;
    align-items: center;
    gap: 6px;
    padding: 20px 16px 16px;
    background: #1f1f21;
    border-bottom: 1px solid #2c2c2e;
  }
  .review-header h1 {
    margin: 0;
    font-size: 17px;
    font-weight: 700;
    letter-spacing: -0.02em;
    text-align: center;
    white-space: nowrap;
  }
  .review-header-action,
  .review-link-button,
  .review-link {
    color: #b6b6bb;
    background: transparent;
    border: none;
    font: inherit;
    padding: 0;
    font-size: 14px;
    font-weight: 500;
    line-height: 1.1;
    white-space: nowrap;
  }
  .review-header-action.left {
    text-align: left;
  }
  .review-header-action.right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
    text-align: right;
  }
  .review-header-link-label {
    display: inline-block;
  }
  .review-header-plus {
    width: 16px;
    height: 16px;
    flex: none;
  }
  .review-header-spacer {
    min-height: 24px;
  }
  .review-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
    padding: 14px 14px 0;
  }
  .review-section-stack {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .review-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .review-create-project,
  .review-secondary-button,
  .review-sync-button {
    width: 100%;
    min-height: 68px;
    border-radius: 16px;
    border: 1px solid #2a2a2c;
    background: #1f1f21;
    color: #f2f2f7;
    font-size: 17px;
    font-weight: 600;
  }
  .review-create-project {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    margin-bottom: 18px;
  }
  .review-plus {
    font-size: 30px;
    line-height: 1;
    font-weight: 300;
  }
  .review-plus-icon {
    width: 24px;
    height: 24px;
    flex: none;
  }
  .review-project-card,
  .review-info-card,
  .review-layer-row,
  .review-settings-card.single {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px;
    border-radius: 20px;
    border: 1px solid #2c2c2e;
    background: #1c1c1e;
    color: inherit;
  }
  .review-project-card {
    gap: 16px;
    padding: 12px;
    border-radius: 16px;
    text-align: left;
  }
  .review-project-thumb,
  .review-layer-thumb {
    border-radius: 16px;
    flex: none;
    background-position: center;
    background-repeat: no-repeat;
    background-size: cover;
  }
  .review-project-thumb {
    width: 60px;
    height: 60px;
    border-radius: 12px;
    display: block;
    object-fit: cover;
    flex: 0 0 60px;
  }
  .review-layer-thumb {
    width: 34px;
    height: 34px;
  }
  .review-layer-thumb.ui { background-image: url('/assets/review/layer-ui.png'); }
  .review-layer-thumb.decor { background-image: url('/assets/review/layer-decor.png'); }
  .review-layer-thumb.foreground { background-image: url('/assets/review/layer-foreground.png'); }
  .review-layer-thumb.obstacles { background-image: url('/assets/review/layer-obstacles.png'); }
  .review-layer-thumb.ground { background-image: url('/assets/review/layer-ground.png'); }
  .review-layer-thumb.background { background-image: url('/assets/review/layer-background.png'); }
  .review-project-list-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid #2a2a2c;
    border-radius: 20px;
    background: #1f1f21;
  }
  .review-project-row {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    min-height: 90px;
    padding: 13px 14px;
    border: none;
    border-top: 1px solid #2a2a2c;
    background: transparent;
    color: inherit;
    text-align: left;
  }
  .review-project-row:first-child {
    border-top: none;
  }
  .review-project-copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    text-align: left;
  }
  .review-project-title,
  .review-info-title,
  .review-layer-name {
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.015em;
    line-height: 1.15;
  }
  .review-project-meta,
  .review-info-meta,
  .review-sync-meta,
  .review-script-row,
  .muted {
    color: #8f8f95;
    font-size: 13px;
    line-height: 1.3;
  }
  .review-bottom-nav {
    flex: none;
    display: grid;
    gap: 8px;
    padding: 10px 12px calc(22px + env(safe-area-inset-bottom, 0px));
    border-top: 1px solid #2c2c2e;
    background: #1f1f21;
  }
  .review-bottom-nav.dashboard {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
  .review-bottom-nav.editor {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
  .review-nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    color: #8f8f95;
    background: transparent;
    border: none;
    font: inherit;
  }
  .review-nav-item.active {
    color: #0a84ff;
  }
  .review-nav-icon {
    position: relative;
    width: 24px;
    height: 24px;
    opacity: 0.92;
  }
  .review-nav-icon-svg {
    width: 24px;
    height: 24px;
    display: block;
  }
  .review-nav-item span {
    font-size: 12px;
    line-height: 1.1;
  }
  .review-editor-page {
    height: 100dvh;
    position: relative;
    --review-editor-nav-height: calc(78px + env(safe-area-inset-bottom, 0px));
    --review-editor-toolbar-height: 68px;
    --review-editor-float-gap: 3px;
  }
  .review-editor-canvas {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    --grid-line-width: 0.5px;
    --grid-size-x: 16px;
    --grid-size-y: 16px;
    --grid-offset-x: 0px;
    --grid-offset-y: 0px;
    background:
      linear-gradient(rgba(255,255,255,0.085) var(--grid-line-width), transparent var(--grid-line-width)),
      linear-gradient(90deg, rgba(255,255,255,0.085) var(--grid-line-width), transparent var(--grid-line-width)),
      #2a2a2a;
    background-size: var(--grid-size-x) var(--grid-size-y);
    background-position: var(--grid-offset-x) var(--grid-offset-y);
  }
  .review-editor-page > .review-editor-toolbar,
  .review-editor-page > .review-bottom-nav.editor {
    position: absolute;
    left: 0;
    right: 0;
    z-index: 20;
  }
  .review-editor-page > .review-editor-toolbar {
    bottom: var(--review-editor-nav-height);
  }
  .review-editor-page > .review-bottom-nav.editor {
    bottom: 0;
    z-index: 21;
  }
  .review-map-surface {
    position: absolute;
    inset: 10px 0 0 0;
    overflow: hidden;
    background: transparent;
    box-shadow: none;
  }
  .review-map-grass,
  .review-map-path,
  .review-map-wall,
  .review-map-shadow {
    position: absolute;
  }
  .review-map-grass.a { inset: 0 0 0 0; background: linear-gradient(180deg, #83c461, #67a54a); }
  .review-map-grass.b { inset: 48% 16% 0 0; background: linear-gradient(180deg, #75b252, #5e9442); }
  .review-map-path { left: 36%; top: 0; width: 28%; height: 100%; background: #d6b37d; }
  .review-map-wall.left { left: 24%; top: 0; width: 10%; height: 72%; background: #7b7b82; }
  .review-map-wall.right { right: 24%; top: 0; width: 10%; height: 72%; background: #7b7b82; }
  .review-map-shadow { inset: auto 0 0 0; height: 40%; background: linear-gradient(180deg, transparent, rgba(0,0,0,0.18)); }
  .review-pan-joystick,
  .review-zoom-control,
  .review-layer-float {
    position: absolute;
    background: rgba(28, 28, 30, 0.86);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  .review-pan-joystick {
    left: 18px;
    bottom: calc(
      var(--review-editor-nav-height) +
      var(--review-editor-toolbar-height) +
      var(--review-editor-float-gap)
    );
    width: 92px;
    height: 92px;
    border-radius: 999px;
    color: #d3d6dc;
    z-index: 12;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
  }
  .review-history-float {
    left: 18px;
    top: 4.5px;
    position: absolute;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0;
    background: transparent;
    border: none;
    backdrop-filter: none;
    z-index: 12;
  }
  .review-selection-actions {
    position: absolute;
    display: flex;
    align-items: stretch;
    gap: 0;
    padding: 3px 4px;
    border-radius: 14px;
    background: rgba(38, 38, 40, 0.92);
    backdrop-filter: blur(14px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.24);
    z-index: 13;
    animation: review-selection-actions-fade 180ms ease-out;
    max-width: calc(100% - 20px);
  }
  .review-selection-actions.closing {
    pointer-events: none;
    animation: review-selection-actions-fade-out 170ms ease-out forwards;
  }
  .review-selection-action {
    min-width: 42px;
    padding: 6px 4px 5px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    border: none;
    background: transparent;
    color: rgba(255,255,255,0.80);
  }
  .review-selection-action + .review-selection-action {
    border-left: 1px solid rgba(255,255,255,0.06);
  }
  .review-selection-action-icon {
    width: 16px;
    height: 16px;
    display: grid;
    place-items: center;
  }
  .review-selection-action span {
    font-size: 9px;
    line-height: 1;
    letter-spacing: -0.02em;
  }
  @keyframes review-selection-actions-fade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes review-selection-actions-fade-out {
    from {
      opacity: 1;
    }
    to {
      opacity: 0;
    }
  }
  .review-history-button {
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border: none;
    border-radius: 999px;
    background: rgba(28, 28, 30, 0.86);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.22);
    color: rgba(255,255,255,0.92);
    padding: 0;
  }
  .review-history-button.disabled {
    color: rgba(255,255,255,0.34);
    background: rgba(28, 28, 30, 0.58);
  }
  .review-pan-joystick-ring,
  .review-pan-joystick-center-mark,
  .review-pan-joystick-knob,
  .review-zoom-control-track,
  .review-zoom-control-knob,
  .review-zoom-control-glyph {
    pointer-events: none;
  }
  .review-pan-joystick-ring {
    position: absolute;
    inset: 10px;
    border-radius: 999px;
    border: 1px solid rgba(255,255,255,0.12);
    background:
      radial-gradient(circle at center, rgba(255,255,255,0.04) 0, rgba(255,255,255,0.02) 42%, transparent 72%);
  }
  .review-pan-joystick-center-mark {
    position: absolute;
    inset: 31px;
    display: grid;
    place-items: center;
    border-radius: 999px;
    color: rgba(211, 214, 220, 0.85);
  }
  .review-pan-joystick-knob {
    position: absolute;
    left: 30px;
    top: 30px;
    width: 32px;
    height: 32px;
    border-radius: 999px;
    background: rgba(255,255,255,0.12);
    border: 1px solid rgba(255,255,255,0.14);
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.28);
  }
  .review-zoom-control {
    right: 18px;
    bottom: calc(
      var(--review-editor-nav-height) +
      var(--review-editor-toolbar-height) +
      var(--review-editor-float-gap)
    );
    width: 118px;
    height: 42px;
    border-radius: 999px;
    z-index: 12;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
  }
  .review-zoom-control-track {
    position: absolute;
    inset: 9px 22px;
    border-radius: 999px;
    background: rgba(255,255,255,0.06);
  }
  .review-zoom-control-knob {
    position: absolute;
    left: 33px;
    top: 6px;
    width: 52px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 999px;
    background: rgba(255,255,255,0.12);
    border: 1px solid rgba(255,255,255,0.14);
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.24);
  }
  .review-zoom-control-label {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: #d3d6dc;
    line-height: 1;
  }
  .review-zoom-control-glyph {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    color: rgba(211, 214, 220, 0.7);
    font-size: 18px;
    line-height: 1;
  }
  .review-zoom-control-glyph.minus {
    left: 11px;
  }
  .review-zoom-control-glyph.plus {
    right: 11px;
  }
  .review-dpad-icon-svg {
    width: 18px;
    height: 18px;
    display: block;
  }
  .review-dpad-center-svg {
    width: 18px;
    height: 18px;
    display: block;
  }
  .review-layer-float {
    right: 18px;
    top: 4.5px;
    width: 158px;
    border-radius: 14px;
    padding: 8px 10px 6px;
    z-index: 12;
  }
  .review-layer-float-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    margin: 0;
    padding: 0;
    border: none;
    background: transparent;
    color: inherit;
    font-size: 12px;
    font-weight: 600;
    text-align: left;
  }
  .review-layer-float-title-stack {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1px;
    min-width: 0;
  }
  .review-layer-float-current {
    max-width: 112px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: rgba(255,255,255,0.66);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.01em;
  }
  .review-layer-float-title-icon,
  .review-eye,
  .review-menu-glyph,
  .review-lock {
    display: grid;
    place-items: center;
  }
  .review-layer-float-body {
    max-height: 0;
    opacity: 0;
    margin-top: 0;
    overflow: hidden;
    transition:
      max-height 220ms cubic-bezier(0.22, 1, 0.36, 1),
      opacity 160ms ease,
      margin-top 220ms cubic-bezier(0.22, 1, 0.36, 1);
  }
  .review-layer-float-body.expanded {
    max-height: 176px;
    opacity: 1;
    margin-top: 5px;
  }
  .review-layer-float-list {
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 176px;
    overflow-y: auto;
  }
  .review-layer-float-item {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr) 14px;
    gap: 6px;
    align-items: center;
    padding: 7px 8px;
    border-radius: 10px;
    background: rgba(255,255,255,0.02);
  }
  .review-layer-float-item:first-of-type {
    border-top: none;
  }
  .review-layer-float-switch {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
    min-width: 0;
    border: none;
    background: transparent;
    color: inherit;
    padding: 0;
    text-align: left;
  }
  .review-layer-float-kind {
    color: rgba(255,255,255,0.66);
    flex: none;
  }
  .review-layer-float-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .review-layer-float-item.active {
    color: #fff;
    background: rgba(142, 142, 147, 0.24);
    box-shadow: inset 0 0 0 1px rgba(255,255,255,0.08);
  }
  .review-layer-float-item.active .review-layer-float-kind {
    color: rgba(255,255,255,0.9);
  }
  .review-menu-glyph,
  .review-eye,
  .review-lock {
    color: #8e8e93;
    text-align: center;
  }
  .review-inline-icon-svg {
    width: 14px;
    height: 14px;
    display: block;
  }
  .review-layer-toggle {
    border: none;
    background: transparent;
    padding: 0;
  }
  .review-eye.on,
  .review-lock.on {
    color: #0a84ff;
  }
  .review-layer-float-item.muted {
    color: #8e8e93;
  }
  .review-editor-toolbar {
    flex: none;
    background: rgba(28, 28, 30, 0.94);
    backdrop-filter: blur(14px);
    border-top: 1px solid #2c2c2e;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .review-tool-row-shell {
    display: flex;
    align-items: stretch;
    gap: 0;
    padding: 4px 8px 2px;
  }
  .review-tool-row {
    display: flex;
    align-items: stretch;
    gap: 3px;
    padding: 0 0 0 6px;
    flex: 1 1 auto;
    overflow-x: auto;
    overflow-y: hidden;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
  }
  .review-tool-row::-webkit-scrollbar {
    display: none;
  }
  .review-tool-row-live {
    flex-wrap: nowrap;
  }
  .review-tool-row-swap {
    animation: review-toolbar-swap-in 180ms ease;
  }
  .review-tool-row-object .review-tool {
    flex-basis: 66px;
  }
  .review-tool-subbutton {
    min-height: 34px;
    border-radius: 9px;
    border: none;
    background: transparent;
    color: #d1d1d6;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 3px 1px;
    font: inherit;
    font-size: 8px;
    line-height: 1.05;
    text-align: center;
    flex: none;
  }
  .review-tool-subbutton.active {
    background: rgba(142, 142, 147, 0.18);
    color: #fff;
  }
  .review-tool-subbutton.placeholder {
    color: #8e8e93;
  }
  .review-tool-subbutton-icon {
    width: 15px;
    height: 15px;
    display: grid;
    place-items: center;
  }
  .review-tool-subbutton-icon .review-tool-icon-svg {
    width: 15px;
    height: 15px;
  }
  .review-tool-pinned {
    flex: 0 0 52px;
  }
  .review-tool-divider {
    width: 1px;
    align-self: stretch;
    margin: 0 0 0 6px;
    background: rgba(255, 255, 255, 0.10);
    border-radius: 999px;
    flex: none;
  }
  .review-tool {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    color: #8e8e93;
    border: none;
    background: transparent;
    font: inherit;
    flex: 0 0 60px;
    min-height: 42px;
    padding: 0;
  }
  .review-tool.active {
    color: #d1d1d6;
  }
  .review-tool.placeholder {
    color: #6e6e73;
  }
  .review-tool-icon {
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
  }
  .review-tool span {
    text-align: center;
    line-height: 1.05;
    font-size: 10px;
  }
  .review-tool-icon-svg {
    width: 20px;
    height: 20px;
    display: block;
  }
  @keyframes review-toolbar-swap-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .review-tile-strip {
    flex: none;
    display: grid;
    grid-template-rows: repeat(2, 44px);
    grid-auto-flow: column;
    grid-auto-columns: 44px;
    gap: 6px;
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0 14px 10px;
  }
  .review-tile-strip-top-shell {
    box-sizing: border-box;
    height: 114px;
    min-height: 114px;
    max-height: 114px;
    display: flex;
    align-items: stretch;
    border-bottom: 1px solid #2c2c2e;
    background: #18181a;
  }
  .review-tile-strip-live {
    min-height: 68px;
    padding-top: 6px;
    border-top: 1px solid #2c2c2e;
    background: #18181a;
    align-items: center;
  }
  .review-tile-strip-top {
    padding: 10px 14px 10px;
    align-items: start;
    align-content: start;
    justify-items: stretch;
  }
  .review-tile-strip-top-main {
    flex: 1 1 auto;
    min-width: 0;
    padding-right: 6px;
  }
  .review-tile-strip-side-divider {
    width: 1px;
    margin: 10px 0;
    background: rgba(255,255,255,0.10);
    border-radius: 999px;
    flex: none;
  }
  .review-tile-strip-side {
    width: 62px;
    min-width: 62px;
    position: relative;
    padding: 8px 4px 8px 4px;
    overflow: hidden;
  }
  .review-tile-strip-side-pane {
    position: absolute;
    inset: 8px 4px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    opacity: 0;
    transform: translateX(10px);
    transition: opacity 180ms ease, transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
    pointer-events: none;
    overflow-y: auto;
    overflow-x: hidden;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
  }
  .review-tile-strip-side-pane::-webkit-scrollbar {
    display: none;
  }
  .review-tile-strip-side-pane.active {
    opacity: 1;
    transform: translateX(0);
    pointer-events: auto;
  }
  .review-tile-strip-side-empty {
    align-items: center;
    justify-content: center;
    gap: 3px;
    color: #6e6e73;
    font-size: 9px;
    line-height: 1.05;
    text-align: center;
  }
  .review-tile-chip {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 8px;
    border: 1px solid #2c2c2e;
    background: #486d35;
    background-repeat: no-repeat;
    background-position: center;
  }
  .review-tile-chip.selected {
    border: 2px solid #0a84ff;
  }
  .review-tile-chip.live {
    background-color: #101113;
  }
  .review-tile-chip.path { background: #b98b54; }
  .review-tile-chip.sand { background: #d2bb81; }
  .review-tile-chip.stone { background: #868993; }
  .review-tile-chip.fence { background: #7b5530; }
  .review-tile-chip.tree { background: #3d6336; }
  .review-tile-chip.tree2 { background: #567f47; }
  .review-section-title,
  .review-caption {
    font-size: 13px;
    font-weight: 600;
    color: #8e8e93;
  }
  .review-section-title {
    font-size: 17px;
    color: #f5f5f7;
  }
  .review-section-title.with-gap { margin-top: 10px; }
  .review-tileset-sheet,
  .review-settings-card {
    display: grid;
    gap: 1px;
    padding: 1px;
    border-radius: 14px;
    background: #2c2c2e;
  }
  .review-tileset-sheet {
    grid-template-columns: repeat(6, minmax(0, 1fr));
    overflow: hidden;
    background: #3a3a3c;
  }
  .review-sheet-cell {
    aspect-ratio: 1;
    background: #151517;
    border: none;
    background-repeat: no-repeat;
    background-position: center;
  }
  .review-sheet-cell.active {
    box-shadow: inset 0 0 0 3px #0a84ff;
  }
  .review-sheet-cell.live {
    background-color: #101113;
  }
  .review-input-row,
  .review-setting-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 0;
  }
  .review-setting-row {
    justify-content: space-between;
  }
  .review-setting-row > .review-color-chip,
  .review-setting-row > .review-toggle,
  .review-setting-row > .review-link-button,
  .review-setting-row > .muted {
    margin-left: auto;
  }
  .review-input-row .label {
    width: 64px;
    color: #8e8e93;
  }
  .review-input-box {
    flex: 1;
    min-height: 44px;
    display: flex;
    align-items: center;
    padding: 0 14px;
    border-radius: 12px;
    background: #2c2c2e;
  }
  .review-toggle {
    width: 52px;
    height: 32px;
    border-radius: 999px;
    background: #3a3a3c;
    position: relative;
  }
  .review-toggle.on { background: #0a84ff; }
  .review-toggle .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 26px;
    height: 26px;
    border-radius: 999px;
    background: #fff;
  }
  .review-toggle.on .knob { left: 23px; }
  .review-stepper {
    display: flex;
    align-items: center;
    border-radius: 12px;
    overflow: hidden;
    background: #2c2c2e;
  }
  .review-stepper button,
  .review-stepper span {
    min-width: 44px;
    min-height: 40px;
    display: grid;
    place-items: center;
    background: transparent;
    border: none;
    color: inherit;
  }
  .review-collision-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: center;
  }
  .review-collision-tools {
    display: flex;
    gap: 16px;
    color: #8e8e93;
  }
  .review-collision-tool.active {
    color: #0a84ff;
  }
  .review-collision-preview {
    width: 72px;
    height: 72px;
    border-radius: 14px;
    background: linear-gradient(135deg, #7b5530, #30271f);
    border: 2px solid #5ccb5f;
  }
  .review-opacity {
    margin-left: auto;
    width: 150px;
    color: #8e8e93;
    font-size: 13px;
  }
  .review-opacity span:first-child {
    margin-right: 8px;
  }
  .review-slider-track {
    position: relative;
    height: 6px;
    margin-top: 8px;
    border-radius: 999px;
    background: #3a3a3c;
  }
  .review-slider-track.wide {
    flex: 1;
    margin-top: 0;
  }
  .review-slider-fill {
    height: 100%;
    border-radius: 999px;
    background: #0a84ff;
  }
  .review-slider-knob {
    position: absolute;
    top: -7px;
    width: 20px;
    height: 20px;
    border-radius: 999px;
    background: #fff;
  }
  .review-layer-row.active {
    border-color: #0a84ff;
    box-shadow: inset 0 0 0 1px rgba(10, 132, 255, 0.24);
  }
  .review-layer-name-button {
    border: none;
    background: transparent;
    color: inherit;
    padding: 0;
    text-align: left;
  }
  .review-layer-title-stack {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .review-empty-row {
    min-height: 74px;
    border-radius: 20px;
    border: 1px solid #2c2c2e;
    background: #1c1c1e;
    opacity: 0.55;
  }
  .review-search-bar {
    min-height: 52px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 16px;
    border-radius: 16px;
    border: 1px solid #2c2c2e;
    background: #1c1c1e;
  }
  .review-object-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }
  .review-object-card {
    min-height: 150px;
    padding: 16px 10px;
    border-radius: 18px;
    border: 1px solid #2c2c2e;
    background: #1c1c1e;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
  }
  .review-object-card.active {
    border-color: #0a84ff;
    background: #173457;
  }
  .review-object-card.ghost {
    opacity: 0.35;
  }
  .review-object-art {
    width: 58px;
    height: 58px;
    border-radius: 16px;
    background-color: transparent;
    background-position: center;
    background-repeat: no-repeat;
    background-size: contain;
  }
  .review-object-art.live {
    border-radius: 14px;
  }
  .review-object-art.villager { background-image: url('/assets/review/object-villager.png'); }
  .review-object-art.chest { background-image: url('/assets/review/object-chest.png'); }
  .review-object-art.portal { background-image: url('/assets/review/object-portal.png'); }
  .review-object-art.slime { background-image: url('/assets/review/object-slime.png'); }
  .review-object-art.potion { background-image: url('/assets/review/object-potion.png'); }
  .review-object-art.flag { background-image: url('/assets/review/object-flag.png'); }
  .review-object-card-label {
    text-align: center;
    line-height: 1.3;
  }
  .review-settings-card {
    padding: 0 16px;
    background: #1c1c1e;
  }
  .review-settings-card.single {
    padding: 0 16px;
  }
  .review-settings-inline-stack {
    width: 100%;
  }
  .review-selected-tile-summary {
    font-size: 16px;
    font-weight: 600;
    color: #f5f5f7;
  }
  .review-property-field-card,
  .review-property-group-card {
    border-radius: 14px;
    background: #1c1c1e;
    border: 1px solid #2c2c2e;
  }
  .review-property-field-row {
    display: grid;
    grid-template-columns: 72px minmax(0, 1fr);
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-top: 1px solid #2c2c2e;
  }
  .review-property-field-row:first-child {
    border-top: none;
  }
  .review-property-field-label {
    color: #f2f2f7;
    font-size: 15px;
  }
  .review-property-field-value {
    min-height: 40px;
    display: flex;
    align-items: center;
    padding: 0 14px;
    border-radius: 10px;
    background: #2c2c2e;
    color: #f2f2f7;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .review-property-empty-row {
    justify-content: center;
    min-height: 56px;
    padding: 18px 16px 12px;
  }
  .review-property-empty-row .muted {
    width: 100%;
    padding: 0 8px;
    text-align: center;
    line-height: 1.4;
  }
  .review-property-add-link {
    display: block;
    width: 100%;
    padding: 0 0 16px;
    color: #0a84ff;
    text-align: center;
    font-size: 16px;
  }
  .review-property-footer-note {
    color: #8e8e93;
    font-size: 13px;
    line-height: 1.35;
  }
  .review-actions-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
  }
  .review-secondary-button.compact {
    min-height: 48px;
    font-size: 14px;
  }
  .review-field {
    display: flex;
    flex-direction: column;
    gap: 8px;
    color: #b6b6bb;
  }
  .review-field input {
    min-height: 46px;
    border-radius: 14px;
    border: 1px solid #2c2c2e;
    background: #1c1c1e;
    color: #f2f2f7;
    padding: 0 14px;
    font: inherit;
  }
  .review-note-card {
    flex-direction: column;
    align-items: flex-start;
  }
  .review-setting-meta {
    text-align: right;
    max-width: 62%;
  }
  .review-about-entry-card {
    align-items: center;
    text-align: center;
  }
  .review-about-entry-card .review-link-button {
    margin: 2px auto 0;
  }
  .review-about-link-list {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .review-about-link {
    display: flex;
    flex-direction: column;
    gap: 2px;
    color: #9cc7ff;
    text-decoration: none;
  }
  .review-about-link-title {
    color: #f2f2f7;
    font-size: 14px;
    font-weight: 600;
  }
  .review-about-link-url {
    color: #74a8ff;
    font-size: 12px;
    line-height: 1.35;
    word-break: break-all;
  }
  .review-about-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 20px 18px 18px;
    border-radius: 22px;
    border: 1px solid #2c2c2e;
    background: linear-gradient(180deg, rgba(33,33,36,0.98), rgba(24,24,26,0.98));
    text-align: center;
  }
  .review-about-logo {
    width: 84px;
    height: 84px;
    display: block;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    filter: drop-shadow(0 8px 20px rgba(0, 0, 0, 0.28));
  }
  .review-disclosure-button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0;
    border: none;
    background: transparent;
    color: inherit;
    font: inherit;
    text-align: left;
  }
  .review-disclosure-copy {
    margin-left: auto;
    color: #8f8f95;
    font-size: 12px;
    line-height: 1.2;
    white-space: nowrap;
  }
  .review-disclosure-panel {
    width: 100%;
    max-height: 0;
    overflow: hidden;
    opacity: 0;
    transition: max-height 220ms ease, opacity 180ms ease, margin-top 180ms ease;
  }
  .review-disclosure-panel.expanded {
    max-height: 240px;
    opacity: 1;
    margin-top: 10px;
  }
  .review-settings-card.about-embedded {
    width: 100%;
    box-sizing: border-box;
    padding: 0 14px;
    border-radius: 16px;
    border: 1px solid #2c2c2e;
    background: #1a1a1c;
    overflow: hidden;
  }
  .review-license-card {
    display: block;
    color: inherit;
    text-decoration: none;
  }
  .review-contributor-row {
    display: flex;
    width: 100%;
    box-sizing: border-box;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    padding: 14px 0;
  }
  .review-contributor-meta {
    width: 100%;
    max-width: none;
    text-align: left;
    white-space: normal;
    overflow-wrap: anywhere;
  }
  .review-map-live .canvas-host {
    height: 100%;
    padding: 0;
    background: transparent;
    overflow: hidden;
    touch-action: none;
  }
  .review-map-live .canvas-stage {
    min-width: 100%;
    min-height: 100%;
    justify-content: flex-start;
    align-items: flex-start;
    touch-action: none;
  }
  .review-map-live .canvas {
    border-radius: 0;
    box-shadow: none;
    touch-action: none;
  }
  .review-map-live .cell-hitbox {
    border-color: rgba(255,255,255,0.03);
  }
  .review-color-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 12px;
    background: #2c2c2e;
  }
  .review-color-chip .swatch {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    background: #ccc;
  }
  .review-segmented {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 2px;
    width: 100%;
    padding: 4px;
    border-radius: 16px;
    background: #2c2c2e;
  }
  .review-segmented button {
    width: 100%;
    min-height: 40px;
    border-radius: 12px;
    border: none;
    background: transparent;
    color: #8e8e93;
    font: inherit;
  }
  .review-segmented button.active {
    background: #4d4d52;
    color: #fff;
  }
  .review-sync-button {
    min-height: 56px;
    color: #0a84ff;
    font-size: 18px;
  }
  .review-sync-meta {
    margin-top: -6px;
    margin-left: 8px;
    font-size: 12px;
  }
  @media (max-width: 900px) {
    .topbar,
    .workspace {
      display: none;
    }
    .review-shell {
      display: flex;
      flex-direction: column;
    }
  }
"#;
