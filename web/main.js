import init, { PseudocodeEngine } from './pkg/pseudocode_wasm.js';
import { PseudocodeLanguageService } from './language-service.js';

let engine = null;
let editor = null;
let errorDecorations = [];
let languageService = null;

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
            provideCompletionItems: (model, position) => {
                if (!languageService) {
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

                return { suggestions: items };
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
            wordWrap: 'on'
        });
        console.log('Monaco Editor initialized successfully');
    });
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
    
    try {
        // Get all INPUT statements from the code
        const inputVars = engine.get_input_statements(code);
        const inputVarsArray = Array.isArray(inputVars) ? inputVars : [];
        
        if (inputVarsArray.length > 0) {
            // Show input modal to collect values
            showInputModal(inputVarsArray, () => {
                // Callback when inputs are submitted - execute the code
                executeCodeWithInputs(code);
            });
        } else {
            // No inputs needed, execute directly
            executeCodeWithInputs(code);
        }
    } catch (error) {
        showOutput(`Error: ${error.message}`, 'error');
        console.error('Execution error:', error);
    }
}

// Execute code with inputs already in queue
function executeCodeWithInputs(code) {
    showOutput('Running...', 'info');
    
    try {
        const result = engine.execute(code);
        // result is already a JavaScript object (JsValue), not a JSON string
        const executionResult = result;
        
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
        // result is already a JavaScript object (JsValue), not a JSON string
        const checkResult = result;
        
        if (checkResult.valid) {
            showOutput('Syntax check passed!', 'success');
        } else {
            let errorText = 'Syntax errors found:\n';
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
        themeBtn.textContent = 'Dark';
    } else {
        body.classList.add('light');
        if (editor) {
            monaco.editor.setTheme('pseudocode-light');
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



