import init, { PseudocodeEngine } from './pkg/pseudocode_wasm.js';
import { PseudocodeLanguageService } from './language-service.js';

let engine = null;
let editor = null;
let errorDecorations = [];
let languageService = null;
let terminal = null;
let isExecuting = false;

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
        console.log('Monaco Editor loaded');
        // Register pseudocode language
        monaco.languages.register({ id: 'pseudocode' });
        
        // Register completion item provider for autocomplete
        monaco.languages.registerCompletionItemProvider('pseudocode', {
            provideCompletionItems: (model, position, context) => {
                if (!languageService) {
                    console.log('Autocomplete: languageService not initialized');
                    return { suggestions: [] };
                }

                const word = model.getWordUntilPosition(position);
                const range = {
                    startLineNumber: position.lineNumber,
                    endLineNumber: position.lineNumber,
                    startColumn: word.startColumn,
                    endColumn: word.endColumn
                };

                const code = model.getValue();
                const prefix = word.word;
                
                console.log(`Autocomplete triggered: prefix="${prefix}", triggerKind=${context.triggerKind}, column=${position.column}`);
                
                // Always try to get suggestions - let the language service decide what to return
                // (This ensures autocomplete works even on the first character typed)
                const suggestions = languageService.getSuggestions(
                    code,
                    position.lineNumber,
                    position.column,
                    prefix
                );

                // Convert to Monaco completion items
                const items = suggestions.map(suggestion => ({
                    label: suggestion.label,
                    kind: mapSuggestionKindToMonaco(suggestion.kind),
                    detail: suggestion.detail,
                    documentation: typeof suggestion.documentation === 'string' 
                        ? { value: suggestion.documentation }
                        : suggestion.documentation,
                    insertText: suggestion.insertText || suggestion.label,
                    range: range,
                    insertTextRules: suggestion.insertText && suggestion.insertText.endsWith('(')
                        ? monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet
                        : undefined
                }));

                // Debug logging
                if (items.length > 0) {
                    console.log(`Autocomplete: Returning ${items.length} items for prefix "${prefix}"`);
                    console.log('First 5 items:', items.slice(0, 5).map(i => i.label));
                } else {
                    console.log(`Autocomplete: No items for prefix "${prefix}"`);
                }

                return { 
                    suggestions: items,
                    incomplete: false  // Tell Monaco we've provided all suggestions
                };
            },
            triggerCharacters: [' ', ':', '(', '<']
        });

        // Helper to map our suggestion kinds to Monaco kinds
        function mapSuggestionKindToMonaco(kind) {
            const kindMap = {
                'keyword': monaco.languages.CompletionItemKind.Keyword,
                'function': monaco.languages.CompletionItemKind.Function,
                'variable': monaco.languages.CompletionItemKind.Variable,
                'constant': monaco.languages.CompletionItemKind.Constant,
                'type': monaco.languages.CompletionItemKind.TypeParameter
            };
            return kindMap[kind] || monaco.languages.CompletionItemKind.Text;
        }

        // Register hover provider
        monaco.languages.registerHoverProvider('pseudocode', {
            provideHover: (model, position) => {
                if (!languageService) {
                    return null;
                }

                const code = model.getValue();
                const hoverInfo = languageService.getHoverInfo(
                    code,
                    position.lineNumber,
                    position.column
                );

                if (hoverInfo) {
                    return {
                        range: new monaco.Range(
                            position.lineNumber,
                            position.column,
                            position.lineNumber,
                            position.column
                        ),
                        contents: hoverInfo.contents.map(item => ({
                            value: item.value
                        }))
                    };
                }

                return null;
            }
        });
        
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
            ],
            colors: {
                'editor.foreground': '#d4d4d4',
                'editor.background': '#1e1e1e'
            }
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
            ],
            colors: {
                'editor.foreground': '#000000',
                'editor.background': '#ffffff'
            }
        });

        // Create editor
        const editorElement = document.getElementById('editor');
        if (!editorElement) {
            console.error('Editor element not found!');
            return;
        }
        editor = monaco.editor.create(editorElement, {
            value: examples.simple,
            language: 'pseudocode',
            theme: 'pseudocode-dark',
            automaticLayout: true,
            fontSize: 14,
            fontFamily: "'Consolas', 'Monaco', 'Courier New', monospace",
            minimap: { enabled: true },
            scrollBeyondLastLine: false,
            wordWrap: 'on',
            quickSuggestions: {
                other: true,
                comments: false,
                strings: false
            },
            suggestOnTriggerCharacters: true,
            acceptSuggestionOnCommitCharacter: true,
            tabCompletion: 'on',
            wordBasedSuggestions: false,  // Disable built-in word-based suggestions
            wordBasedSuggestionsOnlySameLanguage: false,  // Don't use words from other files
            suggest: {
                showKeywords: true,
                showSnippets: false,
                showWords: false  // Explicitly disable word-based suggestions
            },
            quickSuggestionsDelay: 10  // Lower delay for faster autocomplete
        });
        console.log('Monaco Editor initialized successfully');
    });
}

// Initialize Terminal
function initTerminal() {
    const terminalElement = document.getElementById('terminal');
    if (!terminalElement) {
        console.error('Terminal element not found!');
        return;
    }

    // Access Terminal from window object to avoid AMD conflicts
    // xterm.js from CDN exposes Terminal on window
    if (typeof window.Terminal === 'undefined') {
        console.error('Terminal class not found. Make sure xterm.js is loaded before this script.');
        return;
    }
    
    terminal = new window.Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: "'Consolas', 'Monaco', 'Courier New', monospace",
        theme: {
            background: '#1e1e1e',
            foreground: '#d4d4d4',
            cursor: '#d4d4d4',
            selection: 'rgba(255, 255, 255, 0.3)',
            black: '#000000',
            red: '#f48771',
            green: '#89d185',
            yellow: '#d7ba7d',
            blue: '#569cd6',
            magenta: '#c586c0',
            cyan: '#4ec9b0',
            white: '#d4d4d4',
            brightBlack: '#808080',
            brightRed: '#f48771',
            brightGreen: '#89d185',
            brightYellow: '#d7ba7d',
            brightBlue: '#569cd6',
            brightMagenta: '#c586c0',
            brightCyan: '#4ec9b0',
            brightWhite: '#ffffff'
        }
    });

    // Try to use FitAddon if available, otherwise terminal works without it
    let fitAddon = null;
    
    // Check various possible export patterns for FitAddon
    if (typeof window.FitAddon !== 'undefined') {
        let FitAddonClass = window.FitAddon;
        
        // Try FitAddon.FitAddon pattern (common with script tag loading)
        if (typeof FitAddonClass === 'object' && typeof FitAddonClass.FitAddon === 'function') {
            FitAddonClass = FitAddonClass.FitAddon;
        }
        
        // Try direct function
        if (typeof FitAddonClass === 'function') {
            try {
                fitAddon = new FitAddonClass();
                terminal.loadAddon(fitAddon);
            } catch (e) {
                console.warn('Failed to initialize FitAddon, terminal will work without auto-resize:', e);
                fitAddon = null;
            }
        }
    }
    
    // If FitAddon not available, terminal will still work fine
    
    terminal.open(terminalElement);
    
    // Handle window resize - use FitAddon if available, otherwise do nothing
    // (xterm.js handles basic resizing automatically)
    if (fitAddon) {
        fitAddon.fit();
        window.addEventListener('resize', () => {
            fitAddon.fit();
        });
    }

    terminal.writeln('Pseudocode Terminal Ready');
    terminal.writeln('Type your code and press "Run" to execute.\r\n');
}

// Initialize WASM
async function initWasm() {
    try {
        await init();
        engine = new PseudocodeEngine();
        languageService = new PseudocodeLanguageService(engine);
        console.log('WASM initialized successfully');
    } catch (error) {
        console.error('Failed to initialize WASM:', error);
        if (terminal) {
            terminal.writeln(`\x1b[31mError: Failed to load WASM module. Make sure you have built the WASM package.\x1b[0m`);
        }
    }
}

// Show output in terminal
function showOutput(text, type = 'info') {
    if (!terminal) return;
    
    // Clear terminal if starting new execution
    if (type === 'info' && text === 'Running...') {
        terminal.clear();
        terminal.writeln('Executing program...\r\n');
        return;
    }
    
    if (text) {
        const lines = text.split('\n');
        lines.forEach((line, index) => {
            if (line.trim() || index < lines.length - 1) {
                if (type === 'error') {
                    terminal.writeln(`\x1b[31m${line}\x1b[0m`);
                } else if (type === 'success') {
                    terminal.writeln(line);
                } else {
                    terminal.writeln(line);
                }
            }
        });
    }
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

// Prompt for input in terminal (returns a Promise)
function promptInput(promptText) {
    return new Promise((resolve) => {
        if (!terminal) {
            resolve('');
            return;
        }
        
        // Only write prompt if provided (empty string = no prompt)
        if (promptText) {
            terminal.write(`\r\n\x1b[33m${promptText}\x1b[0m `);
        }
        
        let inputBuffer = '';
        let isPrompting = true;
        let disposable = null;
        
        const cleanup = () => {
            isPrompting = false;
            if (disposable && typeof disposable.dispose === 'function') {
                disposable.dispose();
                disposable = null;
            }
        };
        
        const dataHandler = (data) => {
            if (!isPrompting) return;
            
            // Handle Enter key
            if (data === '\r' || data === '\n' || data === '\r\n') {
                terminal.write('\r\n');
                cleanup();
                resolve(inputBuffer);
                return;
            }
            
            // Handle Backspace
            if (data === '\x7f' || data === '\b') {
                if (inputBuffer.length > 0) {
                    inputBuffer = inputBuffer.slice(0, -1);
                    terminal.write('\b \b');
                }
                return;
            }
            
            // Handle Ctrl+C
            if (data === '\x03') {
                terminal.write('^C\r\n');
                cleanup();
                resolve('');
                return;
            }
            
            // Add character to buffer and echo to terminal
            if (data.length === 1 && data >= ' ') {
                inputBuffer += data;
                terminal.write(data);
            }
        };
        
        // onData returns an IDisposable object
        disposable = terminal.onData(dataHandler);
    });
}

// Run code with interactive terminal input (line-by-line execution)
async function runCode() {
    if (isExecuting) {
        if (terminal) {
            terminal.writeln('\r\n\x1b[31mExecution already in progress...\x1b[0m');
        }
        return;
    }
    
    if (!engine) {
        if (terminal) {
            terminal.writeln('\x1b[31mError: WASM not initialized\x1b[0m');
        }
        return;
    }
    
    const code = editor.getValue();
    if (!code.trim()) {
        if (terminal) {
            terminal.writeln('\x1b[33mNo code to execute\x1b[0m');
        }
        return;
    }
    
    isExecuting = true;
    clearErrorDecorations();
    
    if (terminal) {
        terminal.clear();
    }
    
    try {
        // Parse the code for step-by-step execution
        const parseResult = engine.parse_for_execution(code);
        if (!parseResult || !parseResult.valid) {
            const errors = parseResult?.errors || [];
            if (terminal && errors.length > 0) {
                terminal.writeln('\x1b[31mParse errors:\x1b[0m');
                errors.forEach(error => {
                    terminal.writeln(`\x1b[31mLine ${error.line}: ${error.message}\x1b[0m`);
                });
            }
            highlightErrors(errors);
            return;
        }
        
        // Execute statements one by one
        while (engine.has_more_statements()) {
            // Check if next statement is INPUT
            const stmtInfo = engine.get_next_statement_info();
            
            if (stmtInfo.is_input && stmtInfo.input_var_name) {
                // Pause and get input (no prompt text - true terminal style)
                const inputValue = await promptInput('');
                engine.clear_inputs();
                engine.add_input(inputValue);
            }
            
            // Execute the next statement
            const result = engine.execute_next_statement();
            
            // Display any output immediately
            if (result.output) {
                // Split by newlines and write each line properly
                const lines = result.output.split('\n');
                for (let i = 0; i < lines.length; i++) {
                    if (i < lines.length - 1 || lines[i].length > 0) {
                        // Use writeln for lines (handles newlines properly)
                        terminal.writeln(lines[i]);
                    }
                }
            }
            
            // Check for errors
            if (result.errors && result.errors.length > 0) {
                terminal.writeln('\r\n\x1b[31m--- Errors ---\x1b[0m');
                result.errors.forEach(error => {
                    terminal.writeln(`\x1b[31mLine ${error.line}: ${error.message}\x1b[0m`);
                });
                highlightErrors(result.errors);
                break; // Stop execution on error
            }
        }
        
        terminal.writeln('\r\n\x1b[32mProgram execution complete.\x1b[0m\r\n');
    } catch (error) {
        if (terminal) {
            terminal.writeln(`\x1b[31mError: ${error.message}\x1b[0m`);
        }
        console.error('Execution error:', error);
    } finally {
        isExecuting = false;
    }
}

// Show input modal and collect inputs
function showInputModal(inputVars, onSubmit) {
    const modal = document.getElementById('inputModal');
    const inputFields = document.getElementById('inputFields');
    const submitBtn = document.getElementById('submitInputsBtn');
    const cancelBtn = document.getElementById('cancelInputsBtn');
    
    // Clear previous inputs
    inputFields.innerHTML = '';
    
    // Create input fields for each variable
    inputVars.forEach((varName, index) => {
        const inputField = document.createElement('div');
        inputField.className = 'input-field';
        
        const label = document.createElement('label');
        label.textContent = `${varName}:`;
        label.setAttribute('for', `input_${index}`);
        
        const input = document.createElement('input');
        input.type = 'text';
        input.id = `input_${index}`;
        input.name = varName;
        input.placeholder = `Enter value for ${varName}`;
        
        // Handle Enter key to submit
        input.addEventListener('keypress', (e) => {
            if (e.key === 'Enter' && index === inputVars.length - 1) {
                submitBtn.click();
            } else if (e.key === 'Enter') {
                const nextInput = document.getElementById(`input_${index + 1}`);
                if (nextInput) nextInput.focus();
            }
        });
        
        inputField.appendChild(label);
        inputField.appendChild(input);
        inputFields.appendChild(inputField);
    });
    
    // Show modal
    modal.classList.add('show');
    
    // Focus first input
    const firstInput = document.getElementById('input_0');
    if (firstInput) {
        setTimeout(() => firstInput.focus(), 100);
    }
    
    // Submit handler
    let submitHandler;
    let cancelHandler;
    
    submitHandler = () => {
        const inputs = inputVars.map(varName => {
            const input = Array.from(inputFields.querySelectorAll('input')).find(
                inp => inp.name === varName
            );
            return input ? input.value : '';
        });
        
        // Clear any previous inputs
        engine.clear_inputs();
        
        // Add all inputs to the queue (in reverse order since queue uses LIFO)
        // So when INPUT statements execute in order, they pop in the correct order
        for (let i = inputs.length - 1; i >= 0; i--) {
            engine.add_input(inputs[i]);
        }
        
        // Hide modal
        modal.classList.remove('show');
        
        // Remove event listeners
        submitBtn.removeEventListener('click', submitHandler);
        cancelBtn.removeEventListener('click', cancelHandler);
        
        // Execute callback
        onSubmit();
    };
    
    // Cancel handler
    cancelHandler = () => {
        // Hide modal without executing
        modal.classList.remove('show');
        
        // Remove event listeners
        submitBtn.removeEventListener('click', submitHandler);
        cancelBtn.removeEventListener('click', cancelHandler);
    };
    
    // Add event listeners
    submitBtn.addEventListener('click', submitHandler);
    cancelBtn.addEventListener('click', cancelHandler);
}

// Check syntax
async function checkSyntax() {
    if (!engine) {
        if (terminal) {
            terminal.writeln('\x1b[31mError: WASM not initialized\x1b[0m');
        }
        return;
    }
    
    const code = editor.getValue();
    if (!code.trim()) {
        if (terminal) {
            terminal.writeln('\x1b[33mNo code to check\x1b[0m');
        }
        return;
    }
    
    clearErrorDecorations();
    
    try {
        const result = engine.check_syntax(code);
        const checkResult = result;
        
        if (terminal) {
            if (checkResult.valid) {
                terminal.writeln('\x1b[32mSyntax check passed!\x1b[0m');
            } else {
                terminal.writeln('\x1b[31mSyntax errors found:\x1b[0m');
                checkResult.errors.forEach(error => {
                    terminal.writeln(`\x1b[31mLine ${error.line}: ${error.message}\x1b[0m`);
                });
                highlightErrors(checkResult.errors);
            }
        }
    } catch (error) {
        if (terminal) {
            terminal.writeln(`\x1b[31mError: ${error.message}\x1b[0m`);
        }
        console.error('Syntax check error:', error);
    }
}

// Clear editor
function clearEditor() {
    if (editor) {
        editor.setValue('');
        clearErrorDecorations();
    }
    if (terminal) {
        terminal.clear();
        terminal.writeln('Pseudocode Terminal Ready');
        terminal.writeln('Type your code and press "Run" to execute.\r\n');
    }
}

// Clear output
function clearOutput() {
    if (terminal) {
        terminal.clear();
        terminal.writeln('Pseudocode Terminal Ready');
        terminal.writeln('Type your code and press "Run" to execute.\r\n');
    }
    clearErrorDecorations();
}

// Load example
function loadExample() {
    const select = document.getElementById('exampleSelect');
    const exampleName = select.value;
    if (exampleName && examples[exampleName] && editor) {
        editor.setValue(examples[exampleName]);
        clearErrorDecorations();
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
        if (terminal) {
            terminal.options.theme = {
                background: '#1e1e1e',
                foreground: '#d4d4d4',
                cursor: '#d4d4d4',
                selection: 'rgba(255, 255, 255, 0.3)',
                black: '#000000',
                red: '#f48771',
                green: '#89d185',
                yellow: '#d7ba7d',
                blue: '#569cd6',
                magenta: '#c586c0',
                cyan: '#4ec9b0',
                white: '#d4d4d4',
                brightBlack: '#808080',
                brightRed: '#f48771',
                brightGreen: '#89d185',
                brightYellow: '#d7ba7d',
                brightBlue: '#569cd6',
                brightMagenta: '#c586c0',
                brightCyan: '#4ec9b0',
                brightWhite: '#ffffff'
            };
        }
        themeBtn.textContent = 'Dark';
    } else {
        body.classList.add('light');
        if (editor) {
            monaco.editor.setTheme('pseudocode-light');
        }
        if (terminal) {
            terminal.options.theme = {
                background: '#ffffff',
                foreground: '#1e1e1e',
                cursor: '#1e1e1e',
                selection: 'rgba(0, 0, 0, 0.3)',
                black: '#000000',
                red: '#a31515',
                green: '#098658',
                yellow: '#d7ba7d',
                blue: '#0000ff',
                magenta: '#811f3f',
                cyan: '#267f99',
                white: '#1e1e1e',
                brightBlack: '#808080',
                brightRed: '#a31515',
                brightGreen: '#098658',
                brightYellow: '#d7ba7d',
                brightBlue: '#0000ff',
                brightMagenta: '#811f3f',
                brightCyan: '#267f99',
                brightWhite: '#000000'
            };
        }
        themeBtn.textContent = 'Light';
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
    initTerminal();
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



