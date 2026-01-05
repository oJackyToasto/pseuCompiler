/**
 * Pseudocode Language Service Wrapper
 * 
 * This module provides a thin wrapper around the Rust/WASM language service.
 * The core logic is implemented in Rust for accuracy and reusability.
 */

export class PseudocodeLanguageService {
    constructor(engine) {
        this.engine = engine;
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
        if (!this.engine) {
            return [];
        }

        try {
            const result = this.engine.get_completions(code, line, column);
            const completionResult = result;
            
            // Use items directly - Rust already handles prefix filtering
            let items = completionResult.items || [];
            
            // Additional filtering only if needed (Rust should handle this, but keep as fallback)
            if (prefix && prefix.length > 0) {
                const prefixLower = prefix.toLowerCase();
                items = items.filter(item => 
                    item.label.toLowerCase().startsWith(prefixLower)
                );
            }

            return items.map(item => ({
                label: item.label,
                kind: item.kind,
                detail: item.detail,
                documentation: item.documentation,
                insertText: item.insert_text
            }));
        } catch (error) {
            console.error('Error getting completions:', error);
            return [];
        }
    }

    /**
     * Get hover information for a symbol at a given position
     * @param {string} code - The full code text
     * @param {number} line - Current line (1-based)
     * @param {number} column - Current column (1-based)
     * @returns {Object|null} Hover information or null if no symbol found
     */
    getHoverInfo(code, line, column) {
        if (!this.engine) {
            return null;
        }

        try {
            const result = this.engine.get_hover(code, line, column);
            const hoverInfo = result;
            
            if (!hoverInfo.contents || hoverInfo.contents.trim() === '') {
                return null;
            }

            return {
                contents: [
                    { value: hoverInfo.contents }
                ]
            };
        } catch (error) {
            console.error('Error getting hover info:', error);
            return null;
        }
    }

}
