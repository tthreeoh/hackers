#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const DOCS_DIR = __dirname;
const MANIFEST_FILE = path.join(DOCS_DIR, 'docs-manifest.json');
const OUTPUT_FILE = path.join(DOCS_DIR, 'index.html');

function generateStaticIndex() {
    console.log('📄 Generating static index.html...\n');

    // Read the manifest
    if (!fs.existsSync(MANIFEST_FILE)) {
        console.error('❌ Error: docs-manifest.json not found!');
        console.log('   Run "node build-manifest.js" first.\n');
        process.exit(1);
    }

    const manifest = JSON.parse(fs.readFileSync(MANIFEST_FILE, 'utf8'));

    // Sort sections by order
    const sections = Object.entries(manifest).sort((a, b) => {
        return (a[1].order || 999) - (b[1].order || 999);
    });

    // Generate navigation HTML
    const navHTML = sections.map(([key, section]) => {
        const filesHTML = section.files.map(file => {
            const icon = file.type === 'html' ? '🛠️' : '📄';
            return `
                        <li class="nav-item" data-section="${key}" data-file="${file.name}">
                            <span class="nav-item-icon">${icon}</span>
                            <span>${file.title}</span>
                        </li>`;
        }).join('');

        return `
                <div class="nav-section">
                    <div class="nav-section-title">
                        <span class="nav-section-icon">${section.icon || '📁'}</span>
                        <span>${section.title}</span>
                    </div>
                    <ul class="nav-items">
${filesHTML}
                    </ul>
                </div>`;
    }).join('\n');

    // Generate home page section cards
    const sectionCardsHTML = sections.map(([key, section]) => {
        return `
                            <div class="section-card" onclick="expandSection('${key}')">
                                <div class="section-card-icon">${section.icon || '📁'}</div>
                                <h3>${section.title}</h3>
                                <p class="section-card-count">${section.files.length} item${section.files.length !== 1 ? 's' : ''}</p>
                            </div>`;
    }).join('');

    // Generate the complete HTML
    const html = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="shared/shared-styles.css">
    <script src="shared/shared.js"></script>
    
    <!-- Add Prism.js for syntax highlighting -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/prism.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-rust.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-json.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-bash.min.js"></script>
    
    <title>Documentation Hub</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f172a;
            color: #e2e8f0;
            display: flex;
            height: 100vh;
            overflow: hidden;
        }

        /* Sidebar */
        .sidebar {
            width: 280px;
            background: #1e293b;
            border-right: 1px solid #334155;
            display: flex;
            flex-direction: column;
            transition: transform 0.3s;
        }

        .sidebar.collapsed {
            transform: translateX(-280px);
        }

        .sidebar-header {
            padding: 1.5rem;
            border-bottom: 1px solid #334155;
        }

        .sidebar-header h1 {
            font-size: 1.5rem;
            color: #f1f5f9;
            margin-bottom: 0.5rem;
        }

        .sidebar-header p {
            font-size: 0.875rem;
            color: #94a3b8;
        }

        .sidebar-nav {
            flex: 1;
            overflow-y: auto;
            padding: 1rem 0;
        }

        .nav-section {
            margin-bottom: 1.5rem;
        }

        .nav-section-title {
            padding: 0.5rem 1.5rem;
            font-size: 0.875rem;
            font-weight: 600;
            color: #94a3b8;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .nav-section-icon {
            font-size: 1.25rem;
        }

        .nav-items {
            list-style: none;
        }

        .nav-item {
            padding: 0.75rem 1.5rem;
            cursor: pointer;
            transition: background 0.2s;
            display: flex;
            align-items: center;
            gap: 0.75rem;
            color: #cbd5e1;
        }

        .nav-item:hover {
            background: #334155;
        }

        .nav-item.active {
            background: #3730a3;
            color: #e0e7ff;
            border-left: 3px solid #6366f1;
        }

        .nav-item-icon {
            font-size: 0.875rem;
            opacity: 0.6;
        }

        /* Main Content */
        .main {
            flex: 1;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }

        .topbar {
            height: 60px;
            background: #1e293b;
            border-bottom: 1px solid #334155;
            display: flex;
            align-items: center;
            padding: 0 1.5rem;
            gap: 1rem;
        }

        .menu-toggle {
            background: none;
            border: none;
            color: #cbd5e1;
            cursor: pointer;
            padding: 0.5rem;
            border-radius: 0.375rem;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .menu-toggle:hover {
            background: #334155;
        }

        .breadcrumb {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            color: #94a3b8;
            font-size: 0.875rem;
        }

        .breadcrumb-separator {
            opacity: 0.5;
        }

        .breadcrumb-current {
            color: #e2e8f0;
            font-weight: 500;
        }

        .content {
            flex: 1;
            overflow: auto;
            background: #0f172a;
        }

        .content-wrapper {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }

        /* Home Page */
        .home {
            text-align: center;
            padding: 4rem 2rem;
        }

        .home h1 {
            font-size: 3rem;
            margin-bottom: 1rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }

        .home p {
            font-size: 1.25rem;
            color: #94a3b8;
            margin-bottom: 3rem;
        }

        .section-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-top: 3rem;
        }

        .section-card {
            background: #1e293b;
            border: 1px solid #334155;
            border-radius: 0.75rem;
            padding: 2rem;
            cursor: pointer;
            transition: all 0.2s;
        }

        .section-card:hover {
            border-color: #6366f1;
            transform: translateY(-2px);
            box-shadow: 0 10px 30px rgba(99, 102, 241, 0.2);
        }

        .section-card-icon {
            font-size: 3rem;
            margin-bottom: 1rem;
        }

        .section-card h3 {
            font-size: 1.5rem;
            margin-bottom: 0.5rem;
            color: #f1f5f9;
        }

        .section-card p {
            color: #94a3b8;
            font-size: 0.875rem;
        }

        .section-card-count {
            margin-top: 1rem;
            font-size: 0.75rem;
            color: #64748b;
        }

        /* iframe container */
        .iframe-container {
            width: 100%;
            height: 100%;
            border: none;
        }

        .iframe-container iframe {
            width: 100%;
            height: 100%;
            border: none;
        }

        /* Markdown content styling */
        .markdown-content {
            line-height: 1.6;
        }

        .markdown-content h1 {
            font-size: 2.5rem;
            margin-bottom: 1rem;
            color: #f1f5f9;
            border-bottom: 2px solid #334155;
            padding-bottom: 0.5rem;
        }

        .markdown-content h2 {
            font-size: 2rem;
            margin-top: 2rem;
            margin-bottom: 1rem;
            color: #e2e8f0;
        }

        .markdown-content h3 {
            font-size: 1.5rem;
            margin-top: 1.5rem;
            margin-bottom: 0.75rem;
            color: #cbd5e1;
        }

        .markdown-content p {
            margin-bottom: 1rem;
            color: #94a3b8;
        }

        .markdown-content pre {
            background: #1e293b;
            border: 1px solid #334155;
            border-radius: 0.5rem;
            padding: 1rem;
            overflow-x: auto;
            margin-bottom: 1rem;
        }

        .markdown-content code {
            background: #1e293b;
            padding: 0.2rem 0.4rem;
            border-radius: 0.25rem;
            font-size: 0.875em;
            color: #c084fc;
        }

        .markdown-content pre code {
            background: none;
            padding: 0;
        }

        .markdown-content ul, .markdown-content ol {
            margin-left: 1.5rem;
            margin-bottom: 1rem;
            color: #94a3b8;
        }

        .markdown-content li {
            margin-bottom: 0.5rem;
        }

        /* Loading spinner */
        .loading {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100%;
        }

        .spinner {
            width: 50px;
            height: 50px;
            border: 3px solid #334155;
            border-top-color: #6366f1;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }

        /* Error message */
        .error {
            background: #7f1d1d;
            border: 1px solid #991b1b;
            color: #fecaca;
            padding: 1rem;
            border-radius: 0.5rem;
            margin: 2rem 0;
        }

        /* Mobile responsive */
        @media (max-width: 768px) {
            .sidebar {
                position: absolute;
                z-index: 1000;
                height: 100vh;
            }

            .sidebar.collapsed {
                transform: translateX(-280px);
            }
        }
    </style>
</head>
<body>
    <!-- Sidebar -->
    <aside class="sidebar" id="sidebar">
        <div class="sidebar-header">
            <h1>📚 Docs</h1>
            <p>Animation System Documentation</p>
        </div>
        <nav class="sidebar-nav" id="sidebarNav">
${navHTML}
        </nav>
    </aside>

    <!-- Main Content -->
    <main class="main">
        <div class="topbar">
            <button class="menu-toggle" id="menuToggle">
                <svg width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="3" y1="12" x2="21" y2="12"></line>
                    <line x1="3" y1="6" x2="21" y2="6"></line>
                    <line x1="3" y1="18" x2="21" y2="18"></line>
                </svg>
            </button>
            <div class="breadcrumb" id="breadcrumb">
                <span>Home</span>
            </div>
        </div>
        <div class="content" id="content">
            <div class="content-wrapper">
                <div class="home">
                    <h1>📚 Documentation Hub</h1>
                    <p>Select a section to get started</p>
                    
                    <div class="section-grid">
${sectionCardsHTML}
                    </div>
                </div>
            </div>
        </div>
    </main>

    <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
    <script>
        // Embedded manifest
        const manifest = ${JSON.stringify(manifest, null, 8)};
        
        // State
        let currentPage = null;
        let sidebarCollapsed = false;

        // DOM Elements
        const sidebar = document.getElementById('sidebar');
        const sidebarNav = document.getElementById('sidebarNav');
        const menuToggle = document.getElementById('menuToggle');
        const breadcrumb = document.getElementById('breadcrumb');
        const content = document.getElementById('content');

        // Initialize
        function init() {
            setupEventListeners();
            showHome();
        }

        function setupEventListeners() {
            // Menu toggle
            menuToggle.addEventListener('click', () => {
                sidebarCollapsed = !sidebarCollapsed;
                sidebar.classList.toggle('collapsed', sidebarCollapsed);
            });

            // Navigation items
            document.querySelectorAll('.nav-item').forEach(item => {
                item.addEventListener('click', function() {
                    const section = this.dataset.section;
                    const fileName = this.dataset.file;
                    const file = manifest[section].files.find(f => f.name === fileName);
                    loadPage(section, file);
                });
            });

            // Home link in breadcrumb
            document.addEventListener('click', (e) => {
                if (e.target.closest('.breadcrumb span:first-child')) {
                    showHome();
                }
            });
        }

        // Show home page
        function showHome() {
            currentPage = null;
            updateBreadcrumb(['Home']);
            
            // Clear active states
            document.querySelectorAll('.nav-item').forEach(item => {
                item.classList.remove('active');
            });

            const sections = Object.entries(manifest).sort((a, b) => {
                return (a[1].order || 999) - (b[1].order || 999);
            });

            content.innerHTML = \`
                <div class="content-wrapper">
                    <div class="home">
                        <h1>📚 Documentation Hub</h1>
                        <p>Select a section to get started</p>
                        
                        <div class="section-grid">
                            \${sections.map(([key, section]) => \`
                                <div class="section-card" onclick="expandSection('\${key}')">
                                    <div class="section-card-icon">\${section.icon || '📁'}</div>
                                    <h3>\${section.title}</h3>
                                    <p class="section-card-count">\${section.files.length} item\${section.files.length !== 1 ? 's' : ''}</p>
                                </div>
                            \`).join('')}
                        </div>
                    </div>
                </div>
            \`;
        }

        // Expand section (scroll to it in sidebar and highlight)
        function expandSection(sectionKey) {
            const section = manifest[sectionKey];
            if (section && section.files.length > 0) {
                loadPage(sectionKey, section.files[0]);
            }
        }

        // Load page
        async function loadPage(sectionKey, file) {
            currentPage = { section: sectionKey, file };
            
            const section = manifest[sectionKey];
            updateBreadcrumb(['Home', section.title, file.title]);
            
            // Update active state
            document.querySelectorAll('.nav-item').forEach(item => {
                item.classList.remove('active');
                if (item.dataset.section === sectionKey && item.dataset.file === file.name) {
                    item.classList.add('active');
                }
            });

            // Show loading
            content.innerHTML = '<div class="loading"><div class="spinner"></div></div>';

            try {
                if (file.type === 'html') {
                    // Load HTML in iframe
                    const path = \`\${sectionKey}/\${file.name}\`;
                    content.innerHTML = \`
                        <div class="iframe-container">
                            <iframe src="\${path}"></iframe>
                        </div>
                    \`;
                } else if (file.type === 'markdown') {
                    // Load and render markdown
                    const path = \`\${sectionKey}/\${file.name}\`;
                    const response = await fetch(path);
                    const markdown = await response.text();
                    const html = marked.parse(markdown);
                    content.innerHTML = \`
                        <div class="content-wrapper">
                            <div class="markdown-content">
                                \${html}
                            </div>
                        </div>
                    \`;
                    
                    // Highlight code blocks with Prism
                    if (typeof Prism !== 'undefined') {
                        Prism.highlightAllUnder(content);
                    }
                }
            } catch (error) {
                content.innerHTML = \`
                    <div class="content-wrapper">
                        <div class="error">
                            <strong>Error loading content:</strong> \${error.message}
                            <p style="margin-top: 0.5rem;">Make sure the file exists at: \${sectionKey}/\${file.name}</p>
                        </div>
                    </div>
                \`;
            }
        }

        // Update breadcrumb
        function updateBreadcrumb(items) {
            breadcrumb.innerHTML = items.map((item, index) => {
                const isLast = index === items.length - 1;
                const isFirst = index === 0;
                return \`
                    \${isFirst ? '' : '<span class="breadcrumb-separator">›</span>'}
                    <span class="\${isLast ? 'breadcrumb-current' : ''}" 
                          \${isFirst ? 'style="cursor: pointer;"' : ''}>
                        \${item}
                    </span>
                \`;
            }).join('');
        }

        // Initialize app
        init();
    </script>
</body>
</html>`;

    // Write the file
    fs.writeFileSync(OUTPUT_FILE, html, 'utf8');
    
    console.log('✅ Static index.html generated successfully!');
    console.log(`📄 Output: ${OUTPUT_FILE}\n`);
    console.log('📊 Embedded sections:');
    sections.forEach(([key, section]) => {
        console.log(`  ${section.icon} ${section.title} (${section.files.length} files)`);
    });
    console.log('\n🚀 Your documentation hub is ready!');
    console.log('   Open index.html in a browser or serve with:');
    console.log('   python -m http.server 8000\n');
}

// Run the generator
try {
    generateStaticIndex();
    process.exit(0);
} catch (error) {
    console.error('❌ Error generating index:', error.message);
    console.error(error.stack);
    process.exit(1);
}