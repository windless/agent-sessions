# Pet Sprite Generation Prompt

Use this template with AI image generation tools (DALL-E, Midjourney, Stable Diffusion, etc.) to generate pixel-art sprite sheets compatible with the desktop pet system. Replace `[character]` with a description of the desired character.

## Sprite Sheet Specification

- **Canvas size**: 512 × 640 pixels (4 columns × 5 rows)
- **Frame size**: 128 × 128 pixels per cell (18 frames total)
- **Layout**: 2D grid, left-to-right then top-to-bottom, no gaps between frames
- **Background**: Fully transparent
- **Style**: Pixel art, hard edges, no anti-aliasing, no gradients
- **Color palette**: Limited palette (16–32 colors), consistent across all frames
- **Scale**: Character fills ~60-70% of each frame, centered

## Frame Layout (4 columns × 5 rows)

```
┌────────┬────────┬────────┬────────┐
│ idle_0 │ idle_1 │ idle_2 │ idle_3 │  ← row 0
├────────┼────────┼────────┼────────┤
│think_0 │think_1 │think_2 │think_3 │  ← row 1
├────────┼────────┼────────┼────────┤
│ proc_0 │ proc_1 │ proc_2 │ proc_3 │  ← row 2
├────────┼────────┼────────┼────────┤
│ wait_0 │ wait_1 │ wait_2 │ wait_3 │  ← row 3
├────────┼────────┼────────┼────────┤
│sleep_0 │sleep_1 │        │        │  ← row 4
└────────┴────────┴────────┴────────┘
```

| Frames | State | Animation |
|--------|-------|-----------|
| 0–3 | idle | 4-frame subtle idle loop: blinking, gentle breathing, minimal movement |
| 4–7 | thinking | 4-frame deep thought: head tilt, pensive gesture, small visual indicator of mental activity |
| 8–11 | processing | 4-frame active work: rapid repetitive motion, focused/working expression |
| 12–15 | waiting | 4-frame alert rest: looking around, light fidgeting, ready-but-paused body language |
| 16–17 | sleeping | 2-frame sleep: eyes closed, slow breathing, restful/ powered-down posture |

## Prompt Template

```
A pixel-art sprite sheet of [character]. The sprite sheet is exactly 512 pixels wide and 640 pixels tall, divided into 18 frames arranged in a 4-column by 5-row grid. Each frame is exactly 128×128 pixels with no gaps between frames. Fully transparent background.

Grid layout (4 columns × 5 rows, left-to-right then top-to-bottom):
Row 0: idle_0, idle_1, idle_2, idle_3
Row 1: thinking_0, thinking_1, thinking_2, thinking_3
Row 2: processing_0, processing_1, processing_2, processing_3
Row 3: waiting_0, waiting_1, waiting_2, waiting_3
Row 4: sleeping_0, sleeping_1

Frame descriptions:
- idle (row 0 cols 0-3): 4-frame subtle idle loop — blinking, gentle breathing, minimal movement
- thinking (row 1 cols 0-3): 4-frame deep thought — head tilted, pensive gesture, small visual indicator of mental activity
- processing (row 2 cols 0-3): 4-frame active work — rapid repetitive motion, focused expression
- waiting (row 3 cols 0-3): 4-frame alert rest — looking around, light fidgeting, ready-but-paused body language
- sleeping (row 4 cols 0-1): 2-frame sleep — eyes closed, slow breathing, restful posture

Technical requirements:
- Pixel art style with hard pixel edges, no anti-aliasing, no gradients
- Limited color palette of 16–32 colors, consistent across all frames
- The character fills about 60–70% of each 128×128 frame, centered
- All frames share identical transparent background
- Retro game sprite aesthetic, like Game Boy Advance or SNES sprites
- Character design and colors remain 100% consistent across all 18 frames
```
