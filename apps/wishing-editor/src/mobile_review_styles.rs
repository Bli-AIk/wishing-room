pub(crate) const MOBILE_REVIEW_STYLES: &str = r#"
  .review-shell {
    display: none;
    min-height: 100dvh;
    background: #121212;
    color: #ffffff;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Inter", sans-serif;
  }
  .review-page {
    display: flex;
    flex-direction: column;
    min-height: 100dvh;
    background: #121212;
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
    overflow: auto;
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
  .review-editor-canvas {
    position: relative;
    flex: 1;
    min-height: 720px;
    overflow: hidden;
    --grid-size-x: 16px;
    --grid-size-y: 16px;
    --grid-offset-x: 0px;
    --grid-offset-y: 0px;
    background:
      linear-gradient(#333 1px, transparent 1px),
      linear-gradient(90deg, #333 1px, transparent 1px),
      #2a2a2a;
    background-size: var(--grid-size-x) var(--grid-size-y);
    background-position: var(--grid-offset-x) var(--grid-offset-y);
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
  .review-dpad,
  .review-layer-float {
    position: absolute;
    background: rgba(28, 28, 30, 0.86);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  .review-dpad {
    left: 18px;
    bottom: 18px;
    width: 118px;
    height: 118px;
    border-radius: 999px;
    color: #d3d6dc;
  }
  .review-dpad span {
    border: none;
    background: transparent;
    color: inherit;
  }
  .review-dpad button {
    position: absolute;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    color: inherit;
    display: grid;
    place-items: center;
  }
  .review-dpad .up { top: 6px; left: 41px; }
  .review-dpad .left { top: 41px; left: 6px; }
  .review-dpad .center { inset: 41px auto auto 41px; width: 36px; height: 36px; border-radius: 999px; background: rgba(255,255,255,0.08); }
  .review-dpad .right { top: 41px; right: 6px; }
  .review-dpad .down { bottom: 6px; left: 41px; }
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
    bottom: 18px;
    width: 214px;
    border-radius: 20px;
    padding: 12px 14px;
  }
  .review-layer-float-title {
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 16px;
    font-weight: 600;
  }
  .review-layer-float-title-icon,
  .review-eye,
  .review-menu-glyph {
    display: grid;
    place-items: center;
  }
  .review-layer-float-item {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr) 16px;
    gap: 8px;
    align-items: center;
    padding: 10px 0;
    border-top: 1px solid rgba(255,255,255,0.06);
  }
  .review-layer-float-item:first-of-type {
    border-top: none;
  }
  .review-layer-float-item button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    min-width: 0;
    border: none;
    background: transparent;
    color: inherit;
    padding: 0;
    text-align: left;
  }
  .review-layer-float-item .muted {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .review-layer-float-item.active {
    color: #fff;
  }
  .review-menu-glyph,
  .review-eye,
  .review-lock {
    color: #8e8e93;
    text-align: center;
  }
  .review-inline-icon-svg {
    width: 18px;
    height: 18px;
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
    background: #1c1c1e;
    border-top: 1px solid #2c2c2e;
    display: flex;
    flex-direction: column;
  }
  .review-tool-row {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    padding: 14px 18px 8px;
  }
  .review-tool-row-live {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }
  .review-tool {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: #8e8e93;
    border: none;
    background: transparent;
    font: inherit;
    min-height: 62px;
  }
  .review-tool.active {
    color: #0a84ff;
  }
  .review-tool-icon {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
  }
  .review-tool-icon-svg {
    width: 24px;
    height: 24px;
    display: block;
  }
  .review-tile-strip {
    display: grid;
    grid-template-rows: repeat(2, 52px);
    grid-auto-flow: column;
    grid-auto-columns: 52px;
    gap: 8px;
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0 14px 12px;
  }
  .review-tile-strip-live {
    min-height: 68px;
    padding-top: 6px;
    border-top: 1px solid #2c2c2e;
    background: #18181a;
    align-items: center;
  }
  .review-tile-strip-top {
    min-height: 128px;
    padding: 12px 14px 14px;
    border-bottom: 1px solid #2c2c2e;
    background: #18181a;
    align-items: start;
    justify-items: stretch;
  }
  .review-tile-chip {
    width: 52px;
    height: 52px;
    flex: none;
    border-radius: 10px;
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
    border-radius: 16px;
    background: #2c2c2e;
  }
  .review-tileset-sheet {
    grid-template-columns: repeat(6, minmax(0, 1fr));
    overflow: hidden;
  }
  .review-sheet-cell {
    aspect-ratio: 1;
    background: linear-gradient(135deg, #4e3d2f, #161616);
    border: none;
    background-repeat: no-repeat;
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
  .review-selected-tile-card {
    align-items: center;
  }
  .review-selected-tile-art {
    width: 72px;
    height: 72px;
    border-radius: 16px;
    background-color: #11161f;
    background-repeat: no-repeat;
    border: 1px solid rgba(255,255,255,0.08);
    flex: none;
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
  .review-map-live .canvas-host {
    height: 100%;
    padding: 0;
    background: transparent;
    overflow: auto;
  }
  .review-map-live .canvas-stage {
    min-width: 100%;
    min-height: 100%;
    justify-content: flex-start;
    align-items: flex-start;
  }
  .review-map-live .canvas {
    border-radius: 0;
    box-shadow: none;
  }
  .review-map-live .cell-hitbox {
    border-color: rgba(255,255,255,0.04);
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
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 2px;
    padding: 4px;
    border-radius: 16px;
    background: #2c2c2e;
  }
  .review-segmented button {
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
