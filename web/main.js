import init, { PseudocodeEngine } from './pkg/pseudocode_wasm.js';
import { PseudocodeLanguageService } from './language-service.js';

let engine = null;
let editor = null;
let errorDecorations = [];
let languageService = null;
let terminal = null;
let isExecuting = false;

// Terminal theme constants
const TERMINAL_THEMES = {
    dark: {
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
    },
    light: {
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
    }
};

// Snippet keywords (used for filtering autocomplete)
const SNIPPET_KEYWORDS = ['IF', 'WHILE', 'FOR', 'REPEAT', 'CASE', 'FUNCTION', 'PROCEDURE', 'DECLARE'];

// Helper to write to terminal safely
function termWrite(text, color = null) {
    if (!terminal) return;
    if (color) {
        terminal.writeln(`\x1b[${color}m${text}\x1b[0m`);
    } else {
        terminal.writeln(text);
    }
}

// Helper to get FitAddon class
function getFitAddonClass() {
    if (typeof window.FitAddon === 'undefined') return null;
    let FitAddonClass = window.FitAddon;
    if (typeof FitAddonClass === 'object' && typeof FitAddonClass.FitAddon === 'function') {
        FitAddonClass = FitAddonClass.FitAddon;
    }
    return typeof FitAddonClass === 'function' ? FitAddonClass : null;
}

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
OUTPUT "Sum of ", x, " and ", y, " is ", sum`,
    string_manipulation: `// String Manipulation Examples
// Demonstrates: LENGTH, UPPER, LOWER, LEFT, RIGHT, SUBSTRING, MID

DECLARE text <- "Hello, Pseudocode World!": STRING
DECLARE len : INTEGER
DECLARE upperText : STRING
DECLARE lowerText : STRING
DECLARE leftPart : STRING
DECLARE rightPart : STRING
DECLARE middlePart : STRING

OUTPUT "Original text: ", text

// Get string length
len <- LENGTH(text)
OUTPUT "Length: ", len

// Convert to uppercase
upperText <- UPPER(text)
OUTPUT "Uppercase: ", upperText

// Convert to lowercase
lowerText <- LOWER(text)
OUTPUT "Lowercase: ", lowerText

// Extract left part (first 5 characters)
leftPart <- LEFT(text, 5)
OUTPUT "Left 5 chars: ", leftPart

// Extract right part (last 6 characters)
rightPart <- RIGHT(text, 6)
OUTPUT "Right 6 chars: ", rightPart

// Extract middle part using SUBSTRING
middlePart <- SUBSTRING(text, 8, 10)
OUTPUT "Substring from position 8, length 10: ", middlePart

// Extract middle part using MID
middlePart <- MID(text, 8, 10)
OUTPUT "MID from position 8, length 10: ", middlePart

OUTPUT ""
OUTPUT "String manipulation complete!"`,
    case_statement: `// CASE Statement Example
// Demonstrates: CASE, OTHERWISE, multiple value matching

DECLARE grade <- 85: INTEGER
DECLARE message : STRING

OUTPUT "Grade Calculator"
OUTPUT "Grade entered: ", grade

CASE grade OF
    90 TO 100 : 
        message <- "Excellent! Grade A"
    80 TO 89 : 
        message <- "Good! Grade B"
    70 TO 79 : 
        message <- "Average! Grade C"
    60 TO 69 : 
        message <- "Below Average! Grade D"
    0 TO 59 : 
        message <- "Fail! Grade F"
    OTHERWISE : 
        message <- "Invalid grade entered"
ENDCASE

OUTPUT message

// Another example with character matching
DECLARE choice <- "B": STRING
OUTPUT ""
OUTPUT "Menu System"
OUTPUT "Choice entered: ", choice

CASE UPPER(choice) OF
    "A" : 
        OUTPUT "Option A selected"
    "B" : 
        OUTPUT "Option B selected"
    "C" : 
        OUTPUT "Option C selected"
    "Q" : 
        OUTPUT "Quitting..."
    OTHERWISE : 
        OUTPUT "Invalid choice"
ENDCASE`,
    repeat_until: `// REPEAT-UNTIL Loop Example
// Demonstrates: REPEAT-UNTIL, input validation, loop control

DECLARE number <- 42: INTEGER
DECLARE validInput : BOOLEAN

OUTPUT "Number Guessing Game"
OUTPUT "I'm thinking of a number between 1 and 100"
OUTPUT ""

// REPEAT-UNTIL loop for input validation
validInput <- FALSE
REPEAT
    OUTPUT "Enter a number between 1 and 100: ", number
    
    IF number >= 1 AND number <= 100 THEN
        validInput <- TRUE
        OUTPUT "Valid number entered: ", number
    ELSE
        OUTPUT "Invalid! Number must be between 1 and 100."
    ENDIF
UNTIL validInput

// Another example: Countdown timer
DECLARE countdown : INTEGER
countdown <- 5

OUTPUT ""
OUTPUT "Countdown:"
REPEAT
    OUTPUT countdown
    countdown <- countdown - 1
UNTIL countdown < 0

OUTPUT "Blast off!"`,
    procedure_example: `// PROCEDURE Example
// Demonstrates: PROCEDURE, ENDPROCEDURE, procedures without return values

DECLARE num1 <- 10: INTEGER
DECLARE num2 <- 5: INTEGER

// Procedure to display a greeting
PROCEDURE DisplayGreeting(name : STRING)
    OUTPUT "Hello, ", name, "!"
    OUTPUT "Welcome to Pseudocode!"
ENDPROCEDURE

// Procedure to swap two numbers
PROCEDURE SwapNumbers(VAR a : INTEGER, VAR b : INTEGER)
    DECLARE temp : INTEGER
    temp <- a
    a <- b
    b <- temp
ENDPROCEDURE

// Procedure to display calculation results
PROCEDURE DisplayCalculation(x : INTEGER, y : INTEGER, operation : STRING)
    DECLARE result : INTEGER
    
    IF operation = "add" THEN
        result <- x + y
        OUTPUT x, " + ", y, " = ", result
    ELSE IF operation = "subtract" THEN
        result <- x - y
        OUTPUT x, " - ", y, " = ", result
    ELSE IF operation = "multiply" THEN
        result <- x * y
        OUTPUT x, " * ", y, " = ", result
    ELSE
        OUTPUT "Unknown operation"
    ENDIF
ENDPROCEDURE

// Main program
OUTPUT "=== Procedure Examples ==="
OUTPUT ""

// Call greeting procedure
CALL DisplayGreeting("Alice")

OUTPUT ""
OUTPUT "Before swap:"
OUTPUT "num1 = ", num1
OUTPUT "num2 = ", num2

// Call swap procedure
CALL SwapNumbers(num1, num2)

OUTPUT ""
OUTPUT "After swap:"
OUTPUT "num1 = ", num1
OUTPUT "num2 = ", num2

OUTPUT ""
OUTPUT "Calculations:"
CALL DisplayCalculation(15, 3, "add")
CALL DisplayCalculation(15, 3, "subtract")
CALL DisplayCalculation(15, 3, "multiply")`,
    character_functions: `// Character Functions Example
// Demonstrates: ASC, CHR, character code conversion

DECLARE char : STRING
DECLARE asciiCode : INTEGER
DECLARE convertedChar : STRING

OUTPUT "Character Code Conversion"
OUTPUT ""

// Convert character to ASCII code
char <- "A"
asciiCode <- ASC(char)
OUTPUT "Character '", char, "' has ASCII code: ", asciiCode

char <- "a"
asciiCode <- ASC(char)
OUTPUT "Character '", char, "' has ASCII code: ", asciiCode

char <- "5"
asciiCode <- ASC(char)
OUTPUT "Character '", char, "' has ASCII code: ", asciiCode

OUTPUT ""

// Convert ASCII code to character
asciiCode <- 65
convertedChar <- CHR(asciiCode)
OUTPUT "ASCII code ", asciiCode, " is character: '", convertedChar, "'"

asciiCode <- 97
convertedChar <- CHR(asciiCode)
OUTPUT "ASCII code ", asciiCode, " is character: '", convertedChar, "'"

asciiCode <- 48
convertedChar <- CHR(asciiCode)
OUTPUT "ASCII code ", asciiCode, " is character: '", convertedChar, "'"

OUTPUT ""

// Example: Convert string to ASCII codes
DECLARE text <- "Hi": STRING
DECLARE i : INTEGER
DECLARE currentChar : STRING

OUTPUT "Converting '", text, "' to ASCII codes:"
FOR i <- 1 TO LENGTH(text)
    currentChar <- SUBSTRING(text, i, 1)
    asciiCode <- ASC(currentChar)
    OUTPUT "  '", currentChar, "' = ", asciiCode
NEXT i

OUTPUT ""

// Example: Build string from ASCII codes
OUTPUT "Building string from ASCII codes:"
DECLARE codes : ARRAY[1:5] OF INTEGER
codes[1] <- 72
codes[2] <- 101
codes[3] <- 108
codes[4] <- 108
codes[5] <- 111

DECLARE builtString <- "": STRING
FOR i <- 1 TO 5
    builtString <- builtString + CHR(codes[i])
NEXT i

OUTPUT "Result: '", builtString, "'"`,
    array_operations: `// Advanced Array Operations Example
// Demonstrates: 2D arrays, array manipulation, searching, finding max/min

DECLARE matrix : ARRAY[1:3, 1:3] OF INTEGER
DECLARE i : INTEGER
DECLARE j : INTEGER
DECLARE rowSum : INTEGER
DECLARE colSum : INTEGER
DECLARE totalSum <- 0: INTEGER

// Initialize 2D array
OUTPUT "Initializing 3x3 matrix:"
FOR i <- 1 TO 3
    FOR j <- 1 TO 3
        matrix[i, j] <- (i - 1) * 3 + j
        OUTPUT matrix[i, j], " "
    NEXT j
    OUTPUT ""
NEXT i

OUTPUT ""

// Calculate row sums
OUTPUT "Row sums:"
FOR i <- 1 TO 3
    rowSum <- 0
    FOR j <- 1 TO 3
        rowSum <- rowSum + matrix[i, j]
    NEXT j
    OUTPUT "Row ", i, " sum: ", rowSum
    totalSum <- totalSum + rowSum
NEXT i

OUTPUT ""

// Calculate column sums
OUTPUT "Column sums:"
FOR j <- 1 TO 3
    colSum <- 0
    FOR i <- 1 TO 3
        colSum <- colSum + matrix[i, j]
    NEXT i
    OUTPUT "Column ", j, " sum: ", colSum
NEXT j

OUTPUT ""
OUTPUT "Total sum: ", totalSum

// Find maximum value in array
DECLARE maxValue : INTEGER
DECLARE maxRow : INTEGER
DECLARE maxCol : INTEGER

maxValue <- matrix[1, 1]
maxRow <- 1
maxCol <- 1

FOR i <- 1 TO 3
    FOR j <- 1 TO 3
        IF matrix[i, j] > maxValue THEN
            maxValue <- matrix[i, j]
            maxRow <- i
            maxCol <- j
        ENDIF
    NEXT j
NEXT i

OUTPUT ""
OUTPUT "Maximum value: ", maxValue, " at position [", maxRow, ", ", maxCol, "]"

// Find minimum value in array
DECLARE minValue : INTEGER
DECLARE minRow : INTEGER
DECLARE minCol : INTEGER

minValue <- matrix[1, 1]
minRow <- 1
minCol <- 1

FOR i <- 1 TO 3
    FOR j <- 1 TO 3
        IF matrix[i, j] < minValue THEN
            minValue <- matrix[i, j]
            minRow <- i
            minCol <- j
        ENDIF
    NEXT j
NEXT i

OUTPUT "Minimum value: ", minValue, " at position [", minRow, ", ", minCol, "]"

// Search for a specific value
DECLARE searchValue <- 5: INTEGER
DECLARE found <- FALSE: BOOLEAN
DECLARE foundRow : INTEGER
DECLARE foundCol : INTEGER

OUTPUT ""
OUTPUT "Searching for value: ", searchValue
i <- 1
WHILE i <= 3 AND NOT found DO
    j <- 1
    WHILE j <= 3 AND NOT found DO
        IF matrix[i, j] = searchValue THEN
            found <- TRUE
            foundRow <- i
            foundCol <- j
        ENDIF
        j <- j + 1
    ENDWHILE
    i <- i + 1
ENDWHILE

IF found THEN
    OUTPUT "Value found at position [", foundRow, ", ", foundCol, "]"
ELSE
    OUTPUT "Value not found in matrix"
ENDIF`
};

// Initialize Monaco Editor
function initMonaco() {
    require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' } });
    
    require(['vs/editor/editor.main'], function() {
        console.log('Monaco Editor loaded');
        // Register pseudocode language
        monaco.languages.register({ id: 'pseudocode' });
        
        // Configure language for auto-closing brackets and quotes
        monaco.languages.setLanguageConfiguration('pseudocode', {
            brackets: [
                ['(', ')'],
                ['[', ']'],
                ['{', '}']
            ],
            autoClosingPairs: [
                { open: '(', close: ')' },
                { open: '[', close: ']' },
                { open: '{', close: '}' },
                { open: '"', close: '"' },
                { open: "'", close: "'" }
            ],
            surroundingPairs: [
                { open: '(', close: ')' },
                { open: '[', close: ']' },
                { open: '{', close: '}' },
                { open: '"', close: '"' },
                { open: "'", close: "'" }
            ]
        });
        
        // Register completion item provider for autocomplete
        monaco.languages.registerCompletionItemProvider('pseudocode', {
            provideCompletionItems: (model, position, context) => {
                // Don't show snippets when space is pressed
                if (context.triggerCharacter === ' ') {
                    // Return only language service suggestions, no snippets
                    const word = model.getWordUntilPosition(position);
                    const range = {
                        startLineNumber: position.lineNumber,
                        endLineNumber: position.lineNumber,
                        startColumn: word.startColumn,
                        endColumn: word.endColumn
                    };
                    const code = model.getValue();
                    const prefix = word.word;
                    
                    let languageServiceItems = [];
                    if (languageService) {
                        try {
                            const suggestions = languageService.getSuggestions(
                                code,
                                position.lineNumber,
                                position.column,
                                prefix
                            );
                            languageServiceItems = suggestions.map(suggestion => ({
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
                        } catch (error) {
                            console.error('Error getting language service suggestions:', error);
                        }
                    }
                    
                    return { 
                        suggestions: languageServiceItems,
                        incomplete: false
                    };
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
                const lineText = model.getLineContent(position.lineNumber);
                const beforeCursor = lineText.substring(0, position.column - 1);
                
                // Control flow statement snippets
                const controlFlowSnippets = [];
                const prefixUpper = prefix.toUpperCase();
                
                // IF statement snippet
                if (prefix.length === 0 || 'IF'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'IF',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'IF statement (snippet)',
                        documentation: 'IF condition THEN ... ENDIF',
                        insertText: 'IF ${1:condition} THEN\n    ${0:statement}\nENDIF',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0' // Sort before other suggestions
                    });
                }
                
                // WHILE statement snippet
                if (prefix.length === 0 || 'WHILE'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'WHILE',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'WHILE loop (snippet)',
                        documentation: 'WHILE condition DO ... ENDWHILE',
                        insertText: 'WHILE ${1:condition} DO\n    ${0:statement}\nENDWHILE',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0'
                    });
                }
                
                // FOR statement snippet
                if (prefix.length === 0 || 'FOR'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'FOR',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'FOR loop (snippet)',
                        documentation: 'FOR variable <- start TO end ... NEXT variable',
                        insertText: 'FOR ${1:variable} <- ${2:start} TO ${3:end}\n    ${0:statement}\nNEXT ${1:variable}',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0'
                    });
                }
                
                // REPEAT-UNTIL statement snippet
                if (prefix.length === 0 || 'REPEAT'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'REPEAT',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'REPEAT-UNTIL loop (snippet)',
                        documentation: 'REPEAT ... UNTIL condition',
                        insertText: 'REPEAT\n    ${0:statement}\nUNTIL ${1:condition}',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0'
                    });
                }
                
                // CASE statement snippet
                if (prefix.length === 0 || 'CASE'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'CASE',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'CASE statement (snippet)',
                        documentation: 'CASE expression OF ... ENDCASE',
                        insertText: 'CASE ${1:expression} OF\n    ${2:value} : ${3:statement}\n    OTHERWISE : ${4:statement}\nENDCASE',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0'
                    });
                }
                
                // FUNCTION snippet
                if (prefix.length === 0 || 'FUNCTION'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'FUNCTION',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'FUNCTION declaration (snippet)',
                        documentation: 'FUNCTION name(params) RETURNS type ... ENDFUNCTION',
                        insertText: 'FUNCTION ${1:name}(${2:param}) RETURNS ${3:TYPE}\n    ${0:statement}\nENDFUNCTION',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0'
                    });
                }
                
                // PROCEDURE snippet
                if (prefix.length === 0 || 'PROCEDURE'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'PROCEDURE',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'PROCEDURE declaration (snippet)',
                        documentation: 'PROCEDURE name(params) ... ENDPROCEDURE',
                        insertText: 'PROCEDURE ${1:name}(${2:param})\n    ${0:statement}\nENDPROCEDURE',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0'
                    });
                }
                
                // DECLARE snippet
                if (prefix.length === 0 || 'DECLARE'.startsWith(prefixUpper)) {
                    controlFlowSnippets.push({
                        label: 'DECLARE',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        detail: 'DECLARE variable (snippet)',
                        documentation: 'DECLARE variable : TYPE',
                        insertText: 'DECLARE ${1:variable} : ${2:TYPE}',
                        range: range,
                        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                        sortText: '0'
                    });
                }
                
                // Get language service suggestions
                let languageServiceItems = [];
                if (languageService) {
                    try {
                        const suggestions = languageService.getSuggestions(
                            code,
                            position.lineNumber,
                            position.column,
                            prefix
                        );

                        // Convert to Monaco completion items, filtering out snippet keywords
                        languageServiceItems = suggestions
                            .filter(s => !SNIPPET_KEYWORDS.includes(s.label.toUpperCase()))
                            .map(s => ({
                                label: s.label,
                                kind: mapSuggestionKindToMonaco(s.kind),
                                detail: s.detail,
                                documentation: typeof s.documentation === 'string' 
                                    ? { value: s.documentation }
                                    : s.documentation,
                                insertText: s.insertText || s.label,
                                range: range,
                                insertTextRules: s.insertText?.endsWith('(')
                                    ? monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet
                                    : undefined
                            }));
                    } catch (error) {
                        console.error('Error getting language service suggestions:', error);
                    }
                }
                
                // Combine snippets (first, so they appear at top) and language service suggestions
                const allSuggestions = [...controlFlowSnippets, ...languageServiceItems];

                return { 
                    suggestions: allSuggestions,
                    incomplete: false
                };
            },
            triggerCharacters: [':', '(', '<']  // Removed space to prevent triggering on space
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
                'TYPE', 'ENDTYPE', 'CASE', 'ENDCASE', 'OTHERWISE'
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
        
        // Load cached content or use example
        const cachedContent = localStorage.getItem('editorContent');
        const initialContent = cachedContent || examples.simple;
        
        editor = monaco.editor.create(editorElement, {
            value: initialContent,
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
            acceptSuggestionOnEnter: 'off',  // Disable Enter key from accepting autocomplete
            tabCompletion: 'on',
            wordBasedSuggestions: false,  // Disable built-in word-based suggestions
            wordBasedSuggestionsOnlySameLanguage: false,  // Don't use words from other files
            suggest: {
                showKeywords: true,
                showSnippets: true,  // Enable snippets for control flow statements
                showWords: false  // Explicitly disable word-based suggestions
            },
            quickSuggestionsDelay: 10,  // Lower delay for faster autocomplete
            // Auto-closing brackets and quotes
            autoClosingBrackets: 'languageDefined',  // Auto-close brackets: (), [], {}
            autoClosingQuotes: 'languageDefined',  // Auto-close quotes: "", ''
            autoSurround: 'languageDefined'  // Auto-surround selection with brackets/quotes
        });
        
        // Add Enter key handler for auto-indentation after THEN, DO, etc.
        editor.addCommand(monaco.KeyCode.Enter, function() {
            const position = editor.getPosition();
            if (!position) {
                return; // Fall through to default behavior
            }
            
            const model = editor.getModel();
            if (!model) {
                return;
            }
            
            const lineNumber = position.lineNumber;
            const lineText = model.getLineContent(lineNumber);
            const beforeCursor = lineText.substring(0, position.column - 1);
            
            // Keywords that should trigger auto-indent on next line
            const indentTriggerKeywords = ['THEN', 'DO', 'ELSE'];
            
            // Check if line ends with one of the trigger keywords (case-insensitive)
            const shouldIndent = indentTriggerKeywords.some(keyword => {
                const trimmed = beforeCursor.trim();
                const regex = new RegExp(`\\b${keyword}\\s*$`, 'i');
                return regex.test(trimmed);
            });
            
            if (shouldIndent) {
                // Get current indentation
                const indentMatch = lineText.match(/^(\s*)/);
                const currentIndent = indentMatch ? indentMatch[1] : '';
                const newIndent = currentIndent + '    '; // 4 spaces
                
                // Insert newline with indentation
                editor.executeEdits('auto-indent', [{
                    range: new monaco.Range(lineNumber, position.column, lineNumber, position.column),
                    text: '\n' + newIndent
                }]);
                
                // Move cursor to the indented position
                editor.setPosition(new monaco.Position(lineNumber + 1, newIndent.length + 1));
            } else {
                // Normal Enter behavior - preserve current line indentation
                const indentMatch = lineText.match(/^(\s*)/);
                const currentIndent = indentMatch ? indentMatch[1] : '';
                
                // Insert newline with preserved indentation
                editor.executeEdits('enter', [{
                    range: new monaco.Range(lineNumber, position.column, lineNumber, position.column),
                    text: '\n' + currentIndent
                }]);
                
                // Move cursor after the indentation
                editor.setPosition(new monaco.Position(lineNumber + 1, currentIndent.length + 1));
            }
        });
        
        // Save editor content to cache on change (debounced)
        let saveTimeout = null;
        editor.onDidChangeModelContent(() => {
            if (saveTimeout) {
                clearTimeout(saveTimeout);
            }
            saveTimeout = setTimeout(() => {
                const content = editor.getValue();
                localStorage.setItem('editorContent', content);
            }, 500); // Save 500ms after user stops typing
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
        scrollback: 10000,
        theme: TERMINAL_THEMES.dark
    });

    // Initialize FitAddon if available
    const FitAddonClass = getFitAddonClass();
    if (FitAddonClass) {
        try {
            terminal._fitAddon = new FitAddonClass();
            terminal.loadAddon(terminal._fitAddon);
        } catch (e) {
            console.warn('Failed to initialize FitAddon:', e);
        }
    }
    
    terminal.open(terminalElement);
    
    // Initial fit
    if (terminal._fitAddon) {
        terminal._fitAddon.fit();
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
        termWrite('\r\nExecution already in progress...', '31');
        return;
    }
    
    if (!engine) {
        termWrite('Error: WASM not initialized', '31');
        return;
    }
    
    const code = editor.getValue();
    if (!code.trim()) {
        termWrite('No code to execute', '33');
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
            if (errors.length > 0) {
                termWrite('Parse errors:', '31');
                errors.forEach(error => {
                    termWrite(`Line ${error.line}: ${error.message}`, '31');
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
                const inputValue = await promptInput('');
                engine.clear_inputs();
                engine.add_input(inputValue);
            }
            
            // Execute the next statement
            const result = engine.execute_next_statement();
            
            // Display any output immediately
            if (result.output) {
                const lines = result.output.split('\n');
                for (let i = 0; i < lines.length; i++) {
                    if (i < lines.length - 1 || lines[i].length > 0) {
                        terminal.writeln(lines[i]);
                    }
                }
            }
            
            // Check for errors
            if (result.errors && result.errors.length > 0) {
                termWrite('\r\n--- Errors ---', '31');
                result.errors.forEach(error => {
                    termWrite(`Line ${error.line}: ${error.message}`, '31');
                });
                highlightErrors(result.errors);
                break;
            }
        }
        
        termWrite('\r\nProgram execution complete.\r\n', '32');
    } catch (error) {
        termWrite(`Error: ${error.message}`, '31');
        console.error('Execution error:', error);
    } finally {
        isExecuting = false;
    }
}


// Check syntax
async function checkSyntax() {
    if (!engine) {
        termWrite('Error: WASM not initialized', '31');
        return;
    }
    
    const code = editor.getValue();
    if (!code.trim()) {
        termWrite('No code to check', '33');
        return;
    }
    
    clearErrorDecorations();
    
    try {
        const result = engine.check_syntax(code);
        if (result.valid) {
            termWrite('Syntax check passed!', '32');
        } else {
            termWrite('Syntax errors found:', '31');
            result.errors.forEach(error => {
                termWrite(`Line ${error.line}: ${error.message}`, '31');
            });
            highlightErrors(result.errors);
        }
    } catch (error) {
        termWrite(`Error: ${error.message}`, '31');
        console.error('Syntax check error:', error);
    }
}

// Clear editor
function clearEditor() {
    if (editor) {
        // Use executeEdits to preserve undo history (Ctrl+Z will restore content)
        const model = editor.getModel();
        if (model) {
            const fullRange = model.getFullModelRange();
            const currentContent = model.getValue();
            
            // Only clear if there's content to clear
            if (currentContent.length > 0) {
                editor.executeEdits('clear', [{
                    range: fullRange,
                    text: ''
                }]);
            }
        }
        clearErrorDecorations();
    }
    // Reset filename to untitled
    updateFilename('untitled');
    // Clear cached content and filename
    localStorage.removeItem('editorContent');
    localStorage.removeItem('editorFilename');
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
        // Save to cache
        localStorage.setItem('editorContent', examples[exampleName]);
    }
}

// Update filename display
function updateFilename(filename) {
    const filenameInput = document.getElementById('filenameInput');
    if (filenameInput) {
        // Remove .pseu extension if present
        const nameWithoutExt = filename.endsWith('.pseu') 
            ? filename.slice(0, -5) 
            : filename;
        filenameInput.value = nameWithoutExt;
        // Save to cache
        localStorage.setItem('editorFilename', nameWithoutExt);
    }
}

// Handle file selection
function handleFileSelect(event) {
    const file = event.target.files[0];
    if (!file) {
        return;
    }
    
    // Check file extension
    if (!file.name.endsWith('.pseu')) {
        termWrite('Error: Please select a .pseu file', '31');
        alert('Please select a .pseu file');
        event.target.value = '';
        return;
    }
    
    const reader = new FileReader();
    
    reader.onload = (e) => {
        try {
            const content = e.target.result;
            if (editor) {
                editor.setValue(content);
                clearErrorDecorations();
                updateFilename(file.name);
                localStorage.setItem('editorContent', content);
                termWrite(`Opened file: ${file.name}`, '32');
            }
        } catch (error) {
            console.error('Error reading file:', error);
            termWrite(`Error reading file: ${error.message}`, '31');
            alert(`Error reading file: ${error.message}`);
        }
        event.target.value = '';
    };
    
    reader.onerror = () => {
        console.error('Error reading file');
        termWrite('Error reading file', '31');
        alert('Error reading file');
        event.target.value = '';
    };
    
    reader.readAsText(file);
}

// Open file
function openFile() {
    const fileInput = document.getElementById('fileInput');
    if (!fileInput) {
        console.error('File input element not found');
        return;
    }
    
    // Trigger file selection
    fileInput.click();
}

// Download file using browser's native save dialog
async function downloadFile() {
    if (!editor) {
        console.error('Editor not initialized');
        return;
    }
    
    // Get current filename from display
    const filenameInput = document.getElementById('filenameInput');
    const currentFilename = filenameInput ? filenameInput.value.trim() || 'untitled' : 'untitled';
    const filenameWithExt = currentFilename.endsWith('.pseu') ? currentFilename : `${currentFilename}.pseu`;
    
    const content = editor.getValue();
    
    // Try to use File System Access API (shows native save dialog)
    if ('showSaveFilePicker' in window) {
        try {
            const fileHandle = await window.showSaveFilePicker({
                suggestedName: filenameWithExt,
                types: [{
                    description: 'Pseudocode files',
                    accept: {
                        'text/plain': ['.pseu'],
                    },
                }],
            });
            
            const writable = await fileHandle.createWritable();
            await writable.write(content);
            await writable.close();
            
            const savedName = fileHandle.name;
            updateFilename(savedName);
            termWrite(`File saved: ${savedName}`, '32');
        } catch (error) {
            if (error.name !== 'AbortError') {
                console.error('Error saving file:', error);
                termWrite(`Error saving file: ${error.message}`, '31');
            }
        }
    } else {
        // Fallback for browsers without File System Access API
        const blob = new Blob([content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filenameWithExt;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        termWrite(`File download initiated: ${filenameWithExt}`, '32');
    }
}

// Toggle theme
function toggleTheme() {
    const body = document.body;
    const themeBtn = document.getElementById('themeBtn');
    
    const isLight = body.classList.contains('light');
    body.classList.toggle('light');
    
    if (editor) {
        monaco.editor.setTheme(isLight ? 'pseudocode-dark' : 'pseudocode-light');
    }
    if (terminal) {
        terminal.options.theme = isLight ? TERMINAL_THEMES.dark : TERMINAL_THEMES.light;
    }
    themeBtn.textContent = isLight ? 'Dark' : 'Light';
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

// Resize terminal to fit its container
function resizeTerminal() {
    if (!terminal) return;
    
    // Initialize FitAddon if not already loaded
    if (!terminal._fitAddon) {
        const FitAddonClass = getFitAddonClass();
        if (FitAddonClass) {
            try {
                terminal._fitAddon = new FitAddonClass();
                terminal.loadAddon(terminal._fitAddon);
            } catch (e) {
                console.warn('Failed to initialize FitAddon:', e);
            }
        }
    }
    
    // Use FitAddon if available
    if (terminal._fitAddon && typeof terminal._fitAddon.fit === 'function') {
        requestAnimationFrame(() => {
            if (terminal?._fitAddon) {
                terminal._fitAddon.fit();
                setTimeout(() => {
                    if (terminal) {
                        terminal.refresh(0, terminal.rows - 1);
                        const viewport = terminal.element?.querySelector('.xterm-viewport');
                        if (viewport) viewport.style.overflowY = 'auto';
                    }
                }, 50);
            }
        });
        return;
    }
    
    // Fallback: manual resize
    requestAnimationFrame(() => {
        if (terminal?.resize && terminal.element) {
            const el = terminal.element;
            const cols = terminal.cols || 80;
            const lineHeight = parseFloat(getComputedStyle(el).lineHeight) || 14;
            const padding = (parseFloat(getComputedStyle(el).paddingTop) || 0) + 
                           (parseFloat(getComputedStyle(el).paddingBottom) || 0);
            const rows = Math.floor((el.clientHeight - padding) / lineHeight);
            if (rows > 0 && cols > 0) terminal.resize(cols, rows);
        }
    });
}

// Initialize resizer for terminal height
function initResizer() {
    const resizer = document.getElementById('resizer');
    const outputContainer = document.querySelector('.output-container');
    const container = document.querySelector('.container');
    
    if (!resizer || !outputContainer || !container) {
        return;
    }
    
    // Load saved height from localStorage
    const savedHeight = localStorage.getItem('terminalHeight');
    if (savedHeight) {
        const height = parseInt(savedHeight, 10);
        const headerHeight = document.querySelector('header')?.getBoundingClientRect().height || 0;
        const maxAllowedHeight = window.innerHeight - headerHeight - 100;
        if (height >= 200 && height <= maxAllowedHeight) {
            outputContainer.style.height = `${height}px`;
            // Resize terminal after setting height
            setTimeout(() => resizeTerminal(), 100);
        }
    }
    
    let isResizing = false;
    
    resizer.addEventListener('mousedown', (e) => {
        isResizing = true;
        resizer.classList.add('resizing');
        document.body.style.cursor = 'row-resize';
        document.body.style.userSelect = 'none';
        e.preventDefault();
    });
    
    document.addEventListener('mousemove', (e) => {
        if (!isResizing) return;
        
        const containerRect = container.getBoundingClientRect();
        const headerHeight = document.querySelector('header').getBoundingClientRect().height;
        const availableHeight = window.innerHeight - headerHeight;
        
        // Calculate new height from bottom of viewport (not container)
        const newHeight = window.innerHeight - e.clientY;
        
        // Constrain between min (200px) and max (availableHeight - 50px for editor)
        const minHeight = 200;
        const maxHeight = availableHeight - 50;
        
        if (newHeight >= minHeight && newHeight <= maxHeight) {
            outputContainer.style.height = `${newHeight}px`;
            // Resize terminal immediately during drag
            resizeTerminal();
            // Save to localStorage
            localStorage.setItem('terminalHeight', newHeight.toString());
        }
    });
    
    document.addEventListener('mouseup', () => {
        if (isResizing) {
            isResizing = false;
            resizer.classList.remove('resizing');
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
            // Final resize after drag ends
            resizeTerminal();
        }
    });
    
    // Also resize terminal on window resize
    window.addEventListener('resize', () => {
        resizeTerminal();
    });
}

// Event listeners
document.addEventListener('DOMContentLoaded', async () => {
    initTerminal();
    initMonaco();
    await initWasm();
    
    // Set up file input handler
    const fileInput = document.getElementById('fileInput');
    if (fileInput) {
        fileInput.addEventListener('change', handleFileSelect);
    }
    
    // Load cached filename if available
    const cachedFilename = localStorage.getItem('editorFilename');
    if (cachedFilename) {
        updateFilename(cachedFilename);
    }
    
    // Initialize resizer
    initResizer();
    
    // Save filename when it changes
    const filenameInput = document.getElementById('filenameInput');
    if (filenameInput) {
        filenameInput.addEventListener('change', () => {
            localStorage.setItem('editorFilename', filenameInput.value);
        });
        filenameInput.addEventListener('input', () => {
            localStorage.setItem('editorFilename', filenameInput.value);
        });
    }
    
    document.getElementById('openBtn').addEventListener('click', openFile);
    document.getElementById('downloadBtn').addEventListener('click', downloadFile);
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