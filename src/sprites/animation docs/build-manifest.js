#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const DOCS_DIR = __dirname;
const OUTPUT_FILE = path.join(DOCS_DIR, 'docs-manifest.json');

// Files/folders to ignore
const IGNORE = [
    'index.html',
    'build-manifest.js',
    'docs-manifest.json',
    'node_modules',
    '.git',
    '.DS_Store',
    'shared'  // Add shared folder to ignore list
];

function buildManifest() {
    const manifest = {};
    
    console.log('📂 Scanning directory:', DOCS_DIR);
    
    // Read all entries in docs/
    const entries = fs.readdirSync(DOCS_DIR, { withFileTypes: true });
    
    for (const entry of entries) {
        if (!entry.isDirectory()) continue;
        if (IGNORE.includes(entry.name)) continue;
        
        const dirPath = path.join(DOCS_DIR, entry.name);
        
        console.log(`\n📁 Processing folder: ${entry.name}`);
        
        // Check for _meta.json
        const metaPath = path.join(dirPath, '_meta.json');
        let meta = {
            title: entry.name.charAt(0).toUpperCase() + entry.name.slice(1),
            icon: '📄',
            order: 999
        };
        
        if (fs.existsSync(metaPath)) {
            try {
                const metaContent = fs.readFileSync(metaPath, 'utf8');
                meta = { ...meta, ...JSON.parse(metaContent) };
                console.log(`  ✓ Loaded _meta.json: ${meta.title} ${meta.icon}`);
            } catch (error) {
                console.warn(`  ⚠️  Warning: Could not parse ${metaPath}`);
            }
        } else {
            console.log(`  ℹ️  No _meta.json found, using defaults`);
        }
        
        // Scan files in directory
        const files = [];
        const dirEntries = fs.readdirSync(dirPath, { withFileTypes: true });
        
        for (const fileEntry of dirEntries) {
            if (!fileEntry.isFile()) continue;
            if (fileEntry.name.startsWith('_')) continue; // Skip meta files
            
            const ext = path.extname(fileEntry.name);
            const basename = path.basename(fileEntry.name, ext);
            
            // Determine file type
            let type = 'unknown';
            if (ext === '.html') type = 'html';
            else if (ext === '.md') type = 'markdown';
            else {
                console.log(`  ⊘ Skipping ${fileEntry.name} (unsupported type)`);
                continue; // Skip other file types
            }
            
            // Generate title from filename
            let title = basename
                .split('-')
                .map(word => word.charAt(0).toUpperCase() + word.slice(1))
                .join(' ');
            
            files.push({
                name: fileEntry.name,
                title: title,
                type: type
            });
            
            console.log(`  ✓ Added: ${title} (${type})`);
        }
        
        // Sort files alphabetically by title
        files.sort((a, b) => a.title.localeCompare(b.title));
        
        manifest[entry.name] = {
            title: meta.title,
            icon: meta.icon,
            order: meta.order,
            files: files
        };
    }
    
    // Write manifest
    fs.writeFileSync(
        OUTPUT_FILE,
        JSON.stringify(manifest, null, 2),
        'utf8'
    );
    
    console.log('\n✅ Manifest generated successfully!');
    console.log(`📄 Output: ${OUTPUT_FILE}`);
    console.log('\n📊 Sections found:\n');
    
    Object.entries(manifest)
        .sort((a, b) => a[1].order - b[1].order)
        .forEach(([key, section]) => {
            console.log(`  ${section.icon} ${section.title} (${section.files.length} files)`);
            section.files.forEach(file => {
                console.log(`    └─ ${file.title}`);
            });
        });
}

// Run the build
try {
    buildManifest();
    process.exit(0);
} catch (error) {
    console.error('❌ Error building manifest:', error.message);
    console.error(error.stack);
    process.exit(1);
}