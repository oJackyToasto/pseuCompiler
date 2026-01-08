// Documentation structure
const docsStructure = {
    statements: [
        { name: 'IF Statement', file: 'statements/if.md' },
        { name: 'WHILE Loop', file: 'statements/while.md' }
    ],
    builtins: [
        // Add builtin docs here when available
    ]
};

let currentTheme = 'dark';

// Initialize documentation viewer
async function initDocs() {
    // Load theme preference
    const savedTheme = localStorage.getItem('docsTheme') || 'dark';
    setTheme(savedTheme);
    
    // Build navigation
    buildNavigation();
    
    // Load initial document based on URL hash or default
    const hash = window.location.hash.slice(1);
    if (hash) {
        await loadDocument(hash);
    } else {
        // Load first available document
        const firstDoc = docsStructure.statements[0];
        if (firstDoc) {
            await loadDocument(firstDoc.file);
        }
    }
    
    // Set up theme toggle
    const themeBtn = document.getElementById('themeBtn');
    if (themeBtn) {
        themeBtn.addEventListener('click', toggleTheme);
    }
}

// Build navigation menu
function buildNavigation() {
    const nav = document.getElementById('docsNav');
    if (!nav) return;
    
    nav.innerHTML = '';
    
    // Statements section
    if (docsStructure.statements.length > 0) {
        const statementsGroup = document.createElement('li');
        statementsGroup.className = 'nav-group';
        statementsGroup.innerHTML = '<h3>Statements</h3>';
        nav.appendChild(statementsGroup);
        
        const statementsList = document.createElement('ul');
        statementsList.className = 'nav-items';
        
        docsStructure.statements.forEach(doc => {
            const item = document.createElement('li');
            const link = document.createElement('a');
            link.href = `#${doc.file}`;
            link.textContent = doc.name;
            link.addEventListener('click', (e) => {
                e.preventDefault();
                loadDocument(doc.file);
                window.location.hash = doc.file;
            });
            item.appendChild(link);
            statementsList.appendChild(item);
        });
        
        statementsGroup.appendChild(statementsList);
    }
    
    // Builtins section
    if (docsStructure.builtins.length > 0) {
        const builtinsGroup = document.createElement('li');
        builtinsGroup.className = 'nav-group';
        builtinsGroup.innerHTML = '<h3>Built-in Functions</h3>';
        nav.appendChild(builtinsGroup);
        
        const builtinsList = document.createElement('ul');
        builtinsList.className = 'nav-items';
        
        docsStructure.builtins.forEach(doc => {
            const item = document.createElement('li');
            const link = document.createElement('a');
            link.href = `#${doc.file}`;
            link.textContent = doc.name;
            link.addEventListener('click', (e) => {
                e.preventDefault();
                loadDocument(doc.file);
                window.location.hash = doc.file;
            });
            item.appendChild(link);
            builtinsList.appendChild(item);
        });
        
        builtinsGroup.appendChild(builtinsList);
    }
}

// Load and render markdown document
async function loadDocument(filePath) {
    const contentDiv = document.getElementById('docsContent');
    if (!contentDiv) return;
    
    // Show loading state
    contentDiv.innerHTML = '<div class="loading">Loading documentation...</div>';
    
    try {
        // Fetch markdown file
        const response = await fetch(`documentations/${filePath}`);
        if (!response.ok) {
            throw new Error(`Failed to load: ${response.statusText}`);
        }
        
        const markdown = await response.text();
        
        // Configure marked options
        marked.setOptions({
            breaks: true,
            gfm: true,
            highlight: function(code, lang) {
                // Simple code highlighting - you can enhance this with a library like highlight.js
                return `<pre><code class="language-${lang}">${escapeHtml(code)}</code></pre>`;
            }
        });
        
        // Render markdown to HTML
        const html = marked.parse(markdown);
        contentDiv.innerHTML = html;
        
        // Update active nav item
        updateActiveNav(filePath);
        
        // Scroll to top
        window.scrollTo(0, 0);
        
    } catch (error) {
        console.error('Error loading document:', error);
        contentDiv.innerHTML = `
            <div class="error">
                <h2>Error loading documentation</h2>
                <p>${escapeHtml(error.message)}</p>
                <p>File: <code>${escapeHtml(filePath)}</code></p>
            </div>
        `;
    }
}

// Update active navigation item
function updateActiveNav(filePath) {
    const nav = document.getElementById('docsNav');
    if (!nav) return;
    
    const links = nav.querySelectorAll('a');
    links.forEach(link => {
        if (link.getAttribute('href') === `#${filePath}`) {
            link.classList.add('active');
        } else {
            link.classList.remove('active');
        }
    });
}

// Toggle theme
function toggleTheme() {
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    localStorage.setItem('docsTheme', newTheme);
}

// Set theme
function setTheme(theme) {
    currentTheme = theme;
    const body = document.body;
    const themeBtn = document.getElementById('themeBtn');
    
    if (theme === 'light') {
        body.classList.add('light');
        if (themeBtn) themeBtn.textContent = 'Light';
    } else {
        body.classList.remove('light');
        if (themeBtn) themeBtn.textContent = 'Dark';
    }
}

// Escape HTML to prevent XSS
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Handle hash changes (back/forward navigation)
window.addEventListener('hashchange', () => {
    const hash = window.location.hash.slice(1);
    if (hash) {
        loadDocument(hash);
    }
});

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', initDocs);

