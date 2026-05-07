# Desktop Pet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a pixel-art robot desktop pet in a floating Tauri window that shows animations based on aggregate agent session status.

**Architecture:** The pet runs in a separate Tauri webview (frameless, transparent, always-on-top). Both the main window and pet window independently poll `get_all_sessions` every 2s. The pet frontend computes the aggregate status (most active session) and renders CSS sprite animations. Character config is JSON-driven for future extensibility.

**Tech Stack:** Tauri 2.x (Rust) + React 19 + TypeScript + Tailwind CSS 4

---

## File Structure

```
src-tauri/src/
├── desktop_pet.rs        ← CREATE: pet window creation via WebviewWindowBuilder
├── lib.rs                ← MODIFY: call create_pet_window in setup

src/pet/
├── PetApp.tsx            ← CREATE: pet window root, drag region, event wiring
├── PetSprite.tsx         ← CREATE: CSS sprite animation renderer with fallback
├── usePetState.ts        ← CREATE: poll sessions, compute aggregate status
├── config.ts             ← CREATE: types + hardcoded robot config

pet.html                  ← CREATE: pet window entry point
src/pet/main.tsx          ← CREATE: pet window React mount

src-tauri/pets/
├── robot.json            ← CREATE: robot sprite frame definitions

vite.config.ts            ← MODIFY: add pet.html as rollup input
```

---

### Task 1: Rust desktop_pet module (window creation)

**Files:**
- Create: `src-tauri/src/desktop_pet.rs`

- [ ] **Step 1: Write the module**

```rust
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Create the desktop pet floating window.
/// The window is frameless, transparent, always-on-top, and skips the taskbar.
/// If creation fails (e.g., unsupported platform), it fails silently — the
/// pet is an optional enhancement, not a core feature.
pub fn create_pet_window(app: &AppHandle) {
    let result = WebviewWindowBuilder::new(app, "pet", WebviewUrl::App("pet.html".into()))
        .title("Desktop Pet")
        .inner_size(128.0, 128.0)
        .min_inner_size(64.0, 64.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .resizable(false)
        .build();

    match result {
        Ok(window) => {
            log::info!("Desktop pet window created");
            // Position in bottom-right corner of primary monitor
            if let Some(monitor) = window.primary_monitor().ok().flatten() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let x = ((size.width as f64 / scale) - 128.0).max(0.0);
                let y = ((size.height as f64 / scale) - 128.0).max(0.0);
                let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
            }
        }
        Err(e) => {
            log::warn!("Failed to create desktop pet window (non-fatal): {}", e);
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p tauri-temp 2>&1`
Expected: Compilation success (or missing `Manager` import warning — fix by adding the import).

---

### Task 2: Wire pet window creation into lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the module declaration and call in setup**

Add after line 8 (`pub mod terminal;`):
```rust
pub mod desktop_pet;
```

In the `setup` closure, after `Ok(())` line (after the tray setup block), add just before `Ok(())`:
```rust
            // Spawn desktop pet window
            desktop_pet::create_pet_window(app.handle());
```

The setup closure should look like:
```rust
        .setup(|app| {
            // ... existing tray setup code ...

            // Store tray ID
            *TRAY_ID.lock().unwrap() = Some("main-tray".to_string());

            // Spawn desktop pet window
            desktop_pet::create_pet_window(app.handle());

            Ok(())
        })
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p tauri-temp 2>&1`
Expected: Compilation success.

---

### Task 3: Frontend pet types and config

**Files:**
- Create: `src/pet/config.ts`

- [ ] **Step 1: Write config.ts with types and hardcoded robot config**

```typescript
export interface CharacterState {
  startFrame: number;
  count: number;
}

export interface CharacterConfig {
  name: string;
  sprite: string;
  frameWidth: number;
  frameHeight: number;
  frameRate: number; // ms per frame
  states: Record<string, CharacterState>;
  defaultState: string;
}

export type PetAnimationState = 'thinking' | 'processing' | 'waiting' | 'idle' | 'sleeping';

const ROBOT_CONFIG: CharacterConfig = {
  name: 'robot',
  sprite: '/pets/robot.png',
  frameWidth: 128,
  frameHeight: 128,
  frameRate: 500,
  states: {
    idle:       { startFrame: 0, count: 4 },
    thinking:   { startFrame: 4, count: 4 },
    processing: { startFrame: 8, count: 4 },
    waiting:    { startFrame: 12, count: 4 },
    sleeping:   { startFrame: 16, count: 2 },
  },
  defaultState: 'sleeping',
};

/**
 * Load character config by name.
 * v1: returns hardcoded robot config.
 * Future: read from src-tauri/pets/<name>.json at runtime.
 */
export function loadCharacterConfig(_name: string): CharacterConfig {
  return ROBOT_CONFIG;
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `npx tsc --noEmit 2>&1 | head -20`
Expected: No new errors from config.ts.

---

### Task 4: Pet HTML entry point + Vite config

**Files:**
- Create: `pet.html`
- Create: `src/pet/main.tsx`
- Modify: `vite.config.ts`

- [ ] **Step 1: Create pet.html at project root**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Desktop Pet</title>
    <style>
      html, body {
        margin: 0;
        padding: 0;
        background: transparent;
        overflow: hidden;
        user-select: none;
        -webkit-user-select: none;
      }
    </style>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/pet/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 2: Create src/pet/main.tsx**

```typescript
import React from 'react';
import ReactDOM from 'react-dom/client';
import { PetApp } from './PetApp';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <PetApp />
  </React.StrictMode>,
);
```

- [ ] **Step 3: Update vite.config.ts for multi-page build**

Add `build.rollupOptions.input` to the existing config. The config object returned by `defineConfig` should include:

```typescript
build: {
  rollupOptions: {
    input: {
      main: path.resolve(__dirname, 'index.html'),
      pet: path.resolve(__dirname, 'pet.html'),
    },
  },
},
```

Place this at the same level as `clearScreen` and `server` in the config object.

- [ ] **Step 4: Verify build**

Run: `npx vite build 2>&1 | tail -10`
Expected: Build succeeds, both `index.html` and `pet.html` appear in `dist/`.

---

### Task 5: PetSprite component

**Files:**
- Create: `src/pet/PetSprite.tsx`

- [ ] **Step 1: Write PetSprite.tsx**

```typescript
import { useState, useEffect, useCallback } from 'react';
import { CharacterConfig, PetAnimationState } from './config';

interface PetSpriteProps {
  state: PetAnimationState;
  config: CharacterConfig;
}

export function PetSprite({ state, config }: PetSpriteProps) {
  const [frameIndex, setFrameIndex] = useState(0);
  const [imgError, setImgError] = useState(false);

  const stateConfig = config.states[state] ?? config.states[config.defaultState];
  const totalFrames = stateConfig.count;

  // Cycle frame index on a timer
  useEffect(() => {
    if (totalFrames <= 1) {
      setFrameIndex(0);
      return;
    }
    setFrameIndex(0);
    const interval = setInterval(() => {
      setFrameIndex(prev => (prev + 1) % totalFrames);
    }, config.frameRate);
    return () => clearInterval(interval);
  }, [state, config.frameRate, totalFrames]);

  const handleError = useCallback(() => setImgError(true), []);

  const bgX = -(stateConfig.startFrame + frameIndex) * config.frameWidth;

  // Fallback: colored shape when sprite image missing
  if (imgError) {
    const colors: Record<string, string> = {
      thinking: '#fbbf24',
      processing: '#60a5fa',
      waiting: '#a3e635',
      idle: '#94a3b8',
      sleeping: '#6b7280',
    };
    return (
      <div
        data-tauri-drag-region
        className="flex items-center justify-center rounded-full"
        style={{
          width: config.frameWidth,
          height: config.frameHeight,
          backgroundColor: colors[state] ?? colors.sleeping,
          opacity: 0.8,
        }}
      >
        <span className="text-xs font-mono text-white">{state}</span>
      </div>
    );
  }

  return (
    <div
      data-tauri-drag-region
      style={{
        width: config.frameWidth,
        height: config.frameHeight,
        backgroundImage: `url(${config.sprite})`,
        backgroundSize: `${config.frameWidth * 18}px ${config.frameHeight}px`,
        backgroundPositionX: bgX,
        backgroundPositionY: 0,
        backgroundRepeat: 'no-repeat',
        imageRendering: 'pixelated',
        cursor: 'grab',
      }}
      onError={handleError}
    />
  );
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `npx tsc --noEmit 2>&1 | head -20`
Expected: No errors from PetSprite.tsx.

---

### Task 6: usePetState hook

**Files:**
- Create: `src/pet/usePetState.ts`

- [ ] **Step 1: Write usePetState.ts**

```typescript
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PetAnimationState } from './config';

interface SessionsResponse {
  sessions: Array<{ status: string }>;
  totalCount: number;
}

const POLL_INTERVAL = 2000;

const STATUS_PRIORITY: Record<string, number> = {
  thinking: 0,
  processing: 1,
  compacting: 2,
  waiting: 3,
  idle: 4,
};

export function computeAggregate(sessions: Array<{ status: string }>): PetAnimationState {
  if (sessions.length === 0) return 'sleeping';

  let best: PetAnimationState = 'idle';
  let bestPriority = Infinity;

  for (const s of sessions) {
    const priority = STATUS_PRIORITY[s.status] ?? 5;
    if (priority < bestPriority) {
      bestPriority = priority;
      // Map the SessionStatus to PetAnimationState
      switch (s.status) {
        case 'thinking': best = 'thinking'; break;
        case 'processing': best = 'processing'; break;
        case 'compacting': best = 'processing'; break;
        case 'waiting': best = 'waiting'; break;
        case 'idle': best = 'idle'; break;
        default: best = 'idle';
      }
    }
  }
  return best;
}

export function usePetState() {
  const [state, setState] = useState<PetAnimationState>('sleeping');
  const [activeCount, setActiveCount] = useState(0);
  const stateRef = useRef<PetAnimationState>('sleeping');

  useEffect(() => {
    const poll = async () => {
      try {
        const response = await invoke<SessionsResponse>('get_all_sessions');
        const aggregate = computeAggregate(response.sessions);
        setActiveCount(response.totalCount);
        if (aggregate !== stateRef.current) {
          stateRef.current = aggregate;
          setState(aggregate);
        }
      } catch {
        // Silently retain last known state on error
      }
    };

    poll(); // Immediate first poll
    const interval = setInterval(poll, POLL_INTERVAL);
    return () => clearInterval(interval);
  }, []);

  return { state, activeCount };
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `npx tsc --noEmit 2>&1 | head -20`
Expected: No errors from usePetState.ts.

---

### Task 7: PetApp root component + robot.json config

**Files:**
- Create: `src/pet/PetApp.tsx`
- Create: `src-tauri/pets/robot.json`

- [ ] **Step 1: Write PetApp.tsx**

```typescript
import { usePetState } from './usePetState';
import { PetSprite } from './PetSprite';
import { loadCharacterConfig } from './config';

export function PetApp() {
  const { state } = usePetState();
  const config = loadCharacterConfig('robot');

  return (
    <div
      data-tauri-drag-region
      className="fixed inset-0 bg-transparent"
      style={{ cursor: 'grab' }}
    >
      <PetSprite state={state} config={config} />
    </div>
  );
}
```

- [ ] **Step 2: Create robot.json**

```json
{
  "name": "robot",
  "sprite": "/pets/robot.png",
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

- [ ] **Step 3: Add public/pets directory and copy robot.json**

Create the directory: `mkdir -p public/pets`
Copy robot.json there so it's served at `/pets/robot.json` during dev and production.

Run: `cp src-tauri/pets/robot.json public/pets/robot.json`

- [ ] **Step 4: Verify TypeScript compilation**

Run: `npx tsc --noEmit 2>&1 | head -20`
Expected: No errors.

---

### Task 8: Placeholder sprite image

**Files:**
- Create: `public/pets/robot.png` (placeholder for development)

- [ ] **Step 1: Generate minimal placeholder PNG**

Run this Python one-liner to create a 2304×128 transparent PNG with colored frame regions:

```bash
python3 -c "
import struct, zlib

width, height = 2304, 128
# Each frame is 128x128 pixels. 18 frames in a horizontal strip.
colors = [
    (148,163,184), (148,163,184), (148,163,184), (148,163,184),  # idle: gray
    (251,191,36),  (251,191,36),  (251,191,36),  (251,191,36),   # thinking: yellow
    (96,165,250),  (96,165,250),  (96,165,250),  (96,165,250),   # processing: blue
    (163,230,53),  (163,230,53),  (163,230,53),  (163,230,53),   # waiting: green
    (107,114,128), (107,114,128),                                 # sleeping: dark gray
]

def make_chunk(chunk_type, data):
    c = chunk_type + data
    return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)

raw = b''
for frame in range(18):
    r, g, b = colors[frame]
    pixel_row = bytes([r, g, b, 255]) * 128  # 128 identical pixels
    for _ in range(128):                      # 128 rows per frame
        raw += pixel_row

compressed = zlib.compress(raw)

png = b'\x89PNG\r\n\x1a\n'
png += make_chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
png += make_chunk(b'IDAT', compressed)
png += make_chunk(b'IEND', b'')

with open('public/pets/robot.png', 'wb') as f:
    f.write(png)
print(f'Created robot.png ({len(png)} bytes)')
"
```

Run from the project root. Expected: `Created robot.png (xxxx bytes)`

- [ ] **Step 2: Verify the file**

Run: `ls -la public/pets/robot.png`
Expected: File exists with non-zero size.

Note: This placeholder lets us develop and test the animation system. Generate the real pixel-art sprite using the prompt template in `docs/pet-sprite-prompt.md`.

---

### Task 9: Copy robot.json to public/pets

- [ ] **Step 1: Create directory and copy config**

```bash
mkdir -p public/pets
cp src-tauri/pets/robot.json public/pets/robot.json
```

---

### Task 10: Tests for computeAggregate

**Files:**
- Create: `src/pet/usePetState.test.ts`

- [ ] **Step 1: Write the tests**

```typescript
import { describe, it, expect } from 'vitest';
import { computeAggregate } from './usePetState';

describe('computeAggregate', () => {
  it('returns sleeping when no sessions', () => {
    expect(computeAggregate([])).toBe('sleeping');
  });

  it('picks thinking as highest priority', () => {
    const sessions = [
      { status: 'idle' },
      { status: 'thinking' },
      { status: 'waiting' },
    ];
    expect(computeAggregate(sessions)).toBe('thinking');
  });

  it('picks processing when thinking not present', () => {
    const sessions = [
      { status: 'idle' },
      { status: 'processing' },
      { status: 'waiting' },
    ];
    expect(computeAggregate(sessions)).toBe('processing');
  });

  it('maps compacting to processing', () => {
    const sessions = [{ status: 'compacting' }];
    expect(computeAggregate(sessions)).toBe('processing');
  });

  it('returns idle for idle-only sessions', () => {
    const sessions = [{ status: 'idle' }, { status: 'idle' }];
    expect(computeAggregate(sessions)).toBe('idle');
  });

  it('handles unknown status gracefully', () => {
    const sessions = [{ status: 'unknown' }, { status: 'idle' }];
    expect(computeAggregate(sessions)).toBe('idle');
  });
});
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `npx vitest run src/pet/usePetState.test.ts 2>&1`
Expected: 6 tests pass.

---

### Task 11: Tests for config.ts

**Files:**
- Create: `src/pet/config.test.ts`

- [ ] **Step 1: Write the tests**

```typescript
import { describe, it, expect } from 'vitest';
import { loadCharacterConfig } from './config';

describe('loadCharacterConfig', () => {
  it('returns robot config by default', () => {
    const config = loadCharacterConfig('robot');
    expect(config.name).toBe('robot');
    expect(config.frameWidth).toBe(128);
    expect(config.frameHeight).toBe(128);
    expect(config.frameRate).toBe(500);
  });

  it('returns config with all expected states', () => {
    const config = loadCharacterConfig('anything');
    expect(config.states.idle.count).toBe(4);
    expect(config.states.thinking.count).toBe(4);
    expect(config.states.processing.count).toBe(4);
    expect(config.states.waiting.count).toBe(4);
    expect(config.states.sleeping.count).toBe(2);
  });

  it('has valid defaultState', () => {
    const config = loadCharacterConfig('robot');
    expect(config.states[config.defaultState]).toBeDefined();
  });

  it('state startFrame values do not overlap', () => {
    const config = loadCharacterConfig('robot');
    const ranges = Object.values(config.states).map(
      s => [s.startFrame, s.startFrame + s.count] as const
    );
    for (let i = 0; i < ranges.length; i++) {
      for (let j = i + 1; j < ranges.length; j++) {
        const [aStart, aEnd] = ranges[i];
        const [bStart] = ranges[j];
        expect(aEnd).toBeLessThanOrEqual(bStart);
      }
    }
  });
});
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `npx vitest run src/pet/config.test.ts 2>&1`
Expected: 4 tests pass.

---

### Task 12: Full build and verify (build + tests)

**Files:** None (verification only)

- [ ] **Step 1: Full TypeScript check**

Run: `npx tsc --noEmit 2>&1`
Expected: No errors.

- [ ] **Step 2: Vite build**

Run: `npx vite build 2>&1`
Expected: Build succeeds with both `index.html` and `pet.html` in dist.

- [ ] **Step 3: Rust cargo check**

Run: `cargo check -p tauri-temp 2>&1`
Expected: Compilation success.

- [ ] **Step 4: Frontend tests**

Run: `npx vitest run 2>&1`
Expected: All tests pass (10 tests from Tasks 10-11).

- [ ] **Step 5: Rust tests**

Run: `cargo test -p tauri-temp 2>&1`
Expected: All existing tests pass.

---

### Task 13: Commit all changes

```bash
git add src-tauri/src/desktop_pet.rs \
        src-tauri/src/lib.rs \
        src/pet/ \
        pet.html \
        src-tauri/pets/robot.json \
        public/pets/ \
        vite.config.ts
git commit -m "feat: add desktop pet floating window with pixel-art robot

- Create frameless transparent pet window via Tauri WebviewWindowBuilder
- Robot sprite animation system with CSS background-position cycling
- Poll get_all_sessions independently, compute aggregate status
- Character config JSON format for future extensibility (cat, dog, etc.)
- Placeholder sprite sheet for development; real sprite via prompt template"
```

---

### Out of Scope (future tasks)

- Settings UI toggle for pet visibility
- Character selection dropdown
- Right-click context menu (hide/show)
- Pet click interactions
- Position persistence across sessions
- Real pixel-art robot sprite sheet (use prompt template in `docs/pet-sprite-prompt.md`)
