# User Guide: Adding Custom Sprites

A guide for non-programmers who want to add or modify sprites in the game.

## What You'll Need

- **Image Editor:** GIMP, Photoshop, Aseprite, or any tool that can save PNG files
- **Text Editor:** Notepad++, VS Code, or even Windows Notepad
- **Game Installation:** Know where your game is installed

**No programming knowledge required!**

---

## Quick Start: 5-Minute Tutorial

Let's add a custom sprite for the Frozen Orb missile.

### Step 1: Find the Sprite Folder

Navigate to your game installation folder, then:

```
YourGame/
└── custom/
    └── missiles/
        └── (create your folders here)
```

**If `custom` folder doesn't exist, create it!**

### Step 2: Create Your Sprite Folder

Create a new folder with the EXACT name from the game:

```
custom/
└── missiles/
    └── FrozenOrb/  ← Must match exactly!
```

**⚠️ Important:** Folder names are case-sensitive. `FrozenOrb` works, `frozenorb` doesn't!

### Step 3: Add Your Sprite Images

Two ways to organize sprites:

**Option A: Individual frames (numbered)**
```
FrozenOrb/
├── 0.png
├── 1.png
├── 2.png
├── ...
└── 13.png
```

**Option B: Sprite sheets (per-animation)**
```
FrozenOrb/
├── Stand.png
├── Walk.png
└── Attack.png
```

### Step 4: Create Configuration File

Create a text file named `sprite.json` in your folder:

**For Option A (Individual frames):**
```json
{
  "fps": 15.0
}
```

**For Option B (Sprite sheets):**
```json
{
  "default_fps": 15.0,
  "files": {
    "Stand.png": { "frame_count": 1 },
    "Walk.png": { "frame_count": 8 },
    "Attack.png": { "frame_count": 12 }
  }
}
```

**Save as:** `sprite.json` (not .txt!)

### Step 5: Test in Game

1. Launch the game
2. Find an item that uses Frozen Orb
3. Your custom sprite should appear!

**Troubleshooting:** Check `logs/animation.log` if something doesn't work.

---

## Understanding Sprite Folders

### Where to Put Files

The game searches these folders in order:

1. **custom/** - Your custom sprites (checked first!)
2. **assets/images/** - Default game sprites

**This means:** Put your files in `custom/` to override defaults without modifying game files!

### Folder Structure

```
custom/
├── missiles/
│   ├── FrozenOrb/
│   ├── Lightning/
│   └── Meteor/
├── monsters/
│   ├── Zombie/
│   └── Skeleton/
├── players/
│   └── Barbarian/
└── objects/
    ├── Chest/
    └── Door/
```

### Folder Naming Rules

✅ **Correct:**
- `FrozenOrb`
- `BloodGolem`
- `HolyBolt`

❌ **Wrong:**
- `frozen orb` (spaces not allowed)
- `frozenorb` (wrong capitalization)
- `Frozen-Orb` (hyphens not allowed)

**Tip:** Copy folder names from `assets/images/` to be sure!

---

## Creating Sprite Images

### Image Requirements

**Format:** PNG with transparency
**Size:** Depends on the entity (typically 64x64 to 256x256)
**Color:** RGB + Alpha channel
**Background:** Transparent (not white!)

### Two Layout Options

#### Option 1: Individual Frames (Easiest)

Save each frame as a separate file:

```
0.png  (frame 1)
1.png  (frame 2)
2.png  (frame 3)
...
```

**Pros:**
- Simple to understand
- Easy to edit individual frames
- Good for small animations

**Cons:**
- All animations share same frames
- Many files for big animations

#### Option 2: Sprite Sheet (Advanced)

Multiple frames in one image file:

**Horizontal Strip:**
```
[Frame 0][Frame 1][Frame 2][Frame 3]
```

**Grid (for many frames):**
```
[F0][F1][F2][F3]
[F4][F5][F6][F7]
[F8][F9][F10][F11]
```

**Pros:**
- One file per animation
- Organized by action (walk, attack, etc.)
- Easier to manage

**Cons:**
- Requires image editing skills
- Need to calculate frame positions

---

## Configuration File (sprite.json)

### Basic Template

```json
{
  "fps": 15.0
}
```

**That's it!** The game will auto-detect:
- Number of frames (counts your .png files)
- Layout type (numbered or sprite sheet)

### Common Settings

**Frames Per Second (FPS):**
```json
{
  "fps": 15.0
}
```
- **Lower FPS (10-12):** Slower, more dramatic
- **Normal FPS (15):** Standard speed
- **Higher FPS (20-24):** Fast, frenetic

**Speed Multiplier:**
```json
{
  "fps": 15.0,
  "speed": 1.5
}
```
- `1.0` = normal speed
- `1.5` = 50% faster
- `0.5` = 50% slower

**Grid Layout:**
```json
{
  "frame_count": 64,
  "layout": "grid",
  "grid_columns": 8
}
```
- Use when you have 16+ frames in a sprite sheet
- `grid_columns` tells how many frames per row

### Advanced: Per-Animation Files

Different animations for different actions:

```json
{
  "default_fps": 15.0,
  
  "files": {
    "Stand.png": {
      "frame_count": 1
    },
    "Walk.png": {
      "frame_count": 8,
      "fps": 12.0
    },
    "Attack.png": {
      "frame_count": 12,
      "fps": 24.0
    }
  },
  
  "mode_mapping": {
    "stand": "Stand.png",
    "walk": "Walk.png",
    "attack1": "Attack.png",
    "attack2": "Attack.png"
  }
}
```

**Explanation:**
- `Stand.png` = 1 frame, idle pose
- `Walk.png` = 8 frames at 12 FPS
- `Attack.png` = 12 frames at 24 FPS (faster!)
- `attack2` reuses Attack.png

---

## Using the Sheet Analyzer Tool

Don't want to count frames manually? Use the tool!

### Step 1: Open the Tool

In your browser, open:
```
tools/sheet-analyzer.html
```

### Step 2: Upload Your Sprite Sheet

1. Click "Choose File"
2. Select your sprite sheet image
3. Tool shows the image with dimensions

### Step 3: Configure Layout

**If horizontal strip:**
- Layout: Horizontal
- Frame Count: How many frames across?

**If grid:**
- Layout: Grid
- Frame Count: Total frames
- Grid Columns: How many columns?

### Step 4: Verify with Grid

Green lines show where frames will be split.

**If lines are wrong:**
- Adjust Frame Count
- Try different Layout option

### Step 5: Save Configuration

Click "Save sprite.json"
- Downloads the file
- Move it to your sprite folder

**Done!** Perfect configuration without manual work.

---

## Common Issues & Solutions

### Issue 1: Sprites Don't Appear

**Check these:**

1. **Folder name correct?**
   - Must match exactly: `FrozenOrb`, not `frozen orb`
   - Check `assets/images/` for correct names

2. **Files in right location?**
   ```
   ✅ custom/missiles/FrozenOrb/sprite.json
   ❌ custom/FrozenOrb/sprite.json
   ```

3. **PNG files valid?**
   - Open in image editor to verify
   - Must be actual PNG, not renamed JPG

4. **sprite.json valid?**
   - Use JSONLint.com to check
   - Common error: missing comma or quote

### Issue 2: Wrong Animation Speed

**Too fast?**
```json
{
  "fps": 10.0,    // Lower FPS
  "speed": 0.8    // Slower speed
}
```

**Too slow?**
```json
{
  "fps": 20.0,    // Higher FPS
  "speed": 1.5    // Faster speed
}
```

### Issue 3: Frames in Wrong Order

**Numbered files (0.png, 1.png, etc.):**
- Windows Explorer sorts 1, 10, 11, 2...
- Rename: 00.png, 01.png, 02.png, 10.png, 11.png ✅

**Sprite sheet:**
- Verify frames arranged left-to-right, top-to-bottom
- Use Sheet Analyzer tool to verify

### Issue 4: Animation Stutters

**Possible causes:**
1. **Frames not same size** - All must be exactly same dimensions
2. **FPS too high** - Try 15.0 instead of 30.0
3. **Too many frames** - Use grid layout for 20+ frames
4. **File too large** - Optimize PNGs with TinyPNG.com

### Issue 5: Transparent Background Shows White

**Problem:** PNG has white background, not transparent

**Solution:**
1. Open in GIMP/Photoshop
2. Select white background (magic wand tool)
3. Delete → becomes transparent
4. Export as PNG with "Save transparency"

---

## Tips & Best Practices

### Creating Smooth Animations

**Frame count matters:**
- **1 frame:** Static, no animation
- **4-6 frames:** Choppy, retro feel
- **8-12 frames:** Smooth, standard
- **16+ frames:** Very smooth, professional

**Frame rate (FPS) matters:**
- **8-10 FPS:** Slow, deliberate
- **12-15 FPS:** Natural speed (recommended)
- **18-24 FPS:** Fast-paced action
- **30+ FPS:** Silky smooth (overkill?)

### Optimizing File Size

**Large sprite sheets:**
1. Use PNG optimization tools (TinyPNG, OptiPNG)
2. Reduce unnecessary transparency
3. Use grid layout instead of many small files
4. Consider lower resolution if scaled down in-game

**Many entities:**
- Reuse sprites when possible
- Share animations between similar entities
- Use simpler animations for background entities

### Organizing Your Custom Sprites

**Create a backup:**
```
custom_sprites_backup/
└── (copy of your custom folder)
```

**Document your changes:**
```
custom/
└── README.txt  (list what you modified)
```

**Keep originals:**
```
custom/
└── _originals/
    └── (backup of replaced sprites)
```

---

## Advanced: Creating State-Based Sprites

For UI elements like health bars that change based on value.

### Example: Health Globe

**Create 101 frames (0% to 100%):**

```
HealthGlobe/
├── sprite.json
└── sheet.png  (101 frames showing increasing fill)
```

**sprite.json:**
```json
{
  "frame_count": 101,
  "fps": 1.0,
  "layout": "horizontal",
  "animation_mode": {
    "state_mapping": "percentage"
  }
}
```

**What this does:**
- 0% health → frame 0 (empty)
- 50% health → frame 50 (half)
- 100% health → frame 100 (full)

**Tool:** Use [Globe Generator](../tools/globe-generator.html) to create these automatically!

---

## Getting Help

### Resources

1. **Log Files:**
   - Location: `logs/animation.log`
   - Check for error messages

2. **Example Sprites:**
   - Look in `assets/images/`
   - Copy structure for your custom sprites

3. **Tools:**
   - [Sheet Analyzer](../tools/sheet-analyzer.html) - Figure out configurations
   - [Globe Generator](../tools/globe-generator.html) - Create health/mana globes
   - [Icon Generator](../tools/icon-generator.html) - Make element icons

4. **Community:**
   - Game forums
   - Modding Discord
   - Share your creations!

### Reporting Issues

If something doesn't work:

1. **Check log file** - Copy error messages
2. **Verify files** - sprite.json valid? PNGs correct?
3. **Test with simple example** - Try 4 frame animation first
4. **Ask for help** - Provide:
   - Folder structure
   - sprite.json contents
   - Error messages from log

---

## Examples: Step by Step

### Example 1: Replace Zombie Sprites

**Goal:** Make zombie walk faster

1. **Find folder:**
   ```
   custom/monsters/Zombie/
   ```

2. **Create sprite.json:**
   ```json
   {
     "fps": 20.0,
     "speed": 1.5
   }
   ```

3. **Launch game** - Zombies now walk 50% faster!

### Example 2: Add Custom Missile

**Goal:** Custom appearance for Frozen Orb

1. **Create frames:**
   ```
   custom/missiles/FrozenOrb/
   ├── 0.png  (your custom image)
   ├── 1.png
   ├── ...
   └── 13.png
   ```

2. **Add config:**
   ```json
   {
     "fps": 15.0
   }
   ```

3. **Test** - Cast Frozen Orb spell, see custom sprites!

### Example 3: Animated Chest

**Goal:** Chest that opens when clicked

1. **Create frames:**
   ```
   custom/objects/Chest/
   ├── sheet.png  (8 frames: closed → opening → open)
   ```

2. **Configure:**
   ```json
   {
     "frame_count": 8,
     "fps": 12.0,
     "layout": "horizontal"
   }
   ```

3. **In-game** - Smooth opening animation!

---

## Sharing Your Sprites

Want to share with others?

### Package Your Sprites

1. **Create folder structure:**
   ```
   MySpritePack/
   ├── README.txt  (installation instructions)
   ├── preview.png  (screenshots)
   └── custom/
       └── (your sprite folders)
   ```

2. **Write README:**
   ```
   My Custom Sprite Pack
   Installation:
   1. Copy the 'custom' folder to your game directory
   2. Merge with existing 'custom' folder if present
   3. Launch game
   ```

3. **Zip and share!**

---

## Summary

**Key Points:**
- ✅ Put custom sprites in `custom/` folder
- ✅ Match folder names exactly
- ✅ Create `sprite.json` for configuration
- ✅ Use PNG with transparency
- ✅ Use tools to help (Sheet Analyzer)
- ✅ Check logs if problems
- ✅ Start simple, get fancy later

**Remember:** You don't need to know programming. Just follow the folder structure, create your images, and add a simple JSON file!

---

## Next Steps

- **Try the tools:** [Sheet Analyzer](../tools/sheet-analyzer.html), [Globe Generator](../tools/globe-generator.html)
- **Browse examples:** `assets/images/` for inspiration
- **Get creative:** The game is yours to customize!

Happy sprite-making! 🎨