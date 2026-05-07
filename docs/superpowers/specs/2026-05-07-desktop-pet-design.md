# Desktop Pet Design Spec

## Overview

Add a pixel-art desktop pet (robot) to Agent Sessions. The pet lives in an independent floating window and displays different animations based on the aggregate status of all monitored AI coding agent sessions. Character configuration is JSON-driven to support future character extensions (cat, dog, etc.).

## Architecture

```
src-tauri/src/
├── desktop_pet.rs        ← new: pet window lifecycle + status aggregation + event emission
├── lib.rs                ← modify: register commands, spawn pet window on setup
├── session/mod.rs        ← modify: emit status events after each poll cycle

src/pet/
├── PetApp.tsx            ← pet window root (window setup, drag, event listener)
├── PetSprite.tsx         ← CSS sprite animation renderer
├── usePetState.ts        ← listen Tauri events, derive animation state
└── config.ts             ← character config loader (types + JSON reader)

src-tauri/pets/
├── robot.json            ← robot sprite frame definitions
└── robot.png             ← robot pixel-art sprite sheet

docs/
└── pet-sprite-prompt.md  ← prompt template for generating sprite sheets
```

## Data Flow

```
Rust poll (2s interval)
    → compute aggregate status (thinking > processing > waiting > idle)
    → emit Tauri event "pet-status-changed" { status, activeCount, totalCount }
    → PetApp (via usePetState hook) receives event
    → derives animation state
    → PetSprite renders matching CSS sprite animation
```

No sessions → pet window hidden. Status aggregation picks the most active state across all sessions.

## Window Management (Rust: desktop_pet.rs)

Created at app startup via `WebviewWindowBuilder`:

| Property | Value |
|----------|-------|
| decorations | false |
| transparent | true |
| always_on_top | true |
| skip_taskbar | true |
| visible | true (configurable) |
| width/height | 128×128 |
| default position | bottom-right of primary screen |
| url | `pet.html` (separate entry point) |

Position persisted in localStorage. Restored on launch; clamped to screen bounds if display layout changed.

Lifecycle:
- App starts → pet window created
- Main window closes → pet stays (continues monitoring)
- App quits → pet window destroyed
- Right-click menu (future): Hide / Show / Change Character

## Character Configuration (robot.json)

```json
{
  "name": "robot",
  "sprite": "robot.png",
  "frameWidth": 128,
  "frameHeight": 128,
  "frameRate": 500,
  "states": {
    "idle":       { "startFrame": 0, "count": 4 },
    "thinking":   { "startFrame": 4, "count": 4 },
    "processing": { "startFrame": 8, "count": 4 },
    "waiting":    { "startFrame": 12, "count": 4 },
    "sleeping":   { "startFrame": 16, "count": 2 }
  },
  "defaultState": "sleeping"
}
```

### Extensibility

New characters added by placing `<name>.json` + `<name>.png` in `src-tauri/pets/`. A future Settings dropdown reads available configs from this directory. `config.ts` exposes `loadCharacterConfig(name: string): CharacterConfig`.

## Status → Animation Mapping

| Aggregate Status | Pet Animation | Description |
|-----------------|---------------|-------------|
| thinking | thinking | gears/spark animation (4 frames) |
| processing | processing | typing/working animation (4 frames) |
| waiting | waiting | alert but idle, looking around (4 frames) |
| idle | idle | gentle idle animation, blinking (4 frames) |
| no sessions | sleeping | slow sleep/rest animation (2 frames) |

## Component Details

### PetSprite.tsx

- Receives `state: PetAnimationState` and `config: CharacterConfig`
- Computes CSS `@keyframes` or inline style with stepped `background-position-x`
- Renders a `<div>` sized to `frameWidth × frameHeight` with the sprite as background
- Fallback: if sprite image fails to load, renders a colored geometric shape with status label

### usePetState.ts

- Listens to Tauri event `pet-status-changed`
- Derives `PetAnimationState` from `{ status, activeCount }`
- Exposes: `{ state, activeCount, totalCount }`

### PetApp.tsx

- Root component for `pet.html` entry point
- Calls `usePetState` to get current animation state
- Loads character config (hardcoded to `robot` for v1)
- Renders `PetSprite` with `data-tauri-drag-region` for dragging
- Handles right-click context menu (future)

## Error Handling

- **Sprite load failure** → colored fallback shape with status text (never blank)
- **Event listener error** → retains last known state, no flicker
- **Window creation failure** → silent no-op; pet is optional, main app unaffected
- **Multi-monitor** → restore saved position; clamp to available screen bounds if display layout changed
- **Config parse error** → fallback to hardcoded default config

## Testing

### Rust
- Unit test: status aggregation logic (multiple sessions → single aggregate status)
- Unit test: event payload structure

### Frontend
- `usePetState` hook: mock Tauri event channel, verify state transitions
- `PetSprite`: render test for each animation state, verify correct CSS class
- `config.ts`: valid JSON parsing, missing-key fallback, invalid JSON error handling

## Sprite Generation Prompt

A standardized prompt template for AI image generation tools is maintained at `docs/pet-sprite-prompt.md`. It specifies:
- Canvas size: `frameWidth × frameCount` wide, `frameHeight` tall (e.g., 2304×128 for 18 frames)
- Pixel-art style constraints
- Per-frame animation descriptions for each state
- Technical requirements: transparent background, no anti-aliasing, hard pixel edges

## Out of Scope (v1)

- Right-click context menu (Hide/Change Character)
- Settings UI for pet configuration (always enabled, robot only)
- Pet interaction (click reactions, idle behaviors)
- Animated transitions between states
