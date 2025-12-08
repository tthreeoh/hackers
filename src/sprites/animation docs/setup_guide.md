# Sprite Animation Documentation - Setup Guide

Complete directory structure and setup instructions for the documentation system.

## Final Directory Structure

```
docs/
├── index.html                    # Main documentation hub (navigation)
├── docs-manifest.json            # Configuration for navigation
├──shared/
│   ├── shared.js                 # Shared utilities (files, colors, canvas, etc.)
│   └── shared-styles.css         # Unified dark theme CSS
├── animation/
│   ├── overview.md               # System architecture overview
│   ├── sprite-config.html        # Sprite.json format reference (existing)
│   └── developer-guide.md        # Step-by-step implementation guide
├── guides/
│   ├── examples.md               # Real-world code examples
│   ├── migration-guide.md        # Upgrading from old system
│   └── user-guide.md             # For non-programmers
├── api/
│   └── api-reference.md          # Complete API documentation
└── tools/
    ├── globe-generator.html      # Health/mana globe frame generator (existing)
    ├── icon-generator.html       # Icon generator (existing)
    └── sheet-analyzer.html       # Sprite sheet analyzer (existing)
```

## Setup Instructions

### Step 1: Create Directory Structure

```bash
# Create main directories
mkdir -p docs/{shared,animation,guides,api,tools}

# Or on Windows:
md docs\shared
md docs\animation
md docs\guides
md docs\api
md docs\tools
```

### Step 2: Place Files

**Root Level:**
- `index.html` → docs/index.html
- `docs-manifest.json` → docs/docs-manifest.json

**Shared Resources:**
- `shared.js` → docs/shared/shared.js
- `shared-styles.css` → docs/shared/shared-styles.css

**Animation Section:**
- `overview.md` → docs/animation/overview.md
- `sprite-config.html` (existing) → docs/animation/sprite-config.html
- `developer-guide.md` → docs/animation/developer-guide.md

**Guides Section:**
- `examples.md` → docs/guides/examples.md
- `migration-guide.md` → docs/guides/migration-guide.md
- `user-guide.md` → docs/guides/user-guide.md

**API Section:**
- `api-reference.md` → docs/api/api-reference.md

**Tools Section:**
- `globe-generator.html` (existing) → docs/tools/globe-generator.html
- `icon-generator.html` (existing) → docs/tools/icon-generator.html
- `sheet-analyzer.html` (existing) → docs/tools/sheet-analyzer.html

### Step 3: Update Existing HTML Files

Each HTML tool file needs to include the shared resources. Add to `<head>`:

```html
<!-- In globe-generator.html, icon-generator.html, sheet-analyzer.html -->
<link rel="stylesheet" href="../shared/shared-styles.css">
<script src="../shared/shared.js"></script>
```

**Optional:** Update their existing styles to use CSS variables from shared-styles.css:

```css
/* Instead of: */
body {
    background: #1e1e2e;
}

/* Use: */
body {
    background: var(--bg-primary);
}
```

### Step 4: Update sprite-config.html

Add shared resources to make styling consistent:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sprite Configuration Documentation</title>
    
    <!-- Add these lines -->
    <link rel="stylesheet" href="../shared/shared-styles.css">
    <script src="../shared/shared.js"></script>
    
    <!-- Keep existing custom styles if needed -->
    <style>
        /* Custom styles specific to this page */
    </style>
</head>
<!-- Rest of the file unchanged -->
```

### Step 5: Verify Links

Test that all internal links work correctly:

**In overview.md, developer-guide.md, etc.:**
```markdown
- [Sprite Configuration](sprite-config.html)
- [Developer Guide](developer-guide.html)
- [Tools: Sheet Analyzer](../tools/sheet-analyzer.html)
```

**In index.html navigation:**
The `docs-manifest.json` handles all paths automatically.

## Usage

### Viewing Documentation

**Local Development:**
```bash
# Option 1: Python HTTP server
cd docs
python -m http.server 8000

# Option 2: Node.js HTTP server
cd docs
npx http-server -p 8000

# Then open: http://localhost:8000
```

**Production:**
- Upload entire `docs/` folder to web server
- Access via `https://yourdomain.com/docs/`

### Updating Content

**To add new documentation page:**

1. Create the file (markdown or HTML)
2. Add entry to `docs-manifest.json`:

```json
{
  "animation": {
    "files": [
      {
        "name": "new-guide.md",
        "title": "New Guide",
        "type": "markdown",
        "description": "Description here"
      }
    ]
  }
}
```

3. Refresh browser - new page appears in navigation

**To add new tool:**

1. Create HTML file in `tools/`
2. Include shared resources:
   ```html
   <link rel="stylesheet" href="../shared/shared-styles.css">
   <script src="../shared/shared.js"></script>
   ```
3. Add to `docs-manifest.json`

## Shared Resources Usage

### In HTML Files

```html
<!-- Include at top -->
<link rel="stylesheet" href="../shared/shared-styles.css">
<script src="../shared/shared.js"></script>

<!-- Use in your code -->
<script>
    // File operations
    Shared.files.downloadCanvas(myCanvas, "sprite.png");
    Shared.files.downloadJSON(config, "sprite.json");
    
    // Color utilities
    const rgb = Shared.color.hexToRgb("#ff0000");
    const hex = Shared.color.rgbToHex(255, 0, 0);
    
    // Canvas utilities
    const {canvas, ctx} = Shared.canvas.create(256, 256);
    Shared.canvas.drawCircle(ctx, 128, 128, 50, "#ff0000");
    
    // UI utilities
    Shared.ui.showNotification("Saved successfully!", "success");
    
    // Storage (with fallback)
    Shared.storage.set("presets", myPresets);
    const presets = Shared.storage.get("presets", []);
</script>

<!-- Use CSS classes -->
<div class="container">
    <div class="card">
        <div class="card-header">My Tool</div>
        <button class="btn btn-primary">Click Me</button>
    </div>
</div>
```

### CSS Variables Available

```css
/* Colors */
--bg-primary: #0f172a
--bg-secondary: #1e293b
--text-primary: #f1f5f9
--text-secondary: #cbd5e1
--accent-primary: #6366f1

/* Spacing */
--spacing-sm: 0.5rem
--spacing-md: 1rem
--spacing-lg: 1.5rem

/* Use them: */
.my-element {
    background: var(--bg-secondary);
    padding: var(--spacing-md);
    color: var(--text-primary);
}
```

## Customization

### Changing Theme

Edit `docs/shared/shared-styles.css`:

```css
:root {
    /* Change primary color */
    --accent-primary: #6366f1;  /* Indigo */
    --accent-primary: #10b981;  /* Green */
    --accent-primary: #f59e0b;  /* Amber */
    
    /* Change background darkness */
    --bg-primary: #0f172a;      /* Dark */
    --bg-primary: #1e293b;      /* Medium */
    --bg-primary: #334155;      /* Light */
}
```

### Adding New Shared Utility

Edit `docs/shared/shared.js`:

```javascript
// Add to Shared object
Shared.myNewUtility = {
    myFunction(param) {
        // Implementation
    }
};

// Use in any HTML file
Shared.myNewUtility.myFunction("test");
```

## Examples

### Example 1: Create New Tool

```html
<!-- docs/tools/my-new-tool.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>My New Tool</title>
    <link rel="stylesheet" href="../shared/shared-styles.css">
    <script src="../shared/shared.js"></script>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🛠️ My New Tool</h1>
            <p>Description of what it does</p>
        </div>
        
        <div class="card">
            <div class="card-header">Controls</div>
            <label>
                Input:
                <input type="number" id="myInput" value="10">
            </label>
            <button onclick="process()">Process</button>
        </div>
        
        <canvas id="canvas"></canvas>
    </div>
    
    <script>
        function process() {
            const value = parseInt(document.getElementById('myInput').value);
            
            // Use shared utilities
            const {canvas, ctx} = Shared.canvas.create(256, 256);
            Shared.canvas.drawCircle(ctx, 128, 128, value, "#6366f1");
            
            document.getElementById('canvas').replaceWith(canvas);
            
            Shared.ui.showNotification("Processing complete!", "success");
        }
    </script>
</body>
</html>
```

Add to `docs-manifest.json`:
```json
{
  "tools": {
    "files": [
      {
        "name": "my-new-tool.html",
        "title": "My New Tool",
        "type": "html",
        "description": "Does something useful"
      }
    ]
  }
}
```

### Example 2: Add Markdown Guide

```markdown
<!-- docs/guides/my-guide.md -->
# My Guide Title

## Section 1

Content here...

## Section 2

More content...

[Link to tool](../tools/sheet-analyzer.html)
```

Add to `docs-manifest.json`:
```json
{
  "guides": {
    "files": [
      {
        "name": "my-guide.md",
        "title": "My Guide",
        "type": "markdown",
        "description": "Helpful information"
      }
    ]
  }
}
```

## Troubleshooting

### Links Not Working

**Problem:** Clicking navigation doesn't load pages

**Solution:**
- Check file paths in `docs-manifest.json`
- Verify files exist at specified paths
- Use browser console (F12) to see errors

### Shared Resources Not Loading

**Problem:** Styles or JavaScript not applied

**Solution:**
- Check path to shared resources: `../shared/shared.js`
- Verify file exists: `docs/shared/shared.js`
- Check browser console for 404 errors

### Markdown Not Rendering

**Problem:** Markdown shows as plain text

**Solution:**
- Verify marked.js loaded in index.html
- Check file has `.md` extension
- Type in manifest must be `"type": "markdown"`

## Performance Tips

### Optimizing Load Times

1. **Minify CSS/JS for production:**
   ```bash
   # Using npm
   npx uglify-js shared/shared.js -o shared/shared.min.js
   npx clean-css-cli shared/shared-styles.css -o shared/shared-styles.min.css
   ```

2. **Cache manifest:**
   ```javascript
   // In index.html, cache manifest after first load
   localStorage.setItem('manifest-cache', JSON.stringify(manifest));
   ```

3. **Lazy load tools:**
   Only load tool HTML when user clicks, not upfront

## Deployment

### GitHub Pages

```bash
# Push to gh-pages branch
git subtree push --prefix docs origin gh-pages

# Access at: https://username.github.io/repository/
```

### Static Host (Netlify, Vercel)

1. Set build directory: `docs`
2. No build command needed (static files)
3. Deploy

### Self-Hosted

```nginx
# nginx config
location /docs {
    root /var/www/html;
    index index.html;
    try_files $uri $uri/ =404;
}
```

## Maintenance

### Regular Updates

- **Check links:** Quarterly link checking
- **Update examples:** When code changes
- **Add new features:** Document immediately
- **User feedback:** Review and incorporate

### Version Control

```bash
# Tag releases
git tag -a v2.0 -m "Auto-discovery system docs"
git push origin v2.0
```

## Summary

✅ **Complete documentation system** with unified styling  
✅ **Shared utilities** to reduce code duplication  
✅ **Easy to maintain** - just add to manifest  
✅ **User-friendly** - clear navigation and search  
✅ **Developer-friendly** - reusable components  
✅ **Extensible** - easy to add new sections  

---

**Questions?** Check the individual guide files or create an issue!