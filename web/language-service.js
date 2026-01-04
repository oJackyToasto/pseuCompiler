/**
 * Pseudocode Language Service
 * 
 * This module provides language intelligence features (autocomplete, hover info, etc.)
 * that can be reused by both the web editor (Monaco) and a future LSP server.
 * 
 * LSP Integration Notes:
 * - The methods in this class are LSP-agnostic and return simple JavaScript objects
 * - For LSP integration, wrap the methods like this:
 *   - getSuggestions() -> completion provider
 *   - getHoverInfo() -> hover provider  
 *   - extractSymbols() -> document symbols provider
 * - The context analysis and symbol extraction logic can be reused as-is
 * - Convert the returned objects to LSP protocol format (TextDocumentCompletionItem, Hover, etc.)
 */

export class PseudocodeLanguageService {
    constructor() {
        this.keywords = [
            'DECLARE', 'CONSTANT', 'FUNCTION', 'PROCEDURE', 'ENDFUNCTION', 'ENDPROCEDURE',
            'IF', 'THEN', 'ELSE', 'ENDIF', 'WHILE', 'DO', 'ENDWHILE',
            'FOR', 'TO', 'NEXT', 'REPEAT', 'UNTIL',
            'RETURN', 'CALL', 'INPUT', 'OUTPUT',
            'OPENFILE', 'CLOSEFILE', 'READFILE', 'WRITEFILE', 'SEEK',
            'GETRECORD', 'PUTRECORD',
            'INTEGER', 'REAL', 'STRING', 'CHAR', 'BOOLEAN', 'ARRAY', 'OF',
            'AND', 'OR', 'NOT', 'TRUE', 'FALSE',
            'TYPE', 'ENDTYPE', 'CASE', 'ENDCASE', 'OTHERWISE', 'BREAK',
            'RETURNS'
        ];

        this.builtinFunctions = [
            { name: 'LENGTH', description: 'Returns the length of a string', params: ['string'] },
            { name: 'UPPER', description: 'Converts a string to uppercase', params: ['string'] },
            { name: 'LOWER', description: 'Converts a string to lowercase', params: ['string'] },
            { name: 'SUBSTRING', description: 'Extracts a substring from a string', params: ['string', 'start', 'length'] },
            { name: 'LEFT', description: 'Returns the leftmost characters of a string', params: ['string', 'count'] },
            { name: 'RIGHT', description: 'Returns the rightmost characters of a string', params: ['string', 'count'] },
            { name: 'MID', description: 'Extracts characters from the middle of a string', params: ['string', 'start', 'count'] },
            { name: 'ROUND', description: 'Rounds a number to the nearest integer', params: ['number', 'decimals'] },
            { name: 'RANDOM', description: 'Returns a random number between 0 and 1', params: [] },
            { name: 'EOF', description: 'Checks if end of file has been reached', params: ['file'] },
            { name: 'MOD', description: 'Returns the remainder of division', params: ['dividend', 'divisor'] }
        ];

        this.types = [
            'INTEGER', 'REAL', 'STRING', 'CHAR', 'BOOLEAN', 'ARRAY'
        ];

        this.operators = [
            '<-', '=', '<>', '<', '>', '<=', '>=', '+', '-', '*', '/', 'MOD'
        ];
    }

    /**
     * Analyze code context to determine what suggestions are appropriate
     * @param {string} code - The full code text
     * @param {number} line - Current line (1-based)
     * @param {number} column - Current column (1-based)
     * @returns {Object} Context information
     */
    analyzeContext(code, line, column) {
        const lines = code.split('\n');
        const currentLine = lines[line - 1] || '';
        const beforeCursor = currentLine.substring(0, column - 1);
        const afterCursor = currentLine.substring(column - 1);

        // Get all lines before current line for context
        const previousLines = lines.slice(0, line - 1).join('\n');
        const fullContext = previousLines + '\n' + beforeCursor;

        const context = {
            line: line,
            column: column,
            currentLine: currentLine,
            beforeCursor: beforeCursor.trim(),
            afterCursor: afterCursor.trim(),
            isStartOfLine: beforeCursor.trim().length === 0,
            previousLine: lines[line - 2] || '',
            previousLines: previousLines,
            fullContext: fullContext
        };

        // Detect context patterns
        context.afterDeclare = /DECLARE\s+[\w,]*\s*:\s*$/.test(beforeCursor);
        context.afterFunction = /FUNCTION\s+\w+\s*\([^)]*\)\s*RETURNS\s*$/.test(beforeCursor);
        context.inArrayDecl = /ARRAY\s*\[/.test(fullContext) && /:\s*ARRAY/.test(beforeCursor);
        context.inAssignment = /<-/.test(beforeCursor) && !afterCursor.trim().startsWith('<');
        context.afterIf = /IF\s+.+\s+(THEN|DO)\s*$/.test(beforeCursor);
        context.afterFor = /FOR\s+\w+\s*<-\s*.+\s+TO\s*$/.test(beforeCursor);
        context.inFunctionCall = /^\s*[A-Z_][A-Z0-9_]*\s*\(/.test(afterCursor) || /[A-Z_][A-Z0-9_]*\s*\(/.test(beforeCursor);

        return context;
    }

    /**
     * Extract variables, functions, and procedures from code
     * @param {string} code - The code text
     * @returns {Object} Extracted symbols
     */
    extractSymbols(code) {
        const symbols = {
            variables: [],
            constants: [],
            functions: [],
            procedures: [],
            types: []
        };

        // Extract DECLARE statements
        const declareRegex = /DECLARE\s+([\w,]+)\s*(?::\s*([\w\[\],:\s]+))?(?:<-\s*([^\n]+))?/gi;
        let match;
        while ((match = declareRegex.exec(code)) !== null) {
            const varNames = match[1].split(',').map(v => v.trim());
            const type = match[2] ? match[2].trim() : null;
            const initialValue = match[3] ? match[3].trim() : null;

            varNames.forEach(name => {
                symbols.variables.push({
                    name: name.trim(),
                    type: type,
                    line: code.substring(0, match.index).split('\n').length
                });
            });
        }

        // Extract CONSTANT declarations
        const constantRegex = /CONSTANT\s+([\w,]+)/gi;
        while ((match = constantRegex.exec(code)) !== null) {
            const names = match[1].split(',').map(v => v.trim());
            names.forEach(name => {
                symbols.constants.push({
                    name: name.trim(),
                    line: code.substring(0, match.index).split('\n').length
                });
            });
        }

        // Extract FUNCTION definitions
        const functionRegex = /FUNCTION\s+(\w+)\s*\(([^)]*)\)\s*(?:RETURNS\s+(\w+))?/gi;
        while ((match = functionRegex.exec(code)) !== null) {
            const params = match[2] ? match[2].split(',').map(p => {
                const parts = p.trim().split(':');
                return {
                    name: parts[0].trim(),
                    type: parts[1] ? parts[1].trim() : null
                };
            }) : [];
            
            symbols.functions.push({
                name: match[1],
                params: params,
                returnType: match[3] || null,
                line: code.substring(0, match.index).split('\n').length
            });
        }

        // Extract PROCEDURE definitions
        const procedureRegex = /PROCEDURE\s+(\w+)\s*\(([^)]*)\)/gi;
        while ((match = procedureRegex.exec(code)) !== null) {
            const params = match[2] ? match[2].split(',').map(p => {
                const parts = p.trim().split(':');
                return {
                    name: parts[0].trim(),
                    type: parts[1] ? parts[1].trim() : null
                };
            }) : [];
            
            symbols.procedures.push({
                name: match[1],
                params: params,
                line: code.substring(0, match.index).split('\n').length
            });
        }

        // Extract TYPE definitions
        const typeRegex = /TYPE\s+(\w+)/gi;
        while ((match = typeRegex.exec(code)) !== null) {
            symbols.types.push({
                name: match[1],
                line: code.substring(0, match.index).split('\n').length
            });
        }

        return symbols;
    }

    /**
     * Get autocomplete suggestions based on context
     * @param {string} code - The full code text
     * @param {number} line - Current line (1-based)
     * @param {number} column - Current column (1-based)
     * @param {string} prefix - The prefix to match (word being typed)
     * @returns {Array} Array of suggestion objects
     */
    getSuggestions(code, line, column, prefix = '') {
        const context = this.analyzeContext(code, line, column);
        const symbols = this.extractSymbols(code);
        const suggestions = [];
        const prefixLower = prefix.toLowerCase();

        // Helper to check if item matches prefix
        const matchesPrefix = (item) => {
            if (!prefix) return true;
            return item.toLowerCase().startsWith(prefixLower);
        };

        // After DECLARE, suggest types or variable names
        if (context.afterDeclare) {
            this.types.forEach(type => {
                if (matchesPrefix(type)) {
                    suggestions.push({
                        label: type,
                        kind: 'keyword',
                        detail: 'Type',
                        documentation: `Data type: ${type}`,
                        insertText: type
                    });
                }
            });
            // Also suggest ARRAY
            if (matchesPrefix('ARRAY')) {
                suggestions.push({
                    label: 'ARRAY',
                    kind: 'keyword',
                    detail: 'Type',
                    documentation: 'Array type declaration',
                    insertText: 'ARRAY'
                });
            }
        }
        // After FUNCTION/PROCEDURE name, suggest RETURNS
        else if (context.afterFunction) {
            this.types.forEach(type => {
                if (matchesPrefix(type)) {
                    suggestions.push({
                        label: type,
                        kind: 'keyword',
                        detail: 'Return Type',
                        documentation: `Return type: ${type}`,
                        insertText: type
                    });
                }
            });
        }
        // Start of line or after keyword - suggest keywords
        else if (context.isStartOfLine || /[^A-Za-z0-9_]$/.test(context.beforeCursor)) {
            // Suggest keywords
            this.keywords.forEach(keyword => {
                if (matchesPrefix(keyword)) {
                    suggestions.push({
                        label: keyword,
                        kind: 'keyword',
                        detail: 'Keyword',
                        documentation: this.getKeywordDocumentation(keyword),
                        insertText: keyword
                    });
                }
            });

            // Suggest functions and procedures
            symbols.functions.forEach(func => {
                if (matchesPrefix(func.name)) {
                    suggestions.push({
                        label: func.name,
                        kind: 'function',
                        detail: `Function${func.returnType ? ': ' + func.returnType : ''}`,
                        documentation: this.formatFunctionDocumentation(func),
                        insertText: func.name + '('
                    });
                }
            });

            symbols.procedures.forEach(proc => {
                if (matchesPrefix(proc.name)) {
                    suggestions.push({
                        label: proc.name,
                        kind: 'function',
                        detail: 'Procedure',
                        documentation: this.formatProcedureDocumentation(proc),
                        insertText: proc.name + '('
                    });
                }
            });
        }
        // In assignment or expression - suggest variables, functions, built-ins
        else {
            // Suggest variables
            symbols.variables.forEach(variable => {
                if (matchesPrefix(variable.name)) {
                    suggestions.push({
                        label: variable.name,
                        kind: 'variable',
                        detail: variable.type ? `Variable: ${variable.type}` : 'Variable',
                        documentation: `Variable: ${variable.name}`,
                        insertText: variable.name
                    });
                }
            });

            // Suggest constants
            symbols.constants.forEach(constant => {
                if (matchesPrefix(constant.name)) {
                    suggestions.push({
                        label: constant.name,
                        kind: 'constant',
                        detail: 'Constant',
                        documentation: `Constant: ${constant.name}`,
                        insertText: constant.name
                    });
                }
            });

            // Suggest built-in functions
            this.builtinFunctions.forEach(func => {
                if (matchesPrefix(func.name)) {
                    suggestions.push({
                        label: func.name,
                        kind: 'function',
                        detail: 'Built-in Function',
                        documentation: func.description,
                        insertText: func.name + '('
                    });
                }
            });

            // Suggest user-defined functions and procedures
            symbols.functions.forEach(func => {
                if (matchesPrefix(func.name)) {
                    suggestions.push({
                        label: func.name,
                        kind: 'function',
                        detail: `Function${func.returnType ? ': ' + func.returnType : ''}`,
                        documentation: this.formatFunctionDocumentation(func),
                        insertText: func.name + '('
                    });
                }
            });

            symbols.procedures.forEach(proc => {
                if (matchesPrefix(proc.name)) {
                    suggestions.push({
                        label: proc.name,
                        kind: 'function',
                        detail: 'Procedure',
                        documentation: this.formatProcedureDocumentation(proc),
                        insertText: proc.name + '('
                    });
                }
            });

            // Suggest keywords (for control flow, etc.)
            const contextKeywords = ['IF', 'WHILE', 'FOR', 'RETURN', 'OUTPUT', 'INPUT'];
            contextKeywords.forEach(keyword => {
                if (matchesPrefix(keyword)) {
                    suggestions.push({
                        label: keyword,
                        kind: 'keyword',
                        detail: 'Keyword',
                        documentation: this.getKeywordDocumentation(keyword),
                        insertText: keyword
                    });
                }
            });
        }

        return suggestions.sort((a, b) => a.label.localeCompare(b.label));
    }

    /**
     * Get documentation for a keyword
     */
    getKeywordDocumentation(keyword) {
        const docs = {
            'DECLARE': 'Declares a variable or array',
            'CONSTANT': 'Declares a constant value',
            'FUNCTION': 'Defines a function',
            'PROCEDURE': 'Defines a procedure',
            'IF': 'Conditional statement',
            'WHILE': 'While loop',
            'FOR': 'For loop',
            'RETURN': 'Returns a value from a function',
            'OUTPUT': 'Outputs a value',
            'INPUT': 'Reads input from user',
            'ARRAY': 'Array type declaration'
        };
        return docs[keyword] || `Keyword: ${keyword}`;
    }

    /**
     * Format function documentation
     */
    formatFunctionDocumentation(func) {
        const params = func.params.map(p => 
            p.type ? `${p.name}: ${p.type}` : p.name
        ).join(', ');
        const returnInfo = func.returnType ? ` → ${func.returnType}` : '';
        return `Function ${func.name}(${params})${returnInfo}`;
    }

    /**
     * Format procedure documentation
     */
    formatProcedureDocumentation(proc) {
        const params = proc.params.map(p => 
            p.type ? `${p.name}: ${p.type}` : p.name
        ).join(', ');
        return `Procedure ${proc.name}(${params})`;
    }

    /**
     * Get hover information for a symbol at a given position
     * @param {string} code - The full code text
     * @param {number} line - Current line (1-based)
     * @param {number} column - Current column (1-based)
     * @returns {Object|null} Hover information or null if no symbol found
     */
    getHoverInfo(code, line, column) {
        const lines = code.split('\n');
        const currentLine = lines[line - 1] || '';
        const beforeCursor = currentLine.substring(0, column - 1);
        
        // Extract the word at the cursor position
        const wordMatch = beforeCursor.match(/([A-Za-z_][A-Za-z0-9_]*)$/);
        if (!wordMatch) return null;
        
        const word = wordMatch[1];
        const symbols = this.extractSymbols(code);
        
        // Check if it's a keyword
        if (this.keywords.includes(word.toUpperCase())) {
            return {
                contents: [
                    { value: `**${word.toUpperCase()}**` },
                    { value: this.getKeywordDocumentation(word.toUpperCase()) }
                ]
            };
        }
        
        // Check if it's a built-in function
        const builtinFunc = this.builtinFunctions.find(f => f.name === word.toUpperCase());
        if (builtinFunc) {
            const params = builtinFunc.params.length > 0 
                ? builtinFunc.params.join(', ')
                : 'no parameters';
            return {
                contents: [
                    { value: `**${builtinFunc.name}**(${params})` },
                    { value: builtinFunc.description }
                ]
            };
        }
        
        // Check if it's a variable
        const variable = symbols.variables.find(v => v.name === word);
        if (variable) {
            const typeInfo = variable.type ? `: ${variable.type}` : '';
            return {
                contents: [
                    { value: `**Variable:** \`${variable.name}${typeInfo}\`` }
                ]
            };
        }
        
        // Check if it's a constant
        const constant = symbols.constants.find(c => c.name === word);
        if (constant) {
            return {
                contents: [
                    { value: `**Constant:** \`${constant.name}\`` }
                ]
            };
        }
        
        // Check if it's a function
        const func = symbols.functions.find(f => f.name === word);
        if (func) {
            return {
                contents: [
                    { value: `**Function:** ${this.formatFunctionDocumentation(func)}` }
                ]
            };
        }
        
        // Check if it's a procedure
        const proc = symbols.procedures.find(p => p.name === word);
        if (proc) {
            return {
                contents: [
                    { value: `**Procedure:** ${this.formatProcedureDocumentation(proc)}` }
                ]
            };
        }
        
        // Check if it's a type
        const type = symbols.types.find(t => t.name === word);
        if (type) {
            return {
                contents: [
                    { value: `**Type:** \`${type.name}\`` }
                ]
            };
        }
        
        return null;
    }
}

