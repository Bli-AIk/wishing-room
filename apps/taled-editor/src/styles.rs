pub(crate) const STYLES: &str = r#"
  html, body {
    width: 100%;
    max-width: 100%;
    overflow-x: hidden;
  }
  body {
    margin: 0;
    font-family: "Iosevka Term", "Sarasa Mono SC", monospace;
    background: #0f1720;
    color: #d9e3f0;
  }
  button, input {
    font: inherit;
  }
  .app-shell {
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 100vh;
    width: 100%;
    max-width: 100%;
    overflow-x: clip;
    background:
      radial-gradient(circle at top left, rgba(255, 186, 73, 0.16), transparent 28%),
      linear-gradient(180deg, #101927 0%, #091019 100%);
  }
  .topbar {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
    min-width: 0;
    padding: 14px 18px;
    border-bottom: 1px solid rgba(138, 158, 181, 0.2);
    background: rgba(5, 9, 14, 0.65);
    backdrop-filter: blur(16px);
  }
  .topbar input {
    background: rgba(15, 23, 32, 0.95);
    border: 1px solid rgba(138, 158, 181, 0.24);
    color: #eef4fb;
    padding: 8px 10px;
    border-radius: 10px;
    min-width: 260px;
  }
  .topbar button, .panel button {
    background: #1b3348;
    color: #eef4fb;
    border: 1px solid rgba(126, 189, 255, 0.18);
    padding: 8px 12px;
    border-radius: 10px;
    cursor: pointer;
  }
  .topbar button:hover, .panel button:hover {
    background: #214565;
  }
  .topbar .status {
    margin-left: auto;
    color: #91b6d8;
    font-size: 13px;
    max-width: 32rem;
    text-align: right;
  }
  .workspace {
    display: grid;
    grid-template-columns: 280px 1fr 340px;
    min-height: 0;
    min-width: 0;
    max-width: 100%;
  }
  .workspace > * {
    min-width: 0;
  }
  .desktop-panel {
    display: block;
  }
  .panel {
    border-right: 1px solid rgba(138, 158, 181, 0.14);
    padding: 16px;
    overflow: auto;
    min-width: 0;
    background: rgba(8, 13, 21, 0.72);
  }
  .panel.right {
    border-right: none;
    border-left: 1px solid rgba(138, 158, 181, 0.14);
  }
  .panel h2, .panel h3 {
    margin: 0 0 10px;
    font-size: 14px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #9fb7cf;
  }
  .tool-grid, .zoom-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
    margin-bottom: 14px;
  }
  .tool-grid button.active, .layer-row.active button.name {
    background: #d77b3f;
    border-color: rgba(255, 201, 166, 0.4);
    color: #081019;
    font-weight: 700;
  }
  .layer-list, .object-list, .property-list, .tileset-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .layer-row, .object-row, .property-row, .tileset-card {
    padding: 10px;
    border-radius: 12px;
    background: rgba(16, 24, 36, 0.94);
    border: 1px solid rgba(138, 158, 181, 0.16);
  }
  .layer-row {
    display: grid;
    gap: 8px;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
  }
  .layer-row button.name {
    text-align: left;
  }
  .layer-name-stack {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .layer-kind {
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #7f99b3;
  }
  .layer-row .meta {
    display: flex;
    gap: 6px;
  }
  .canvas-host {
    overflow: auto;
    position: relative;
    min-width: 0;
    max-width: 100%;
    padding: 18px;
    overscroll-behavior: contain;
    touch-action: none;
  }
  .canvas-stage {
    width: max-content;
    min-height: 100%;
    min-width: 100%;
    display: flex;
    align-items: start;
    justify-content: start;
    touch-action: none;
  }
  .canvas {
    position: relative;
    background:
      linear-gradient(90deg, rgba(255,255,255,0.028) 0.5px, transparent 0.5px),
      linear-gradient(180deg, rgba(255,255,255,0.028) 0.5px, transparent 0.5px),
      #142131;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.45);
    transform-origin: top left;
    touch-action: none;
  }
  .tile-sprite, .tile-preview, .shape-fill-preview-frame, .cell-hitbox, .object-overlay {
    position: absolute;
    box-sizing: border-box;
  }
  .tile-sprite {
    z-index: 1;
  }
  .tile-preview,
  .shape-fill-preview-tile {
    position: absolute;
    box-sizing: border-box;
    z-index: 4;
    pointer-events: none;
  }
  .shape-fill-preview-tile.fallback {
    background: rgba(203, 213, 225, 0.20);
  }
  .shape-fill-preview-frame {
    z-index: 5;
    pointer-events: none;
    border: 0.5px solid rgba(203, 213, 225, 0.56);
    background: rgba(203, 213, 225, 0.045);
  }
  .tile-selection-region,
  .tile-selection-irregular-bounds,
  .tile-selection-region-cells,
  .tile-selection-cell-fragment,
  .tile-selection-frame {
    position: absolute;
    box-sizing: border-box;
    pointer-events: none;
  }
  .tile-selection-handle {
    position: absolute;
    box-sizing: border-box;
    width: 22px;
    height: 22px;
    border-radius: 999px;
    background: transparent;
    pointer-events: auto;
    touch-action: none;
    z-index: 7;
  }
  .tile-selection-region {
    z-index: 6;
    background: rgba(58, 174, 255, 0.16);
    box-shadow:
      inset 0 0 0 0.5px rgba(58, 174, 255, 0.92),
      0 0 10px rgba(58, 174, 255, 0.18);
    animation: tile-selection-fade-in 160ms ease-out;
  }
  .tile-selection-region-cells {
    z-index: 6;
    animation: tile-selection-fade-in 160ms ease-out;
  }
  .tile-selection-irregular-bounds {
    z-index: 5;
    background: rgba(58, 174, 255, 0.04);
    border: 0.5px dashed rgba(90, 196, 255, 0.54);
    animation: tile-selection-fade-in 160ms ease-out;
  }
  .tile-selection-irregular-bounds.preview {
    background: rgba(58, 174, 255, 0.025);
    border-color: rgba(90, 196, 255, 0.42);
  }
  .tile-selection-irregular-bounds.closing {
    background: rgba(58, 174, 255, 0.025);
    border-color: rgba(90, 196, 255, 0.38);
    animation: tile-selection-fade-out 170ms ease-out forwards;
  }
  .tile-selection-region-cells.preview {
    opacity: 0.9;
  }
  .tile-selection-region-cells.closing {
    animation: tile-selection-fade-out 170ms ease-out forwards;
  }
  .tile-selection-cell-fragment {
    background: rgba(58, 174, 255, 0.14);
    box-shadow: inset 0 0 0 0.5px rgba(90, 196, 255, 0.84);
  }
  .tile-selection-region.preview {
    background: rgba(58, 174, 255, 0.11);
    box-shadow:
      inset 0 0 0 0.5px rgba(58, 174, 255, 0.74),
      0 0 8px rgba(58, 174, 255, 0.14);
  }
  .tile-selection-region.closing {
    background: rgba(58, 174, 255, 0.11);
    box-shadow:
      inset 0 0 0 0.5px rgba(58, 174, 255, 0.62),
      0 0 7px rgba(58, 174, 255, 0.10);
    animation: tile-selection-fade-out 170ms ease-out forwards;
  }
  .tile-selection-frame {
    inset: 0;
    border: 0.5px solid rgba(90, 196, 255, 0.94);
  }
  .tile-selection-handle-dot {
    position: absolute;
    inset: 6px;
    width: 10px;
    height: 10px;
    border-radius: 999px;
    background: #eef8ff;
    border: 0.5px solid rgba(90, 196, 255, 0.98);
    box-shadow:
      0 0 0 1px rgba(28, 106, 158, 0.52),
      0 0 10px rgba(58, 174, 255, 0.24);
  }
  .tile-selection-handle.ghost {
    pointer-events: none;
  }
  .tile-selection-handle-dot.ghost {
    inset: 6px;
    background: rgba(7, 17, 27, 0.18);
    border: 0.5px solid rgba(173, 228, 255, 0.72);
    box-shadow: 0 0 0 1px rgba(28, 106, 158, 0.24);
  }
  @keyframes tile-selection-fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes tile-selection-fade-out {
    from {
      opacity: 1;
    }
    to {
      opacity: 0;
    }
  }
  .canvas.camera-transition {
    transition: transform 220ms cubic-bezier(0.22, 1, 0.36, 1);
  }
  .cell-hitbox {
    z-index: 2;
    background: transparent;
    border: 0.5px solid rgba(255, 255, 255, 0.048);
    cursor: crosshair;
  }
  .cell-hitbox.selected {
    outline: 0.5px solid rgba(168, 174, 182, 0.68);
    outline-offset: -0.5px;
    background: rgba(168, 174, 182, 0.055);
  }
  .object-overlay {
    z-index: 3;
    cursor: pointer;
    background-repeat: no-repeat;
  }
  .object-overlay.selected {
    filter: drop-shadow(0 0 0.5px rgba(168, 174, 182, 0.42));
  }
  @media (pointer: coarse) {
    .tile-sprite,
    .cell-hitbox,
    .object-overlay {
      pointer-events: none;
    }
  }
  .palette-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(44px, 1fr));
    gap: 8px;
  }
  .palette-tile {
    width: 44px;
    height: 44px;
    padding: 0;
    border-radius: 10px;
    background-color: #081019;
    background-repeat: no-repeat;
    border: 1px solid rgba(138, 158, 181, 0.18);
  }
  .palette-tile.active {
    border: 2px solid #f7b267;
  }
  .field-stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .field-stack label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    color: #93aac0;
  }
  .field-stack input {
    background: #0b131d;
    border: 1px solid rgba(138, 158, 181, 0.18);
    border-radius: 8px;
    color: #eef4fb;
    padding: 7px 9px;
  }
  .object-row button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    text-align: left;
  }
  .object-shape-icon {
    width: 18px;
    height: 18px;
    flex: none;
    background-repeat: no-repeat;
    background-position: center;
    background-size: contain;
  }
  .inline-row {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100%;
    color: #8aa0b7;
    font-size: 18px;
  }
  .web-log-panel {
    position: fixed;
    left: 8px;
    right: 8px;
    bottom: 8px;
    max-height: 48vh;
    padding: 12px;
    border-radius: 14px;
    border: 1px solid rgba(138, 158, 181, 0.24);
    background: rgba(3, 8, 14, 0.96);
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.45);
    z-index: 40;
  }
  .web-log-panel pre {
    margin: 10px 0 0;
    max-height: 32vh;
    overflow: auto;
    white-space: pre-wrap;
    color: #dbe8f4;
    font-size: 12px;
    line-height: 1.5;
  }
"#;
