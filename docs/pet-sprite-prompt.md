# Pet Sprite Generation Prompt

Use this template with AI image generation tools (DALL-E, Midjourney, Stable Diffusion, etc.) to generate pixel-art sprite sheets compatible with the desktop pet system.

## Sprite Sheet Specification

- **Canvas size**: columns × rows, each cell `frameWidth × frameHeight`
- **Example for robot**: 6 columns × 3 rows = 768×384 pixels (18 frames of 128×128)
- **Frame width**: 128px, **Frame height**: 128px
- **Layout**: 2D grid, 6 columns × 3 rows. Frames arranged left-to-right, top-to-bottom in order
- **Background**: Fully transparent
- **Style**: Pixel art, hard edges, no anti-aliasing
- **Color palette**: Limited palette (16–32 colors), consistent across all frames
- **Scale**: The character should fill ~60-70% of each frame, centered

## Frame Order (18 frames total, 6 columns × 3 rows)

```
┌────────┬────────┬────────┬────────┬────────┬────────┐
│ idle_0 │ idle_1 │ idle_2 │ idle_3 │think_0 │think_1 │  ← row 0
├────────┼────────┼────────┼────────┼────────┼────────┤
│think_2 │think_3 │ proc_0 │ proc_1 │ proc_2 │ proc_3 │  ← row 1
├────────┼────────┼────────┼────────┼────────┼────────┤
│ wait_0 │ wait_1 │ wait_2 │ wait_3 │sleep_0 │sleep_1 │  ← row 2
└────────┴────────┴────────┴────────┴────────┴────────┘
```

| Frames | State | Description |
|--------|-------|-------------|
| 0–3 | idle | Standing still with subtle movement: blinking eyes, slight antenna bob, gentle breathing |
| 4–7 | thinking | Deep in thought: head tilted, gears turning above head, hand on chin, occasional spark |
| 8–11 | processing | Actively working: rapid typing motion, screen flicker on chest panel, keyboard clicks |
| 12–15 | waiting | Alert but paused: looking left/right, tapping foot, checking watch, ready to spring into action |
| 16–17 | sleeping | Asleep: eyes closed, slow breathing, tiny "Z" floating up, powered-down posture |

## Prompt Template

```
A pixel-art sprite sheet for a cute tiny robot character. The sprite sheet is 768 pixels wide and 384 pixels tall, divided into 18 frames (each 128x128 pixels) arranged in a 6-column by 3-row grid, with no gaps between frames. Transparent background.

Grid layout (6 columns, 3 rows, left-to-right top-to-bottom):
Row 0: idle_0, idle_1, idle_2, idle_3, thinking_0, thinking_1
Row 1: thinking_2, thinking_3, processing_0, processing_1, processing_2, processing_3
Row 2: waiting_0, waiting_1, waiting_2, waiting_3, sleeping_0, sleeping_1

The robot is small, chibi-style, with a round head, large expressive LED eyes, small antenna on top, and a compact body with a chest screen/panel. Limited 24-color palette, hard pixel edges, no anti-aliasing, no gradients. The character fills about 60% of each frame, centered.

Frame descriptions:
- idle (row 0 cols 0-3): Robot standing, blinking eyes, antenna bobbing slightly, gentle idle breathing
- thinking (row 0 cols 4-5, row 1 cols 0-1): Head tilted to side, one hand on chin, small gear icons turning above head, thinking expression
- processing (row 1 cols 2-5): Hands typing rapidly on an invisible keyboard, chest screen flickering with code/text patterns, focused expression
- waiting (row 2 cols 0-3): Looking left and right, foot tapping, arm checking an invisible wristwatch, alert but resting expression
- sleeping (row 2 cols 4-5): Eyes closed, head drooped slightly, small "Z" floating above, powered-down dimmed screen, calm sleeping expression

Consistent character design across all frames. All frames share the same background color level so it can be made transparent. Clean pixel art with clear silhouettes. Retro game sprite style, like a Game Boy Advance or SNES character sprite.
```

## Per-Character Variations

Replace the character description in the prompt:

### Robot
> The robot is small, chibi-style, with a round head, large expressive LED eyes, small antenna on top, and a compact body with a chest screen/panel.

### Cat
> A chubby pixel-art cat, round face, big expressive eyes, small pointed ears, curled tail visible behind. Orange tabby with white chest.

### Dog
> A pixel-art puppy, floppy ears, big expressive eyes, short wagging tail. Golden retriever colors, white patch on chest.
