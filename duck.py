from PIL import Image

def create_rgba_png(filename, pixels):
    """Creates a 16x16 PNG with Alpha Channel"""
    img = Image.new('RGBA', (16, 16))
    for y in range(16):
        for x in range(16):
            img.putpixel((x, y), pixels[y][x])
    img.save(filename, 'PNG')

# Colors (RGBA)
TR = (0, 0, 0, 0)        # Transparent
GR = (34, 139, 34, 255)   # Green Head
W  = (255, 255, 255, 255) # White Neck Ring
BN = (139, 69, 19, 255)   # Brown Chest
GY = (160, 160, 160, 255) # Grey Body
OR = (255, 140, 0, 255)   # Orange Beak/Feet
BK = (0, 0, 0, 255)       # Eye
LN = (80, 80, 80, 255)    # Ground Line
SM = (210, 210, 230, 180) # Smoke
ST = (255, 215, 0, 255)   # Star

def get_canvas(): return [[TR for _ in range(16)] for _ in range(16)]

def draw_line(p, mode):
    if mode == "half":
        for x in range(4, 12): p[14][x] = LN
    elif mode == "full":
        for x in range(0, 16): p[14][x] = LN

def draw_duck(p, y_off=0, pose="normal"):
    """Draws a large mallard duck. y_off shifts it up/down."""
    hx, hy = 6, 2 + y_off # Head Pos
    # Head 4x4
    for y in range(hy, hy+4):
        for x in range(hx, hx+4): 
            if 0 <= y < 16: p[y][x] = GR
    if 0 <= hy+1 < 16: p[hy+1][hx+3] = BK # Eye
    if 0 <= hy+2 < 16: 
        for x in range(hx-3, hx): p[hy+2][x] = OR # Big Beak
    # Neck Ring
    if 0 <= hy+4 < 16:
        for x in range(hx, hx+4): p[hy+4][x] = W
    # Body
    by = hy + 5
    for y in range(by, by+4):
        for x in range(hx-2, hx+7):
            if 0 <= y < 16:
                p[y][x] = BN if x < hx+1 else GY
    # Feet
    fy = by + 4
    if pose == "normal" and 0 <= fy < 16:
        p[fy][hx] = p[fy][hx+4] = OR
    elif pose == "run1" and 0 <= fy < 16:
        p[fy][hx-1] = OR; p[fy-1][hx+3] = OR
    elif pose == "run2" and 0 <= fy < 16:
        p[fy-1][hx] = OR; p[fy][hx+5] = OR

# --- Build 11 Frames ---
frames = []

# 1. Line Half Width
f1 = get_canvas(); draw_line(f1, "half"); frames.append(f1)

# 2. Duck Head Full Width
f2 = get_canvas(); draw_line(f2, "full")
for y in range(11, 14): # Peeking top of head
    for x in range(7, 11): f2[y][x] = GR
f2[12][10] = BK; f2[13][4:7] = [OR]*3
frames.append(f2)

# 3. Duck in Air + Line Half Width
f3 = get_canvas(); draw_line(f3, "half")
draw_duck(f3, y_off=-2); frames.append(f3)

# 4. Duck Land + LINE GONE
f4 = get_canvas(); draw_duck(f4, y_off=1); frames.append(f4)

# 5. Get Ready (Crouch)
f5 = get_canvas(); draw_duck(f5, y_off=2); frames.append(f5)

# 6. Start Run
f6 = get_canvas(); draw_duck(f6, y_off=1, pose="run1"); frames.append(f6)

# 7. Run Frame 1 (Center)
f7 = get_canvas(); draw_duck(f7, y_off=0, pose="run2"); frames.append(f7)

# 8. Run Frame 2 (Center)
f8 = get_canvas(); draw_duck(f8, y_off=1, pose="run1"); frames.append(f8)

# 9. Pouf 1
f9 = get_canvas()
for y in range(6, 12):
    for x in range(5, 11): f9[y][x] = SM
frames.append(f9)

# 10. Pouf 2
f10 = get_canvas()
for y in range(3, 13):
    for x in range(2, 14): f10[y][x] = SM
frames.append(f10)

# 11. Pouf 3 with stars
f11 = get_canvas()
f11[4][4]=f11[2][12]=f11[10][13]=ST # Stars
f11[7][8]=f11[8][7]=SM # Fading smoke
frames.append(f11)

# Save
for i, frame in enumerate(frames):
    create_rgba_png(f"duck_anim_{i+1:02d}.png", frame)

print("Created 11 files: duck_anim_01.png to duck_anim_11.png")