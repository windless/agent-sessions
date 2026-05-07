# Pet Sprite Generation Prompt

Use this template with AI image generation tools (DALL-E, Midjourney, Stable Diffusion, etc.) to generate pixel-art sprite sheets compatible with the desktop pet system.

## Sprite Sheet Specification

- **Canvas size**: `(frameWidth × totalFrames) × frameHeight` pixels
- **Example for robot**: 18 frames × 128px = 2304×128 pixels
- **Frame width**: 128px
- **Frame height**: 128px
- **Layout**: Horizontal strip, frames arranged left-to-right in order
- **Background**: Fully transparent
- **Style**: Pixel art, hard edges, no anti-aliasing
- **Color palette**: Limited palette (16–32 colors), consistent across all frames
- **Scale**: The character should fill ~60-70% of each frame, centered

## Frame Order (18 frames total)

| Frames | State | Description |
|--------|-------|-------------|
| 0–3 | idle | Standing still with subtle movement: blinking eyes, slight antenna bob, gentle breathing |
| 4–7 | thinking | Deep in thought: head tilted, gears turning above head, hand on chin, occasional spark |
| 8–11 | processing | Actively working: rapid typing motion, screen flicker on chest panel, keyboard clicks |
| 12–15 | waiting | Alert but paused: looking left/right, tapping foot, checking watch, ready to spring into action |
| 16–17 | sleeping | Asleep: eyes closed, slow breathing, tiny "Z" floating up, powered-down posture |

## Prompt Template

```
A pixel-art sprite sheet for a cute tiny robot character. The sprite sheet is 2304 pixels wide and 128 pixels tall, divided into 18 frames (each 128x128 pixels) arranged horizontally in a single row, with no gaps between frames. Transparent background.

The robot is small, chibi-style, with a round head, large expressive LED eyes, small antenna on top, and a compact body with a chest screen/panel. Limited 24-color palette, hard pixel edges, no anti-aliasing, no gradients. The character fills about 60% of each frame, centered.

Frame order from left to right (128px each):
- Frames 0-3 (idle): Robot standing, blinking eyes, antenna bobbing slightly, gentle idle breathing
- Frames 4-7 (thinking): Head tilted to side, one hand on chin, small gear icons turning above head, thinking expression
- Frames 8-11 (processing): Hands typing rapidly on an invisible keyboard, chest screen flickering with code/text patterns, focused expression
- Frames 12-15 (waiting): Looking left and right, foot tapping, arm checking an invisible wristwatch, alert but resting expression
- Frames 16-17 (sleeping): Eyes closed, head drooped slightly, small "Z" floating above, powered-down dimmed screen, calm sleeping expression

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
