const COLOR_STRING = "\x1b[38;2;197;148;124m";
const COLOR_METHOD = "\x1b[38;2;220;220;175m";
const COLOR_KEYWORD = "\x1b[38;2;188;137;189m";
const COLOR_OPERAND = "\x1b[38;2;103;155;209m";
const COLOR_PUNCTUATION_1 = "\x1b[38;2;249;217;73m";
const COLOR_PUNCTUATION_2 = "\x1b[38;2;204;118;209m";
const COLOR_COMMENT = "\x1b[90m";
const COLOR_NUMBER = "\x1b[33m";
const RESET = "\x1b[0m";

// A simplified highlight function
export const highlightPythonSyntax = (input: string): string => {
    // Order matters for regex capture groups
    const regex = new RegExp([
        // 1. f-Strings (f"..." or f'...')
        /(f(?:(["'])(?:\\.|[^\\])*?\2))/.source,
        // 2. Strings (single/double quoted)
        /((["'])(?:\\.|[^\\])*?\2)/.source,
        // 3. Comments (#...)
        /(#.*)/.source,
        // 4. Keywords
        /(\b(?:def|class|import|from|return|if|else|elif|while|for|try|except|finally|with|as|pass|break|continue|lambda|yield|global|nonlocal|assert|del|raise)\b)/.source,
        // 5. Operands
        /(\b(?:is|not|in|and|or|True|False|None)\b)/.source,
        // 6. Built-ins / Methods
        `(\\b(?:sys\\.shell)\\b)`,
        // 7. Numbers
        /(\b\d+\b)/.source,
        // 8. Punctuation
        /([()[\]{}])/.source
    ].join("|"), "g");

    let lastIndex = 0;
    let result = "";
    let match;
    let depth = 0;

    while ((match = regex.exec(input)) !== null) {
        if (match.index > lastIndex) {
            result += input.slice(lastIndex, match.index);
        }

        const text = match[0];
        if (match[1]) {
            // f-String
            // We need to highlight variables inside `{}`
            const fstr = text;
            const replaced = fstr.replace(/\{\{|\}\}|\{[^}]*\}/g, (m) => {
                if (m === '{{' || m === '}}') {
                    return m;
                }
                const inner = m.slice(1, -1);
                const highlightedInner = highlightPythonSyntax(inner);
                return `${RESET}${COLOR_PUNCTUATION_1}{${RESET}${highlightInner}${COLOR_PUNCTUATION_1}}${RESET}${COLOR_STRING}`;
            });
            result += `${COLOR_STRING}${replaced}${RESET}`;
        } else if (match[3]) {
            // String
            result += `${COLOR_STRING}${text}${RESET}`;
        } else if (match[5]) {
            // Comment
            result += `${COLOR_COMMENT}${text}${RESET}`;
        } else if (match[6]) {
            // Keyword
            result += `${COLOR_KEYWORD}${text}${RESET}`;
        } else if (match[7]) {
            // Operand
            result += `${COLOR_OPERAND}${text}${RESET}`;
        } else if (match[8]) {
            // Built-in / Method
            result += `${COLOR_METHOD}${text}${RESET}`;
        } else if (match[9]) {
            // Number
            result += `${COLOR_NUMBER}${text}${RESET}`;
        } else if (match[10]) {
            // Punctuation
            if (/[)\]}]/.test(text)) {
                depth = Math.max(0, depth - 1);
            }

            const color = depth % 2 === 0 ? COLOR_PUNCTUATION_1 : COLOR_PUNCTUATION_2;
            result += `${color}${text}${RESET}`;

            if (/[[({]/.test(text)) {
                depth++;
            }
        } else {
            result += text;
        }

        lastIndex = regex.lastIndex;
    }

    if (lastIndex < input.length) {
        result += input.slice(lastIndex);
    }

    return result;
};

console.log(highlightPythonSyntax('x = f"Hello {name} {age + 1} and {{escaped}}"'));
