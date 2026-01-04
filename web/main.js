import init, { PseudocodeEngine } from './pkg/pseudocode_wasm.js';

let engine = null;
let editor = null;
let errorDecorations = [];

// Example code
const examples = {
    bubble_sort: `// Bubble Sort Algorithm
DECLARE n <- 10: INTEGER
CONSTANT n

DECLARE arr : ARRAY[1:10] OF INTEGER
DECLARE i : INTEGER
DECLARE j : INTEGER
DECLARE temp : INTEGER

// Initialize array with unsorted values
FOR i <- 1 TO n
    arr[i] <- ROUND(RANDOM() * 100, 0)
NEXT i

OUTPUT "Original array:"
FOR i <- 1 TO n
    OUTPUT arr[i], " "
NEXT i
OUTPUT ""

// Bubble Sort Algorithm
FOR i <- 1 TO n - 1
    FOR j <- 1 TO n - i
        IF arr[j] > arr[j + 1] THEN
            // Swap elements
            temp <- arr[j]
            arr[j] <- arr[j + 1]
            arr[j + 1] <- temp
        ENDIF
    NEXT j
NEXT i

OUTPUT "Sorted array:"
FOR i <- 1 TO n
    OUTPUT arr[i], " "
NEXT i
OUTPUT ""`,
    simple: `OUTPUT "Hello, World!"
DECLARE x <- 5: INTEGER
DECLARE y <- 10: INTEGER
DECLARE sum: INTEGER
sum <- x + y
OUTPUT "Sum of ", x, " and ", y, " is ", sum`
};

// Initialize Monaco Editor
function initMonaco() {
    require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' } });
    
    require(['vs/editor/editor.main'], function() {
        // Register pseudocode language
        monaco.languages.register({ id: 'pseudocode' });
        
        // Define syntax highlighting
        monaco.languages.setMonarchTokensProvider('pseudocode', {
            keywords: [
                'DECLARE', 'CONSTANT', 'FUNCTION', 'PROCEDURE', 'ENDFUNCTION', 'ENDPROCEDURE',
                'IF', 'THEN', 'ELSE', 'ENDIF', 'WHILE', 'DO', 'ENDWHILE',
                'FOR', 'TO', 'NEXT', 'REPEAT', 'UNTIL',
                'RETURN', 'CALL', 'INPUT', 'OUTPUT',
                'OPENFILE', 'CLOSEFILE', 'READFILE', 'WRITEFILE', 'SEEK',
                'GETRECORD', 'PUTRECORD',
                'INTEGER', 'REAL', 'STRING', 'CHAR', 'BOOLEAN', 'ARRAY', 'OF',
                'AND', 'OR', 'NOT', 'TRUE', 'FALSE',
                'TYPE', 'ENDTYPE', 'CASE', 'ENDCASE', 'OTHERWISE', 'BREAK'
            ],
            operators: ['<-', '=', '<>', '<', '>', '<=', '>=', '+', '-', '*', '/', 'MOD'],
            builtinFunctions: [
                'LENGTH', 'UPPER', 'LOWER', 'SUBSTRING', 'LEFT', 'RIGHT', 'MID',
                'ROUND', 'RANDOM', 'EOF', 'ASC', 'CHR'
            ],
            tokenizer: {
                root: [
                    [/[a-z_$][\w$]*/, {
                        cases: {
                            '@keywords': 'keyword',
                            '@builtinFunctions': 'type.identifier',
                            '@default': 'identifier'
                        }
                    }],
                    [/[A-Z][\w$]*/, {
                        cases: {
                            '@keywords': 'keyword',
                            '@builtinFunctions': 'type.identifier',
                            '@default': 'identifier'
                        }
                    }],
                    [/"([^"\\]|\\.)*"/, 'string'],
                    [/'([^'\\]|\\.)*'/, 'string'],
                    [/\d+\.\d+/, 'number.float'],
                    [/\d+/, 'number'],
                    [/\/\/.*$/, 'comment'],
                    [/<-/, 'operator'],
                    [/[=<>+\-*/]/, 'operator'],
                    [/[(),:;\[\]]/, 'delimiter']
                ]
            }
        });

        // Define theme colors
        monaco.editor.defineTheme('pseudocode-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [
                { token: 'keyword', foreground: '569cd6' },
                { token: 'string', foreground: 'ce9178' },
                { token: 'number', foreground: 'b5cea8' },
                { token: 'comment', foreground: '6a9955', fontStyle: 'italic' },
                { token: 'operator', foreground: 'd4d4d4' },
                { token: 'type.identifier', foreground: '4ec9b0' }
            ]
        });

        monaco.editor.defineTheme('pseudocode-light', {
            base: 'vs',
            inherit: true,
            rules: [
                { token: 'keyword', foreground: '0000ff' },
                { token: 'string', foreground: 'a31515' },
                { token: 'number', foreground: '098658' },
                { token: 'comment', foreground: '008000', fontStyle: 'italic' },
                { token: 'operator', foreground: '000000' },
                { token: 'type.identifier', foreground: '267f99' }
            ]
        });

        // Create editor
        editor = monaco.editor.create(document.getElementById('editor'), {
            value: examples.simple,
            language: 'pseudocode',
            theme: 'pseudocode-dark',
            automaticLayout: true,
            fontSize: 14,
            minimap: { enabled: true },
            scrollBeyondLastLine: false,
            wordWrap: 'on'
        });
    });
}

// Initialize WASM
async function initWasm() {
    try {
        await init();
        engine = new PseudocodeEngine();
        console.log('WASM initialized successfully');
    } catch (error) {
        console.error('Failed to initialize WASM:', error);
        showOutput('Error: Failed to load WASM module. Make sure you have built the WASM package.', 'error');
    }
}

// Show output in output panel
function showOutput(text, type = 'info') {
    const outputDiv = document.getElementById('output');
    outputDiv.textContent = text;
    outputDiv.className = `output ${type}`;
}

// Clear error decorations
function clearErrorDecorations() {
    if (editor && errorDecorations.length > 0) {
        editor.deltaDecorations(errorDecorations, []);
        errorDecorations = [];
    }
}

// Highlight error lines
function highlightErrors(errors) {
    if (!editor) return;
    
    clearErrorDecorations();
    
    const decorations = errors.map(error => ({
        range: new monaco.Range(error.line, 1, error.line, 1),
        options: {
            isWholeLine: true,
            className: 'error-line',
            glyphMarginClassName: 'error-glyph',
            hoverMessage: { value: error.message }
        }
    }));
    
    errorDecorations = editor.deltaDecorations([], decorations);
}

// Run code
async function runCode() {
    if (!engine) {
        showOutput('Error: WASM not initialized', 'error');
        return;
    }
    
    const code = editor.getValue();
    if (!code.trim()) {
        showOutput('No code to execute', 'info');
        return;
    }
    
    clearErrorDecorations();
    showOutput('Running...', 'info');
    
    try {
        const result = engine.execute(code);
        const executionResult = JSON.parse(result);
        
        let outputText = executionResult.output || '';
        
        if (executionResult.errors && executionResult.errors.length > 0) {
            outputText += '\n\n--- Errors ---\n';
            executionResult.errors.forEach(error => {
                outputText += `Line ${error.line}: ${error.message}\n`;
            });
            highlightErrors(executionResult.errors);
            showOutput(outputText, 'error');
        } else {
            showOutput(outputText || '(No output)', 'success');
        }
    } catch (error) {
        showOutput(`Error: ${error.message}`, 'error');
        console.error('Execution error:', error);
    }
}

// Check syntax
async function checkSyntax() {
    if (!engine) {
        showOutput('Error: WASM not initialized', 'error');
        return;
    }
    
    const code = editor.getValue();
    if (!code.trim()) {
        showOutput('No code to check', 'info');
        return;
    }
    
    clearErrorDecorations();
    
    try {
        const result = engine.check_syntax(code);
        const checkResult = JSON.parse(result);
        
        if (checkResult.valid) {
            showOutput('✓ Syntax check passed!', 'success');
        } else {
            let errorText = '✗ Syntax errors found:\n';
            checkResult.errors.forEach(error => {
                errorText += `Line ${error.line}: ${error.message}\n`;
            });
            highlightErrors(checkResult.errors);
            showOutput(errorText, 'error');
        }
    } catch (error) {
        showOutput(`Error: ${error.message}`, 'error');
        console.error('Syntax check error:', error);
    }
}

// Clear editor
function clearEditor() {
    if (editor) {
        editor.setValue('');
        clearErrorDecorations();
    }
    showOutput('', 'info');
}

// Clear output
function clearOutput() {
    showOutput('', 'info');
    clearErrorDecorations();
}

// Load example
function loadExample() {
    const select = document.getElementById('exampleSelect');
    const exampleName = select.value;
    if (exampleName && examples[exampleName] && editor) {
        editor.setValue(examples[exampleName]);
        clearErrorDecorations();
        showOutput('', 'info');
    }
}

// Toggle theme
function toggleTheme() {
    const body = document.body;
    const themeBtn = document.getElementById('themeBtn');
    
    if (body.classList.contains('light')) {
        body.classList.remove('light');
        if (editor) {
            monaco.editor.setTheme('pseudocode-dark');
        }
        themeBtn.textContent = '🌙 Dark';
    } else {
        body.classList.add('light');
        if (editor) {
            monaco.editor.setTheme('pseudocode-light');
        }
        themeBtn.textContent = '☀️ Light';
    }
}

// Add CSS for error highlighting
const style = document.createElement('style');
style.textContent = `
    .error-line {
        background-color: rgba(255, 0, 0, 0.1) !important;
    }
    .error-glyph {
        background-color: #f48771;
        width: 4px !important;
    }
`;
document.head.appendChild(style);

// Event listeners
document.addEventListener('DOMContentLoaded', async () => {
    initMonaco();
    await initWasm();
    
    document.getElementById('runBtn').addEventListener('click', runCode);
    document.getElementById('checkBtn').addEventListener('click', checkSyntax);
    document.getElementById('clearBtn').addEventListener('click', clearEditor);
    document.getElementById('clearOutputBtn').addEventListener('click', clearOutput);
    document.getElementById('exampleSelect').addEventListener('change', loadExample);
    document.getElementById('themeBtn').addEventListener('click', toggleTheme);
    
    // Keyboard shortcut: Ctrl+Enter to run
    if (editor) {
        editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, runCode);
    }
});

