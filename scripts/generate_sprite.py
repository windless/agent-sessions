"""Generate a compliant pixel-art robot sprite sheet for testing.

Spec:
  - Canvas: 512 × 640 pixels (4 columns × 5 rows)
  - Frame: 128 × 128 pixels each, 18 frames total
  - Character fills ~60-70% of each frame, centered
  - Transparent background, hard pixel edges, limited palette (16-32 colors)
"""

from PIL import Image

W, H = 512, 640
FW, FH = 128, 128
TARGET_FILL = 0.65  # Character should fill 65% of frame area


def new_frame(w=FW, h=FH) -> Image.Image:
    return Image.new("RGBA", (w, h), (0, 0, 0, 0))


def put_pixel(img: Image.Image, x: int, y: int, color: tuple):
    if 0 <= x < img.width and 0 <= y < img.height:
        img.putpixel((x, y), color)


# Use a 2x internal canvas (256x256) for more drawing precision,
# then NEAREST-downscale to 128x128 to preserve crisp pixel edges.
IW = 256  # internal canvas size


def scale_to_frame(frames: list[Image.Image]) -> list[Image.Image]:
    """Scale each frame so the character fills TARGET_FILL of the 128x128 frame."""
    result = []
    for img in frames:
        # Find bounding box of non-transparent pixels
        bbox = img.getbbox()
        if bbox is None:
            result.append(img.resize((FW, FH), Image.NEAREST))
            continue
        x1, y1, x2, y2 = bbox
        char_w, char_h = x2 - x1, y2 - y1
        # Target: character area = TARGET_FILL of frame area
        # target_w * target_h = TARGET_FILL * FW * FH
        # Scale factor: sqrt(TARGET_FILL * FW * FH / (char_w * char_h))
        current_area = char_w * char_h
        target_area = TARGET_FILL * FW * FH
        scale = (target_area / current_area) ** 0.5
        # Cap scale to avoid clipping
        scale = min(scale, FW / char_w, FH / char_h)

        new_w = int(char_w * scale)
        new_h = int(char_h * scale)
        # Crop to character, scale up with NEAREST for pixel-art crispness
        char_img = img.crop(bbox)
        scaled = char_img.resize((new_w, new_h), Image.NEAREST)
        # Center in 128x128 frame
        out = new_frame(FW, FH)
        ox = (FW - new_w) // 2
        oy = (FH - new_h) // 2
        out.paste(scaled, (ox, oy), scaled)
        result.append(out)
    return result


def draw_robot_base(internal: bool = True) -> Image.Image:
    """Draw the base robot body on the internal canvas (256x256).
    All coordinates scaled for 256x256 canvas."""
    c = {
        "outline": (40, 40, 50, 255),
        "body": (100, 120, 160, 255),
        "eye": (255, 255, 255, 255),
        "accent": (255, 180, 50, 255),
        "arm": (80, 100, 140, 255),
        "leg": (70, 85, 120, 255),
        "think_light": (180, 160, 80, 255),
        "proc_light": (80, 220, 120, 255),
        "proc_eye": (100, 255, 255, 255),
        "wait_light": (255, 220, 60, 255),
        "sleep_body": (70, 85, 115, 255),
        "sleep_light": (50, 55, 65, 255),
    }
    s = 2.0  # scale factor for internal canvas
    img = new_frame(IW, IW)
    _draw_robot_body_scaled(img, c, s)
    return img


def _draw_robot_body_scaled(img: Image.Image, c: dict, s: float = 2.0):
    """Draw robot body with coordinate scaling. s=2.0 means 256x256 canvas."""
    def x(v): return int(v * s)
    def y(v): return int(v * s)

    # Head
    for px in range(x(44), x(84)):
        put_pixel(img, px, y(20), c["outline"])
        put_pixel(img, px, y(44), c["outline"])
    for py in range(y(20), y(45)):
        put_pixel(img, x(44), py, c["outline"])
        put_pixel(img, x(83), py, c["outline"])
    for px in range(x(45), x(83)):
        for py in range(y(21), y(44)):
            put_pixel(img, px, py, c["body"])

    # Eyes
    for px in range(x(53), x(61)):
        for py in range(y(28), y(34)):
            put_pixel(img, px, py, c["eye"])
    for px in range(x(67), x(75)):
        for py in range(y(28), y(34)):
            put_pixel(img, px, py, c["eye"])

    # Mouth
    for px in range(x(56), x(72)):
        put_pixel(img, px, y(38), c["outline"])

    # Antenna
    for py in range(y(10), y(20)):
        put_pixel(img, x(63), py, c["outline"])
        put_pixel(img, x(64), py, c["outline"])
    for px in range(x(61), x(67)):
        put_pixel(img, px, y(10), c["accent"])
        put_pixel(img, px, y(11), c["accent"])

    # Body (torso)
    for px in range(x(42), x(86)):
        put_pixel(img, px, y(45), c["outline"])
        put_pixel(img, px, y(75), c["outline"])
    for py in range(y(45), y(76)):
        put_pixel(img, x(42), py, c["outline"])
        put_pixel(img, x(85), py, c["outline"])
    for px in range(x(43), x(85)):
        for py in range(y(46), y(75)):
            put_pixel(img, px, py, c["body"])

    # Chest light
    for px in range(x(59), x(69)):
        for py in range(y(52), y(60)):
            put_pixel(img, px, py, c["accent"])

    # Arms
    for px in range(x(34), x(42)):
        for py in range(y(48), y(68)):
            put_pixel(img, px, py, c["arm"])
    for px in range(x(34), x(43)):
        put_pixel(img, px, y(47), c["outline"])
        put_pixel(img, px, y(68), c["outline"])
    for py in range(y(47), y(69)):
        put_pixel(img, x(34), py, c["outline"])
    for px in range(x(30), x(35)):
        for py in range(y(64), y(72)):
            put_pixel(img, px, py, c["accent"])

    for px in range(x(86), x(94)):
        for py in range(y(48), y(68)):
            put_pixel(img, px, py, c["arm"])
    for px in range(x(85), x(95)):
        put_pixel(img, px, y(47), c["outline"])
        put_pixel(img, px, y(68), c["outline"])
    for py in range(y(47), y(69)):
        put_pixel(img, x(94), py, c["outline"])
    for px in range(x(93), x(98)):
        for py in range(y(64), y(72)):
            put_pixel(img, px, py, c["accent"])

    # Legs
    for lx_base in [x(46), x(68)]:
        for px in range(lx_base, lx_base + x(14)):
            for py in range(y(76), y(100)):
                put_pixel(img, px, py, c["leg"])
        for px in range(lx_base - x(1), lx_base + x(15)):
            put_pixel(img, px, y(75), c["outline"])
            put_pixel(img, px, y(100), c["outline"])
        for py in range(y(75), y(101)):
            put_pixel(img, lx_base - x(1), py, c["outline"])
            put_pixel(img, lx_base + x(14), py, c["outline"])
        for px in range(lx_base - x(3), lx_base + x(17)):
            for py in range(y(100), y(106)):
                put_pixel(img, px, py, c["outline"])


def draw_idle_body(alt: int = 0) -> Image.Image:
    """Idle frames with subtle variations."""
    c = {
        "outline": (40, 40, 50, 255),
        "body": (100, 120, 160, 255),
        "eye": (255, 255, 255, 255),
        "accent": (255, 180, 50, 255),
        "arm": (80, 100, 140, 255),
        "leg": (70, 85, 120, 255),
    }
    img = new_frame(IW, IW)
    s = 2.0
    def x(v): return int(v * s)
    def y(v): return int(v * s)

    _draw_robot_body_scaled(img, c, s)

    if alt == 1:
        # Slightly shifted arms (micro-movement)
        pass
    elif alt == 2:
        # Blink — closed eyes
        for px in range(x(53), x(61)):
            for py in range(y(29), y(33)):
                put_pixel(img, px, py, c["body"])
        for px in range(x(67), x(75)):
            for py in range(y(29), y(33)):
                put_pixel(img, px, py, c["body"])
        for px in range(x(54), x(60)):
            put_pixel(img, px, y(30), c["outline"])
            put_pixel(img, px, y(31), c["outline"])
        for px in range(x(68), x(74)):
            put_pixel(img, px, y(30), c["outline"])
            put_pixel(img, px, y(31), c["outline"])

    return img


def draw_think_frame(tilt: int = 0) -> Image.Image:
    """Thinking pose — head tilted, arm to chin."""
    c = {
        "outline": (40, 40, 50, 255),
        "body": (100, 120, 160, 255),
        "eye": (255, 255, 255, 255),
        "accent": (255, 180, 50, 255),
        "arm": (80, 100, 140, 255),
        "leg": (70, 85, 120, 255),
        "think_light": (180, 160, 80, 255),
    }
    img = new_frame(IW, IW)
    s = 2.0
    def x(v): return int(v * s)
    def y(v): return int(v * s)

    # Body
    for px in range(x(42), x(86)):
        put_pixel(img, px, y(45), c["outline"])
        put_pixel(img, px, y(75), c["outline"])
    for py in range(y(45), y(76)):
        put_pixel(img, x(42), py, c["outline"])
        put_pixel(img, x(85), py, c["outline"])
    for px in range(x(43), x(85)):
        for py in range(y(46), y(75)):
            put_pixel(img, px, py, c["body"])

    # Chest light (thinking = amber)
    for px in range(x(59), x(69)):
        for py in range(y(52), y(60)):
            put_pixel(img, px, py, c["think_light"])

    # Head with tilt offset
    hx = tilt
    hy = abs(tilt) // 2
    for px in range(x(44), x(84)):
        put_pixel(img, px + hx, y(20) + hy, c["outline"])
        put_pixel(img, px + hx, y(44) + hy, c["outline"])
    for py in range(y(20), y(45)):
        put_pixel(img, x(44) + hx, py + hy, c["outline"])
        put_pixel(img, x(83) + hx, py + hy, c["outline"])
    for px in range(x(45), x(83)):
        for py in range(y(21), y(44)):
            put_pixel(img, px + hx, py + hy, c["body"])

    # Squinting eyes
    for px in range(x(54), x(60)):
        for py in range(y(28), y(32)):
            put_pixel(img, px + hx, py + hy, c["eye"])
    for px in range(x(68), x(74)):
        for py in range(y(28), y(32)):
            put_pixel(img, px + hx, py + hy, c["eye"])

    # Mouth
    for px in range(x(58), x(70)):
        put_pixel(img, px + hx, y(39) + hy, c["outline"])

    # Antenna (with tilt)
    for py in range(y(10), y(20)):
        put_pixel(img, x(63) + hx, py + hy, c["outline"])
        put_pixel(img, x(64) + hx, py + hy, c["outline"])
    for px in range(x(61), x(67)):
        put_pixel(img, px + hx, y(10) + hy, c["think_light"])
        put_pixel(img, px + hx, y(11) + hy, c["think_light"])

    # Right arm raised to chin
    for py in range(y(38), y(50)):
        put_pixel(img, x(86), py, c["arm"])
        put_pixel(img, x(87), py, c["arm"])
    for px in range(x(84), x(90)):
        put_pixel(img, px, y(37), c["outline"])
    for px in range(x(83), x(89)):
        for py in range(y(36), y(40)):
            put_pixel(img, px, py, c["accent"])

    # Left arm at side
    for px in range(x(34), x(42)):
        for py in range(y(48), y(68)):
            put_pixel(img, px, py, c["arm"])
    for px in range(x(34), x(43)):
        put_pixel(img, px, y(47), c["outline"])
        put_pixel(img, px, y(68), c["outline"])
    for py in range(y(47), y(69)):
        put_pixel(img, x(34), py, c["outline"])
    for px in range(x(30), x(35)):
        for py in range(y(64), y(72)):
            put_pixel(img, px, py, c["accent"])

    # Legs
    for lx_base in [x(46), x(68)]:
        for px in range(lx_base, lx_base + x(14)):
            for py in range(y(76), y(100)):
                put_pixel(img, px, py, c["leg"])
        for px in range(lx_base - x(1), lx_base + x(15)):
            put_pixel(img, px, y(75), c["outline"])
            put_pixel(img, px, y(100), c["outline"])
        for py in range(y(75), y(101)):
            put_pixel(img, lx_base - x(1), py, c["outline"])
            put_pixel(img, lx_base + x(14), py, c["outline"])
        for px in range(lx_base - x(3), lx_base + x(17)):
            for py in range(y(100), y(106)):
                put_pixel(img, px, py, c["outline"])

    return img


def draw_process_frame(arm_angle: int = 0) -> Image.Image:
    """Processing — active work, arms moving."""
    c = {
        "outline": (40, 40, 50, 255),
        "body": (100, 120, 160, 255),
        "eye": (255, 255, 255, 255),
        "accent": (255, 180, 50, 255),
        "arm": (80, 100, 140, 255),
        "leg": (70, 85, 120, 255),
        "proc_light": (80, 220, 120, 255),
        "proc_eye": (100, 255, 255, 255),
    }
    img = new_frame(IW, IW)
    s = 2.0
    def x(v): return int(v * s)
    def y(v): return int(v * s)

    # Body
    for px in range(x(42), x(86)):
        put_pixel(img, px, y(45), c["outline"])
        put_pixel(img, px, y(75), c["outline"])
    for py in range(y(45), y(76)):
        put_pixel(img, x(42), py, c["outline"])
        put_pixel(img, x(85), py, c["outline"])
    for px in range(x(43), x(85)):
        for py in range(y(46), y(75)):
            put_pixel(img, px, py, c["body"])

    # Chest light (green = active)
    for px in range(x(59), x(69)):
        for py in range(y(52), y(60)):
            put_pixel(img, px, py, c["proc_light"])

    # Head
    for px in range(x(44), x(84)):
        put_pixel(img, px, y(20), c["outline"])
        put_pixel(img, px, y(44), c["outline"])
    for py in range(y(20), y(45)):
        put_pixel(img, x(44), py, c["outline"])
        put_pixel(img, x(83), py, c["outline"])
    for px in range(x(45), x(83)):
        for py in range(y(21), y(44)):
            put_pixel(img, px, py, c["body"])

    # Focused eyes (cyan)
    for px in range(x(54), x(61)):
        for py in range(y(28), y(32)):
            put_pixel(img, px, py, c["proc_eye"])
    for px in range(x(67), x(74)):
        for py in range(y(28), y(32)):
            put_pixel(img, px, py, c["proc_eye"])

    # Determined mouth
    for px in range(x(56), x(72)):
        put_pixel(img, px, y(38), c["outline"])

    # Antenna (bright)
    for py in range(y(10), y(20)):
        put_pixel(img, x(63), py, c["outline"])
        put_pixel(img, x(64), py, c["outline"])
    for px in range(x(61), x(67)):
        put_pixel(img, px, y(10), c["proc_light"])
        put_pixel(img, px, y(11), c["proc_light"])

    # Arms vary by angle
    arm_configs = [
        (x(30), x(38), y(48), y(72), x(90), x(98), y(48), y(72), "down"),
        (x(30), x(38), y(30), y(55), x(90), x(98), y(30), y(55), "up"),
        (x(24), x(36), y(50), y(58), x(92), x(104), y(50), y(58), "side"),
        (x(38), x(48), y(52), y(62), x(80), x(90), y(52), y(62), "cross"),
    ]
    la_x1, la_x2, la_y1, la_y2, ra_x1, ra_x2, ra_y1, ra_y2, _ = arm_configs[arm_angle]
    for px in range(la_x1, la_x2):
        for py in range(la_y1, la_y2):
            put_pixel(img, px, py, c["arm"])
    for px in range(ra_x1, ra_x2):
        for py in range(ra_y1, ra_y2):
            put_pixel(img, px, py, c["arm"])

    # Hands
    if arm_angle == 0:
        for px in range(x(26), x(31)):
            for py in range(y(68), y(76)):
                put_pixel(img, px, py, c["accent"])
        for px in range(x(97), x(102)):
            for py in range(y(68), y(76)):
                put_pixel(img, px, py, c["accent"])
    elif arm_angle == 1:
        for px in range(x(27), x(32)):
            for py in range(y(26), y(32)):
                put_pixel(img, px, py, c["accent"])
        for px in range(x(96), x(101)):
            for py in range(y(26), y(32)):
                put_pixel(img, px, py, c["accent"])

    # Legs
    for lx_base in [x(46), x(68)]:
        for px in range(lx_base, lx_base + x(14)):
            for py in range(y(76), y(100)):
                put_pixel(img, px, py, c["leg"])
        for px in range(lx_base - x(1), lx_base + x(15)):
            put_pixel(img, px, y(75), c["outline"])
            put_pixel(img, px, y(100), c["outline"])
        for py in range(y(75), y(101)):
            put_pixel(img, lx_base - x(1), py, c["outline"])
            put_pixel(img, lx_base + x(14), py, c["outline"])
        for px in range(lx_base - x(3), lx_base + x(17)):
            for py in range(y(100), y(106)):
                put_pixel(img, px, py, c["outline"])

    return img


def draw_wait_frame(look: int = 0) -> Image.Image:
    """Waiting — looking around, alert."""
    c = {
        "outline": (40, 40, 50, 255),
        "body": (100, 120, 160, 255),
        "eye": (255, 255, 255, 255),
        "accent": (255, 180, 50, 255),
        "arm": (80, 100, 140, 255),
        "leg": (70, 85, 120, 255),
        "wait_light": (255, 220, 60, 255),
    }
    img = new_frame(IW, IW)
    s = 2.0
    def x(v): return int(v * s)
    def y(v): return int(v * s)

    _draw_robot_body_scaled(img, c, s)

    # Overwrite chest light with wait color
    for px in range(x(59), x(69)):
        for py in range(y(52), y(60)):
            put_pixel(img, px, py, c["wait_light"])

    # Shift eyes for looking direction
    off = look * s  # -4, 0, 4 for left/center/right
    # Erase original eyes
    for px in range(x(53), x(61)):
        for py in range(y(28), y(34)):
            put_pixel(img, px, py, c["body"])
    for px in range(x(67), x(75)):
        for py in range(y(28), y(34)):
            put_pixel(img, px, py, c["body"])
    # Draw shifted
    for px in range(int(x(53) + off), int(x(61) + off)):
        for py in range(y(28), y(34)):
            put_pixel(img, px, py, c["eye"])
    for px in range(int(x(67) + off), int(x(75) + off)):
        for py in range(y(28), y(34)):
            put_pixel(img, px, py, c["eye"])

    return img


def draw_sleep_frame(frame_idx: int = 0) -> Image.Image:
    """Sleeping — powered down, eyes closed."""
    c = {
        "outline": (40, 40, 50, 255),
        "body": (100, 120, 160, 255),
        "eye": (255, 255, 255, 255),
        "accent": (255, 180, 50, 255),
        "arm": (80, 100, 140, 255),
        "leg": (70, 85, 120, 255),
        "sleep_body": (70, 85, 115, 255),
        "sleep_light": (50, 55, 65, 255),
        "wait_light": (255, 220, 60, 255),
    }
    img = new_frame(IW, IW)
    s = 2.0
    def x(v): return int(v * s)
    def y(v): return int(v * s)

    # Dimmed body
    for px in range(x(42), x(86)):
        put_pixel(img, px, y(45), c["outline"])
        put_pixel(img, px, y(75), c["outline"])
    for py in range(y(45), y(76)):
        put_pixel(img, x(42), py, c["outline"])
        put_pixel(img, x(85), py, c["outline"])
    for px in range(x(43), x(85)):
        for py in range(y(46), y(75)):
            put_pixel(img, px, py, c["sleep_body"])

    # Chest light off
    for px in range(x(59), x(69)):
        for py in range(y(52), y(60)):
            put_pixel(img, px, py, c["sleep_light"])

    # Head (dimmed)
    for px in range(x(44), x(84)):
        put_pixel(img, px, y(20), c["outline"])
        put_pixel(img, px, y(44), c["outline"])
    for py in range(y(20), y(45)):
        put_pixel(img, x(44), py, c["outline"])
        put_pixel(img, x(83), py, c["outline"])
    for px in range(x(45), x(83)):
        for py in range(y(21), y(44)):
            put_pixel(img, px, py, c["sleep_body"])

    # Closed eyes
    for px in range(x(54), x(60)):
        put_pixel(img, px, y(30), c["outline"])
        put_pixel(img, px, y(31), c["outline"])
    for px in range(x(68), x(74)):
        put_pixel(img, px, y(30), c["outline"])
        put_pixel(img, px, y(31), c["outline"])

    # Mouth (snoring o)
    for px in range(x(60), x(68)):
        put_pixel(img, px, y(37), c["outline"])
        put_pixel(img, px, y(38), c["outline"])

    # Drooped antenna
    for py in range(y(12), y(20)):
        put_pixel(img, x(62), py, c["outline"])
    for py in range(y(13), y(19)):
        put_pixel(img, x(61), py, c["outline"])
    for px in range(x(59), x(64)):
        put_pixel(img, px, y(12), c["sleep_light"])

    # Arms relaxed
    for px in range(x(36), x(42)):
        for py in range(y(50), y(68)):
            put_pixel(img, px, py, c["arm"])
    for px in range(x(86), x(92)):
        for py in range(y(50), y(68)):
            put_pixel(img, px, py, c["arm"])

    # Legs
    for lx_base in [x(46), x(68)]:
        for px in range(lx_base, lx_base + x(14)):
            for py in range(y(76), y(100)):
                put_pixel(img, px, py, c["leg"])
        for px in range(lx_base - x(1), lx_base + x(15)):
            put_pixel(img, px, y(75), c["outline"])
            put_pixel(img, px, y(100), c["outline"])
        for py in range(y(75), y(101)):
            put_pixel(img, lx_base - x(1), py, c["outline"])
            put_pixel(img, lx_base + x(14), py, c["outline"])
        for px in range(lx_base - x(3), lx_base + x(17)):
            for py in range(y(100), y(106)):
                put_pixel(img, px, py, c["outline"])

    # Zzz (only on second frame, frame_idx=1)
    if frame_idx == 1:
        z_positions = [(x(86), y(18)), (x(92), y(10)), (x(100), y(4))]
        for zx, zy in z_positions:
            for dzx in range(4):
                for dzy in range(4):
                    put_pixel(img, zx + dzx, zy + dzy, c["wait_light"])

    return img


def main():
    # Build frames on internal canvas (256x256)
    raw_frames = [
        # Row 0: idle_0..idle_3, think_0, think_1
        draw_idle_body(0),
        draw_idle_body(1),
        draw_idle_body(2),  # blink
        draw_idle_body(0),
        draw_think_frame(-4),
        draw_think_frame(0),
        # Row 1: think_2, think_3, proc_0..proc_3
        draw_think_frame(4),
        draw_think_frame(0),
        draw_process_frame(0),
        draw_process_frame(1),
        draw_process_frame(2),
        draw_process_frame(3),
        # Row 2: wait_0..wait_3, sleep_0, sleep_1
        draw_wait_frame(-1),
        draw_wait_frame(0),
        draw_wait_frame(1),
        draw_wait_frame(0),
        draw_sleep_frame(0),
        draw_sleep_frame(1),
    ]

    # Scale each frame to fill ~65% of 128x128
    frames = scale_to_frame(raw_frames)

    # Assemble into sprite sheet
    sheet = new_frame(W, H)
    layout = [
        (0, 0), (0, 1), (0, 2), (0, 3),
        (1, 0), (1, 1), (1, 2), (1, 3),
        (2, 0), (2, 1), (2, 2), (2, 3),
        (3, 0), (3, 1), (3, 2), (3, 3),
        (4, 0), (4, 1),
    ]
    for (row, col), frame in zip(layout, frames):
        sheet.paste(frame, (col * FW, row * FH))

    # Ensure no semi-transparent pixels (pixel art = hard edges)
    pixels = sheet.load()
    for y in range(H):
        for x in range(W):
            r, g, b, a = pixels[x, y]
            if 0 < a < 128:
                pixels[x, y] = (r, g, b, 0)
            elif 128 <= a < 255:
                pixels[x, y] = (r, g, b, 255)

    out_path = "public/pets/robot.png"
    sheet.save(out_path, "PNG", optimize=False)
    print(f"Saved {out_path} ({sheet.size[0]}x{sheet.size[1]})")

    # Verify
    verify = Image.open(out_path)
    assert verify.size == (512, 640), f"Wrong size: {verify.size}"
    assert verify.mode in ("RGBA", "RGB"), f"Wrong mode: {verify.mode}"

    # Check per-cell fill
    for row in range(5):
        for col in range(4):
            cell = verify.crop((col * FW, row * FH, (col + 1) * FW, (row + 1) * FH))
            non_alpha = sum(1 for px in range(cell.width) for py in range(cell.height)
                           if cell.getpixel((px, py))[3] > 0)
            pct = non_alpha / (FW * FH) * 100
            print(f"  Cell [{row},{col}]: {non_alpha} px ({pct:.1f}%)")

    # Count semi-transparent
    semi = sum(1 for py in range(H) for px in range(W)
               if 0 < verify.getpixel((px, py))[3] < 255)
    print(f"Semi-transparent pixels: {semi}")
    print("Done.")


if __name__ == "__main__":
    main()
