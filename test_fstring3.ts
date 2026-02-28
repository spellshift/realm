const COLOR_STRING = "\x1b[38;2;197;148;124m";
const COLOR_PUNCTUATION_1 = "\x1b[38;2;249;217;73m";
const RESET = "\x1b[0m";

const highlightPythonSyntax = (input: string): string => {
    // Just a stub for testing
    return `[highlight: ${input}]`;
}

const processFString = (fstr: string): string => {
    // Look for f"..." or f'...' where ... is fstr.slice(2, -1)
    // To handle { ... } inside, we can replace { ... } with recursive calls.
    // This is simple since f-strings can have nested {} but usually just have simple variables.

    // First, color the whole f-string as a string
    // But we need to color `{expr}` normally by extracting them

    let result = "";

    // Pattern to find `{ ... }` inside an f-string
    // This simple regex handles balanced `{}` if they are not nested `{ { } }`
    // We can just use a simple state machine or regex: `/(\{\{)|(\}\})|(\{.*?\})/g`

    result = fstr.replace(/\{\{|\}\}|\{[^}]*\}/g, (match) => {
        if (match === '{{' || match === '}}') {
            return match; // Keep escaped braces colored as string
        }

        // This is an expression
        const inner = match.slice(1, -1);

        // We highlight the inner expression, but we also want the braces to be colored like punctuation
        // Note: the `highlightPythonSyntax` function handles punctuation internally if we pass the whole thing
        // But we want to specifically color the `{` and `}` and highlight the inner part
        return `${RESET}${COLOR_PUNCTUATION_1}{${RESET}${highlightPythonSyntax(inner)}${COLOR_PUNCTUATION_1}}${RESET}${COLOR_STRING}`;
    });

    return `${COLOR_STRING}${result}${RESET}`;
}

console.log(processFString('f"Hello {name} {age + 1}"'));
