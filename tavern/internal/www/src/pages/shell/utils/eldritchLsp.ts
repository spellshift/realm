import docsData from "../../../assets/eldritch-docs.json";

export interface EldritchDocEntry {
    signature: string;
    description: string;
}

export interface EldritchParamInfo {
    name: string;
    typeHint?: string;
    hasDefault: boolean;
    isVariadic: boolean;
    isKwargs: boolean;
}

export interface EldritchFunctionSymbol {
    qualifiedName: string;
    moduleName?: string;
    name: string;
    signature: string;
    description: string;
    params: EldritchParamInfo[];
    minArgs: number;
    maxArgs: number | null; // null if variadic
    hasKwargs: boolean;
    returnType?: string;
    deprecatedReplacement?: string;
}

export type DiagnosticSeverity = "error" | "warning" | "info";

export interface EldritchDiagnostic {
    line: number; // 0-based line index
    startCol: number;
    endCol: number;
    severity: DiagnosticSeverity;
    code: string;
    message: string;
    replacement?: {
        line: number;
        startCol: number;
        endCol: number;
        newText: string;
        label: string;
    };
}

export type CompletionKind =
    | "module"
    | "function"
    | "method"
    | "builtin"
    | "keyword"
    | "variable"
    | "parameter";

export interface EldritchCompletionItem {
    label: string;
    insertText: string;
    kind: CompletionKind;
    detail?: string;
    documentation?: string;
    deprecated?: boolean;
}

export interface EldritchCompletionResult {
    startOffset: number;
    endOffset: number;
    replacePrefix: string;
    items: EldritchCompletionItem[];
}

export interface EldritchSignatureHelp {
    functionName: string;
    signature: string;
    description: string;
    params: string[];
    activeParam: number;
}

export type HighlightTokenType =
    | "plain"
    | "keyword"
    | "boolean"
    | "module"
    | "function"
    | "builtin"
    | "string"
    | "number"
    | "comment"
    | "operator"
    | "punctuation";

export interface HighlightToken {
    text: string;
    type: HighlightTokenType;
}

const rawDocs = docsData as Record<string, EldritchDocEntry>;

export const ELDRITCH_MODULES: Record<string, string> = {
    aes: "AES encryption and decryption for bytes and files",
    agent: "Agent metadata and runtime control",
    assets: "Embedded tome asset access",
    base64: "Base64 encoding and decoding",
    chain: "Multi-agent C2 chaining",
    dns: "DNS resolution and lookups",
    file: "File system operations",
    gzip: "Gzip compression and decompression",
    hash: "Cryptographic hashing (md5, sha1, sha256, file)",
    http: "HTTP/HTTPS client requests",
    json: "JSON encoding, decoding, and validation",
    linux: "Linux-specific system operations (uname)",
    net: "Network interfaces, connections, and TCP/UDP clients",
    portal: "Bi-directional interactive portal streams",
    process: "Process management, PID, cwd, and environment variables",
    random: "Cryptographically secure random generation",
    regex: "Regular expression operations",
    registry: "Windows Registry reading and writing",
    report: "Structured Knowledge Graph reporting to Tavern",
    scan: "Network discovery and port scanning (arp, ports)",
    ssh: "SSH command execution, file copy, and deployment",
    sys: "System OS info, hostname, and command execution",
    template: "Jinja2 template rendering from strings and files",
    time: "Time formatting, epoch conversion, and sleep delays",
    user: "Current user info and user account enumeration",
    windows: "Windows-specific DLL injection/reflection and tokens",
};

export const DEPRECATED_STDLIB_FUNCTIONS: Record<string, string> = {
    "file.temp_file": "file.tmp",
    "file.pwd": "process.cwd",
    "file.template": "template.render_file",
    "file.template_str": "template.render",
    "file.read_binary": "file.read",
    "file.write_binary": "file.write",
    "file.append_binary": "file.append",
    "file.compress": "gzip.compress_file",
    "file.decompress": "gzip.decompress_file",
    "assets.read_binary": "assets.read",
    "time.format_to_epoch": "time.to_epoch",
    "time.format_to_readable": "time.format",
    "agent.get_config": "agent.config",
    "agent.get_transport": "agent.transport",
    "agent.get_callback_interval": "agent.callback_interval",
    "crypto.aes_decrypt": "aes.decrypt",
    "crypto.aes_decrypt_file": "aes.decrypt_file",
    "crypto.aes_encrypt": "aes.encrypt",
    "crypto.aes_encrypt_file": "aes.encrypt_file",
    "crypto.decode_b64": "base64.decode",
    "crypto.encode_b64": "base64.encode",
    "crypto.from_json": "json.decode",
    "crypto.hash_file": "hash.file",
    "crypto.is_json": "json.is_valid",
    "crypto.md5": "hash.md5",
    "crypto.sha1": "hash.sha1",
    "crypto.sha256": "hash.sha256",
    "crypto.to_json": "json.encode",
    "pivot.arp_scan": "scan.arp",
    "pivot.create_portal": "portal.create",
    "pivot.ncat": "net.tcp",
    "pivot.port_scan": "scan.ports",
    "pivot.ssh_copy": "ssh.copy",
    "pivot.ssh_deploy": "ssh.deploy",
    "pivot.ssh_exec": "ssh.exec",
    "process.netstat": "net.conns",
    "sys.dll_inject": "windows.dll_inject",
    "sys.dll_reflect": "windows.dll_reflect",
    "sys.get_env": "process.env",
    "sys.get_ip": "net.interfaces",
    "sys.get_os": "sys.info",
    "sys.get_pid": "process.pid",
    "sys.get_reg": "registry.read",
    "sys.get_user": "user.info",
    "sys.impersonate": "windows.impersonate",
    "sys.list_users": "user.list",
    "sys.tokens": "windows.tokens",
    "sys.use_token": "windows.use_token",
    "sys.whereis": "file.which",
    "sys.write_reg": "registry.write",
};

export const REMOVED_STDLIB_FUNCTIONS: Record<string, string> = {
    "file.moveto": "'file.moveto' does not exist in Eldritch v0.4.1. Use 'file.move(src, dst)' to move/rename files.",
    "pivot.bind_proxy": "'pivot.bind_proxy' was an unimplemented proposal and has been removed.",
    "pivot.port_forward": "'pivot.port_forward' was an unimplemented proposal and has been removed.",
    "pivot.smb_exec": "'pivot.smb_exec' was an unimplemented proposal and has been removed.",
};

export const ELDRITCH_KEYWORDS = [
    "def",
    "return",
    "if",
    "elif",
    "else",
    "for",
    "in",
    "and",
    "or",
    "not",
    "break",
    "continue",
    "pass",
    "lambda",
];

export const ELDRITCH_BOOLEANS = ["True", "False", "None"];

export const ELDRITCH_BUILTINS: Record<string, { signature: string; description: string }> = {
    print: { signature: "print(*args)", description: "Prints values to standard output." },
    eprint: { signature: "eprint(*args)", description: "Prints values to standard error." },
    pprint: { signature: "pprint(value, indent=2)", description: "Pretty-prints a dictionary or list." },
    tprint: { signature: "tprint(rows)", description: "Formats and prints a list of dictionaries as an aligned table." },
    len: { signature: "len(collection) -> int", description: "Returns the number of elements in a collection or string." },
    range: { signature: "range(start, stop=None, step=1) -> list[int]", description: "Generates a sequence of integers." },
    str: { signature: "str(value='') -> str", description: "Converts a value (including Bytes) to a UTF-8 string." },
    int: { signature: "int(value, base=10) -> int", description: "Converts a string or number to an integer." },
    float: { signature: "float(value) -> float", description: "Converts a string or number to a floating-point number." },
    bool: { signature: "bool(value=False) -> bool", description: "Converts a value to a boolean." },
    bytes: { signature: "bytes(value='') -> Bytes", description: "Converts a UTF-8 string or integer list into Bytes." },
    list: { signature: "list(iterable=None) -> list", description: "Creates a list from an iterable." },
    dict: { signature: "dict(**kwargs) -> dict", description: "Creates a dictionary." },
    tuple: { signature: "tuple(iterable=None) -> tuple", description: "Creates an immutable tuple." },
    set: { signature: "set(iterable=None) -> set", description: "Creates a set of unique values." },
    type: { signature: "type(value) -> str", description: "Returns the type name of the given value." },
    dir: { signature: "dir(value=None) -> list[str]", description: "Lists attributes and methods on an object or in scope." },
    enumerate: { signature: "enumerate(iterable, start=0) -> list", description: "Returns (index, item) pairs for an iterable." },
    zip: { signature: "zip(*iterables) -> list", description: "Combines elements from multiple iterables into tuples." },
    sorted: { signature: "sorted(iterable, key=None, reverse=False) -> list", description: "Returns a new sorted list." },
    reversed: { signature: "reversed(iterable) -> list", description: "Returns a reversed list." },
    min: { signature: "min(*args)", description: "Returns the smallest item." },
    max: { signature: "max(*args)", description: "Returns the largest item." },
    abs: { signature: "abs(number) -> number", description: "Returns the absolute value of a number." },
    all: { signature: "all(iterable) -> bool", description: "Returns True if all elements in the iterable are truthy." },
    any: { signature: "any(iterable) -> bool", description: "Returns True if any element in the iterable is truthy." },
    chr: { signature: "chr(codepoint) -> str", description: "Returns the character for a Unicode codepoint." },
    ord: { signature: "ord(char) -> int", description: "Returns the Unicode codepoint for a single character." },
    hex: { signature: "hex(number) -> str", description: "Formats an integer as a lowercase hexadecimal string." },
    repr: { signature: "repr(value) -> str", description: "Returns the debug string representation of a value." },
    fail: { signature: "fail(message)", description: "Aborts execution immediately with the given error message." },
    assert: { signature: "assert(condition, message=None)", description: "Fails execution if condition is falsy." },
    assert_eq: { signature: "assert_eq(left, right, message=None)", description: "Fails execution if left != right." },
    map: { signature: "map(func, iterable) -> list", description: "Applies func to every item of iterable." },
    filter: { signature: "filter(func, iterable) -> list", description: "Filters items of iterable where func(item) is truthy." },
    reduce: { signature: "reduce(func, iterable, initial=None)", description: "Reduces iterable cumulatively using func." },
    eval: { signature: "eval(script) -> Any", description: "Evaluates an Eldritch code string." },
    libs: { signature: "libs() -> list[str]", description: "Returns the list of available standard library modules." },
    builtins: { signature: "builtins() -> list[str]", description: "Returns the list of built-in functions." },
};

export function parseFunctionSignature(
    qualifiedName: string,
    signature: string,
    description: string
): EldritchFunctionSymbol {
    const dotIdx = qualifiedName.indexOf(".");
    const moduleName = dotIdx !== -1 ? qualifiedName.slice(0, dotIdx) : undefined;
    const name = dotIdx !== -1 ? qualifiedName.slice(dotIdx + 1) : qualifiedName;

    const openParen = signature.indexOf("(");
    const closeParen = signature.lastIndexOf(")");
    const returnMatch = signature.slice(closeParen + 1).match(/->\s*(.+)$/);
    const returnType = returnMatch ? returnMatch[1].trim() : undefined;

    const params: EldritchParamInfo[] = [];
    let minArgs = 0;
    let maxArgs: number | null = 0;
    let hasKwargs = false;

    if (openParen !== -1 && closeParen > openParen) {
        const rawParamStr = signature.slice(openParen + 1, closeParen).trim();
        if (rawParamStr.length > 0) {
            // Split top-level commas (ignoring commas inside nested brackets/parens)
            const parts: string[] = [];
            let current = "";
            let depth = 0;
            for (const ch of rawParamStr) {
                if (ch === "[" || ch === "(" || ch === "{") depth++;
                else if (ch === "]" || ch === ")" || ch === "}") depth = Math.max(0, depth - 1);
                if (ch === "," && depth === 0) {
                    parts.push(current.trim());
                    current = "";
                } else {
                    current += ch;
                }
            }
            if (current.trim()) parts.push(current.trim());

            for (const part of parts) {
                if (!part) continue;
                const isKwargs = part.startsWith("**");
                const isVariadic = !isKwargs && part.startsWith("*");
                const hasDefault = part.includes("=") || part.includes("Optional<");
                const cleaned = part.replace(/^\*{1,2}/, "");
                const [nameAndType] = cleaned.split("=");
                const [rawName, rawType] = nameAndType.split(":");
                const paramName = rawName.trim();
                if (!paramName) continue;

                params.push({
                    name: paramName,
                    typeHint: rawType ? rawType.trim() : undefined,
                    hasDefault,
                    isVariadic,
                    isKwargs,
                });

                if (isKwargs) {
                    hasKwargs = true;
                } else if (isVariadic) {
                    maxArgs = null;
                } else {
                    if (!hasDefault) {
                        minArgs++;
                    }
                    if (maxArgs !== null) {
                        maxArgs++;
                    }
                }
            }
        }
    }

    return {
        qualifiedName,
        moduleName,
        name,
        signature,
        description,
        params,
        minArgs,
        maxArgs,
        hasKwargs,
        returnType,
        deprecatedReplacement: DEPRECATED_STDLIB_FUNCTIONS[qualifiedName],
    };
}

class StdlibRegistry {
    public functions = new Map<string, EldritchFunctionSymbol>();
    public byModule = new Map<string, EldritchFunctionSymbol[]>();
    public builtins = new Map<string, EldritchFunctionSymbol>();
    public valueMethods = new Map<string, EldritchFunctionSymbol[]>();

    constructor() {
        for (const [key, entry] of Object.entries(rawDocs)) {
            const sym = parseFunctionSignature(key, entry.signature, entry.description);
            if (sym.moduleName && ["str", "list", "dict", "set", "bytes"].includes(sym.moduleName)) {
                const arr = this.valueMethods.get(sym.moduleName) || [];
                arr.push(sym);
                this.valueMethods.set(sym.moduleName, arr);
                continue;
            }
            this.functions.set(key, sym);
            if (sym.moduleName) {
                const list = this.byModule.get(sym.moduleName) || [];
                list.push(sym);
                this.byModule.set(sym.moduleName, list);
            }
        }

        for (const [name, entry] of Object.entries(ELDRITCH_BUILTINS)) {
            const sym = parseFunctionSignature(name, entry.signature, entry.description);
            this.builtins.set(name, sym);
        }
    }
}

export const stdlibRegistry = new StdlibRegistry();

function levenshteinDistance(a: string, b: string): number {
    const dp: number[][] = Array.from({ length: a.length + 1 }, () =>
        new Array(b.length + 1).fill(0)
    );
    for (let i = 0; i <= a.length; i++) dp[i][0] = i;
    for (let j = 0; j <= b.length; j++) dp[0][j] = j;
    for (let i = 1; i <= a.length; i++) {
        for (let j = 1; j <= b.length; j++) {
            const cost = a[i - 1] === b[j - 1] ? 0 : 1;
            dp[i][j] = Math.min(
                dp[i - 1][j] + 1,
                dp[i][j - 1] + 1,
                dp[i - 1][j - 1] + cost
            );
        }
    }
    return dp[a.length][b.length];
}

function suggestClosestMember(moduleName: string, funcName: string): string | null {
    const members = stdlibRegistry.byModule.get(moduleName);
    if (!members || members.length === 0) return null;
    let best: string | null = null;
    let bestDist = Infinity;
    for (const m of members) {
        const d = levenshteinDistance(funcName, m.name);
        if (d < bestDist && d <= 3) {
            bestDist = d;
            best = `${moduleName}.${m.name}`;
        }
    }
    return best;
}

/**
 * Strips comments and replaces string contents with spaces of identical length
 * so column offsets for structural/semantic analysis match the original line.
 */
function maskStringsAndCommentsOnLine(line: string): {
    masked: string;
    unterminatedQuote: { col: number; quote: string } | null;
} {
    let masked = "";
    let inSingle = false;
    let inDouble = false;
    let quoteStartCol = 0;
    let escape = false;

    for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        if (escape) {
            escape = false;
            masked += inSingle || inDouble ? " " : ch;
            continue;
        }
        if (ch === "\\") {
            escape = true;
            masked += inSingle || inDouble ? " " : ch;
            continue;
        }
        if (inSingle) {
            if (ch === "'") {
                inSingle = false;
                masked += "'";
            } else {
                masked += " ";
            }
            continue;
        }
        if (inDouble) {
            if (ch === '"') {
                inDouble = false;
                masked += '"';
            } else {
                masked += " ";
            }
            continue;
        }
        if (ch === "#") {
            masked += " ".repeat(line.length - i);
            break;
        }
        if (ch === "'") {
            inSingle = true;
            quoteStartCol = i;
            masked += "'";
            continue;
        }
        if (ch === '"') {
            inDouble = true;
            quoteStartCol = i;
            masked += '"';
            continue;
        }
        masked += ch;
    }

    const unterminatedQuote =
        inSingle
            ? { col: quoteStartCol, quote: "'" }
            : inDouble
              ? { col: quoteStartCol, quote: '"' }
              : null;

    return { masked, unterminatedQuote };
}

function parseCallArguments(argsStr: string): {
    positionalCount: number;
    keywordNames: { name: string; offset: number }[];
    hasStarArgs: boolean;
} {
    const trimmed = argsStr.trim();
    if (!trimmed) {
        return { positionalCount: 0, keywordNames: [], hasStarArgs: false };
    }
    const parts: { text: string; startOffset: number }[] = [];
    let current = "";
    let startOffset = 0;
    let depth = 0;

    for (let i = 0; i < argsStr.length; i++) {
        const ch = argsStr[i];
        if (ch === "(" || ch === "[" || ch === "{") depth++;
        else if (ch === ")" || ch === "]" || ch === "}") depth = Math.max(0, depth - 1);

        if (ch === "," && depth === 0) {
            parts.push({ text: current, startOffset });
            current = "";
            startOffset = i + 1;
        } else {
            current += ch;
        }
    }
    if (current.trim()) {
        parts.push({ text: current, startOffset });
    }

    let positionalCount = 0;
    let hasStarArgs = false;
    const keywordNames: { name: string; offset: number }[] = [];

    for (const part of parts) {
        const t = part.text.trim();
        if (!t) continue;
        if (t.startsWith("*")) {
            hasStarArgs = true;
            continue;
        }
        const kwMatch = part.text.match(/^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=(?!=)/);
        if (kwMatch) {
            keywordNames.push({
                name: kwMatch[2],
                offset: part.startOffset + kwMatch[1].length,
            });
        } else {
            positionalCount++;
        }
    }

    return { positionalCount, keywordNames, hasStarArgs };
}

/**
 * Runs static syntax, indentation, semantic, and linter analysis matching `eldritch-lsp`.
 */
export function analyzeEldritchCode(code: string): EldritchDiagnostic[] {
    const diagnostics: EldritchDiagnostic[] = [];
    if (!code.trim()) return diagnostics;

    const lines = code.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");

    // 1. Collect user-defined functions (`def name(...):`)
    const userFunctions = new Map<string, EldritchFunctionSymbol>();
    for (let i = 0; i < lines.length; i++) {
        const defMatch = lines[i].match(/^\s*def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(\([^)]*\))\s*:/);
        if (defMatch) {
            const fnName = defMatch[1];
            const sig = `${fnName}${defMatch[2]}`;
            userFunctions.set(fnName, parseFunctionSignature(fnName, sig, "User-defined function"));
        }
    }

    // 2. Track bracket balance, indentation, control flow scopes, and line-level checks
    const bracketStack: { char: string; expected: string; line: number; col: number }[] = [];
    const indentStack: number[] = [0];
    let expectIndentedBlockAfterLine: number | null = null;
    const functionScopeIndents: number[] = [];
    const loopScopeIndents: number[] = [];
    const terminatedBlockIndents = new Set<number>();

    let inTripleSingle = false;
    let inTripleDouble = false;

    for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
        const rawLine = lines[lineIdx];

        // Check `no-forbidden-runes` linter rule from `eldritch-lsp/src/linter.rs`
        const lowerLine = rawLine.toLowerCase();
        const vecnaIdx = lowerLine.indexOf("vecna");
        if (vecnaIdx !== -1) {
            diagnostics.push({
                line: lineIdx,
                startCol: vecnaIdx,
                endCol: vecnaIdx + 5,
                severity: "warning",
                code: "no-forbidden-runes",
                message: "Forbidden rune 'vecna' detected by eldritch-lsp linter.",
            });
        }

        // Handle multi-line triple-quoted strings
        if (inTripleSingle) {
            const closeIdx = rawLine.indexOf("'''");
            if (closeIdx !== -1) {
                inTripleSingle = false;
            }
            continue;
        }
        if (inTripleDouble) {
            const closeIdx = rawLine.indexOf('"""');
            if (closeIdx !== -1) {
                inTripleDouble = false;
            }
            continue;
        }

        const tripleSingleMatches = rawLine.match(/'''/g);
        if (tripleSingleMatches && tripleSingleMatches.length % 2 === 1) {
            inTripleSingle = true;
            continue;
        }
        const tripleDoubleMatches = rawLine.match(/"""/g);
        if (tripleDoubleMatches && tripleDoubleMatches.length % 2 === 1) {
            inTripleDouble = true;
            continue;
        }

        const indentMatch = rawLine.match(/^([ \t]*)/);
        const leadingWhitespace = indentMatch ? indentMatch[1] : "";
        const isShellMacroLine = rawLine.trimStart().startsWith("!");

        const { masked, unterminatedQuote } = isShellMacroLine
            ? { masked: `${leadingWhitespace}sys.shell(" ")`, unterminatedQuote: null }
            : maskStringsAndCommentsOnLine(rawLine);
        if (unterminatedQuote) {
            diagnostics.push({
                line: lineIdx,
                startCol: unterminatedQuote.col,
                endCol: rawLine.length,
                severity: "error",
                code: "syntax-unterminated-string",
                message: `Unterminated string literal starting with ${unterminatedQuote.quote}`,
            });
        }

        const trimmedMasked = masked.trim();
        if (!trimmedMasked) {
            continue;
        }

        const insideGrouping = bracketStack.length > 0;

        // Indentation & block structure validation when not inside open (), [], {}
        const indent = leadingWhitespace.replace(/\t/g, "    ").length;

        if (leadingWhitespace.includes("\t") && leadingWhitespace.includes(" ")) {
            diagnostics.push({
                line: lineIdx,
                startCol: 0,
                endCol: leadingWhitespace.length,
                severity: "warning",
                code: "mixed-tabs-spaces",
                message: "Mixed tabs and spaces in indentation; prefer 4 spaces.",
            });
        }

        if (!insideGrouping) {
            const currentIndent = indentStack[indentStack.length - 1];

            if (expectIndentedBlockAfterLine !== null) {
                if (indent <= currentIndent) {
                    diagnostics.push({
                        line: lineIdx,
                        startCol: 0,
                        endCol: Math.max(1, rawLine.trim().length),
                        severity: "error",
                        code: "expected-indented-block",
                        message: `Expected an indented block after ':' on line ${expectIndentedBlockAfterLine + 1}.`,
                    });
                } else {
                    indentStack.push(indent);
                }
                expectIndentedBlockAfterLine = null;
            } else if (indent > currentIndent) {
                diagnostics.push({
                    line: lineIdx,
                    startCol: 0,
                    endCol: indent,
                    severity: "error",
                    code: "unexpected-indent",
                    message: "Unexpected indentation.",
                });
                indentStack.push(indent);
            } else if (indent < currentIndent) {
                while (indentStack.length > 1 && indentStack[indentStack.length - 1] > indent) {
                    const popped = indentStack.pop()!;
                    terminatedBlockIndents.delete(popped);
                }
                if (indentStack[indentStack.length - 1] !== indent) {
                    diagnostics.push({
                        line: lineIdx,
                        startCol: 0,
                        endCol: indent,
                        severity: "error",
                        code: "inconsistent-indentation",
                        message: `Inconsistent indentation (${indent} spaces does not match any outer block level).`,
                    });
                }
            }

            // Pop closed function / loop scopes
            while (
                functionScopeIndents.length > 0 &&
                functionScopeIndents[functionScopeIndents.length - 1] >= indent
            ) {
                functionScopeIndents.pop();
            }
            while (
                loopScopeIndents.length > 0 &&
                loopScopeIndents[loopScopeIndents.length - 1] >= indent
            ) {
                loopScopeIndents.pop();
            }

            // Check unreachable code after return/break/continue/fail at same indent
            if (terminatedBlockIndents.has(indent)) {
                const firstChar = rawLine.search(/\S/);
                diagnostics.push({
                    line: lineIdx,
                    startCol: Math.max(0, firstChar),
                    endCol: rawLine.length,
                    severity: "warning",
                    code: "unreachable-code",
                    message: "Unreachable code after return/break/continue/fail.",
                });
                terminatedBlockIndents.delete(indent);
            }

            // Check missing colon on compound statement headers
            const headerMatch = trimmedMasked.match(/^(def|if|elif|else|for)\b(.*)$/);
            if (headerMatch && !trimmedMasked.endsWith(":")) {
                // Only flag if brackets on the line are balanced
                let lineBracketDiff = 0;
                for (const ch of masked) {
                    if (ch === "(" || ch === "[" || ch === "{") lineBracketDiff++;
                    else if (ch === ")" || ch === "]" || ch === "}") lineBracketDiff--;
                }
                if (lineBracketDiff === 0) {
                    diagnostics.push({
                        line: lineIdx,
                        startCol: rawLine.search(/\S/),
                        endCol: rawLine.length,
                        severity: "error",
                        code: "missing-colon",
                        message: `Expected ':' at the end of '${headerMatch[1]}' statement.`,
                    });
                }
            }

            // Track entering `def` or `for` scopes
            if (/^def\b/.test(trimmedMasked) && trimmedMasked.endsWith(":")) {
                functionScopeIndents.push(indent);
            }
            if (/^for\b/.test(trimmedMasked) && trimmedMasked.endsWith(":")) {
                loopScopeIndents.push(indent);
            }

            // Validate `return` inside `def` and `break`/`continue` inside `for`
            if (/^return\b/.test(trimmedMasked)) {
                if (functionScopeIndents.length === 0) {
                    const col = rawLine.indexOf("return");
                    diagnostics.push({
                        line: lineIdx,
                        startCol: col,
                        endCol: col + 6,
                        severity: "error",
                        code: "return-outside-function",
                        message: "'return' outside of function definition.",
                    });
                } else {
                    terminatedBlockIndents.add(indent);
                }
            }
            if (/^(break|continue)\b/.test(trimmedMasked)) {
                const kw = trimmedMasked.startsWith("break") ? "break" : "continue";
                if (loopScopeIndents.length === 0) {
                    const col = rawLine.indexOf(kw);
                    diagnostics.push({
                        line: lineIdx,
                        startCol: col,
                        endCol: col + kw.length,
                        severity: "error",
                        code: `${kw}-outside-loop`,
                        message: `'${kw}' outside of 'for' loop.`,
                    });
                } else {
                    terminatedBlockIndents.add(indent);
                }
            }
            if (/^fail\s*\(/.test(trimmedMasked)) {
                terminatedBlockIndents.add(indent);
            }

            // Check shadowed stdlib module assignment (`file = ...`)
            const assignMatch = masked.match(/^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=(?!=)/);
            if (assignMatch) {
                const varName = assignMatch[1];
                if (ELDRITCH_MODULES[varName]) {
                    const col = rawLine.indexOf(varName);
                    diagnostics.push({
                        line: lineIdx,
                        startCol: col,
                        endCol: col + varName.length,
                        severity: "warning",
                        code: "shadowed-stdlib-module",
                        message: `Variable '${varName}' shadows standard library module '${varName}'.`,
                    });
                }
            }

            if (trimmedMasked.endsWith(":")) {
                expectIndentedBlockAfterLine = lineIdx;
            }
        }

        // Update bracket stack across the line
        for (let col = 0; col < masked.length; col++) {
            const ch = masked[col];
            if (ch === "(") bracketStack.push({ char: "(", expected: ")", line: lineIdx, col });
            else if (ch === "[") bracketStack.push({ char: "[", expected: "]", line: lineIdx, col });
            else if (ch === "{") bracketStack.push({ char: "{", expected: "}", line: lineIdx, col });
            else if (ch === ")" || ch === "]" || ch === "}") {
                const top = bracketStack.pop();
                if (!top) {
                    diagnostics.push({
                        line: lineIdx,
                        startCol: col,
                        endCol: col + 1,
                        severity: "error",
                        code: "unmatched-closing-bracket",
                        message: `Unmatched closing '${ch}'.`,
                    });
                } else if (top.expected !== ch) {
                    diagnostics.push({
                        line: lineIdx,
                        startCol: col,
                        endCol: col + 1,
                        severity: "error",
                        code: "mismatched-bracket",
                        message: `Mismatched '${ch}'; expected '${top.expected}' to close '${top.char}' from line ${top.line + 1}.`,
                    });
                }
            }
        }

        // 3. Check stdlib & user function calls on this line
        const callRegex = /\b(?:([a-zA-Z_][a-zA-Z0-9_]*)\.)?([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/g;
        let match: RegExpExecArray | null;
        while ((match = callRegex.exec(masked)) !== null) {
            const fullMatchStart = match.index;
            const modName = match[1];
            const fnName = match[2];
            const qualified = modName ? `${modName}.${fnName}` : fnName;

            // Skip `def fnName(` declaration itself
            const prefixBeforeMatch = masked.slice(0, fullMatchStart);
            if (!modName && /\bdef\s+$/.test(prefixBeforeMatch)) {
                continue;
            }

            const nameEndCol = fullMatchStart + (modName ? modName.length + 1 : 0) + fnName.length;

            // Check removed 0.4.0 functions
            if (REMOVED_STDLIB_FUNCTIONS[qualified]) {
                diagnostics.push({
                    line: lineIdx,
                    startCol: fullMatchStart,
                    endCol: nameEndCol,
                    severity: "error",
                    code: "removed-stdlib-function",
                    message: REMOVED_STDLIB_FUNCTIONS[qualified],
                });
                continue;
            }

            // Check deprecated 0.4.1 functions
            if (DEPRECATED_STDLIB_FUNCTIONS[qualified]) {
                const newFn = DEPRECATED_STDLIB_FUNCTIONS[qualified];
                diagnostics.push({
                    line: lineIdx,
                    startCol: fullMatchStart,
                    endCol: nameEndCol,
                    severity: "warning",
                    code: "deprecated-stdlib-function",
                    message: `'${qualified}' is deprecated in Eldritch v0.4.1; use '${newFn}' instead.`,
                    replacement: {
                        line: lineIdx,
                        startCol: fullMatchStart,
                        endCol: nameEndCol,
                        newText: newFn,
                        label: `Replace with ${newFn}`,
                    },
                });
            }

            // Check unknown function on a known stdlib module
            if (modName && (ELDRITCH_MODULES[modName] || modName === "crypto" || modName === "pivot")) {
                if (
                    !stdlibRegistry.functions.has(qualified) &&
                    !DEPRECATED_STDLIB_FUNCTIONS[qualified]
                ) {
                    const suggestion = suggestClosestMember(modName, fnName);
                    diagnostics.push({
                        line: lineIdx,
                        startCol: fullMatchStart,
                        endCol: nameEndCol,
                        severity: "error",
                        code: "unknown-stdlib-function",
                        message: suggestion
                            ? `Unknown function '${qualified}' in module '${modName}'. Did you mean '${suggestion}'?`
                            : `Unknown function '${qualified}' in standard library module '${modName}'.`,
                        replacement: suggestion
                            ? {
                                  line: lineIdx,
                                  startCol: fullMatchStart,
                                  endCol: nameEndCol,
                                  newText: suggestion,
                                  label: `Replace with ${suggestion}`,
                              }
                            : undefined,
                    });
                    continue;
                }
            }

            // Validate argument count & keyword arguments if the call closes on this line
            const sym =
                (modName
                    ? stdlibRegistry.functions.get(qualified) ||
                      (DEPRECATED_STDLIB_FUNCTIONS[qualified]
                          ? stdlibRegistry.functions.get(DEPRECATED_STDLIB_FUNCTIONS[qualified])
                          : undefined)
                    : userFunctions.get(fnName) || stdlibRegistry.builtins.get(fnName)) || undefined;

            if (sym) {
                const openParenCol = masked.indexOf("(", nameEndCol);
                if (openParenCol !== -1) {
                    let depth = 0;
                    let closeParenCol = -1;
                    for (let c = openParenCol; c < masked.length; c++) {
                        if (masked[c] === "(" || masked[c] === "[" || masked[c] === "{") depth++;
                        else if (masked[c] === ")" || masked[c] === "]" || masked[c] === "}") {
                            depth--;
                            if (depth === 0 && masked[c] === ")") {
                                closeParenCol = c;
                                break;
                            }
                        }
                    }

                    if (closeParenCol !== -1) {
                        const argsSlice = masked.slice(openParenCol + 1, closeParenCol);
                        const { positionalCount, keywordNames, hasStarArgs } =
                            parseCallArguments(argsSlice);

                        const validParamNames = new Set(sym.params.map((p) => p.name));
                        if (!sym.hasKwargs && validParamNames.size > 0) {
                            for (const kw of keywordNames) {
                                if (!validParamNames.has(kw.name)) {
                                    diagnostics.push({
                                        line: lineIdx,
                                        startCol: openParenCol + 1 + kw.offset,
                                        endCol: openParenCol + 1 + kw.offset + kw.name.length,
                                        severity: "error",
                                        code: "unexpected-keyword-arg",
                                        message: `Unexpected keyword argument '${kw.name}' for '${sym.signature}'.`,
                                    });
                                }
                            }
                        }

                        if (!hasStarArgs) {
                            const totalProvided = positionalCount + keywordNames.length;
                            if (totalProvided < sym.minArgs) {
                                diagnostics.push({
                                    line: lineIdx,
                                    startCol: fullMatchStart,
                                    endCol: closeParenCol + 1,
                                    severity: "error",
                                    code: "too-few-arguments",
                                    message: `'${qualified}' requires at least ${sym.minArgs} argument${sym.minArgs === 1 ? "" : "s"}, got ${totalProvided}. Signature: ${sym.signature}`,
                                });
                            } else if (sym.maxArgs !== null && totalProvided > sym.maxArgs) {
                                diagnostics.push({
                                    line: lineIdx,
                                    startCol: fullMatchStart,
                                    endCol: closeParenCol + 1,
                                    severity: "error",
                                    code: "too-many-arguments",
                                    message: `'${qualified}' accepts at most ${sym.maxArgs} argument${sym.maxArgs === 1 ? "" : "s"}, got ${totalProvided}. Signature: ${sym.signature}`,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    if (expectIndentedBlockAfterLine !== null) {
        const lastLine = lines[expectIndentedBlockAfterLine] || "";
        diagnostics.push({
            line: expectIndentedBlockAfterLine,
            startCol: Math.max(0, lastLine.search(/\S/)),
            endCol: lastLine.length,
            severity: "error",
            code: "expected-indented-block",
            message: `Incomplete block: expected an indented body after ':' on line ${expectIndentedBlockAfterLine + 1}.`,
        });
    }

    for (const unclosed of bracketStack) {
        diagnostics.push({
            line: unclosed.line,
            startCol: unclosed.col,
            endCol: unclosed.col + 1,
            severity: "error",
            code: "unclosed-bracket",
            message: `Unclosed '${unclosed.char}'; expected matching '${unclosed.expected}'.`,
        });
    }

    return diagnostics;
}

/**
 * Computes context-aware completions at `cursorOffset` matching `eldritch-lsp`.
 */
export function completeEldritchCode(
    code: string,
    cursorOffset: number
): EldritchCompletionResult | null {
    const safeCursor = Math.max(0, Math.min(cursorOffset, code.length));
    const beforeCursor = code.slice(0, safeCursor);
    const currentLineStart = beforeCursor.lastIndexOf("\n") + 1;
    const lineBeforeCursor = beforeCursor.slice(currentLineStart);

    // Do not trigger autocomplete on `!command` shell macro lines, inside comments, or string literals
    if (lineBeforeCursor.trimStart().startsWith("!")) {
        return null;
    }
    const { masked, unterminatedQuote } = maskStringsAndCommentsOnLine(lineBeforeCursor);
    if (unterminatedQuote) {
        return null;
    }
    const commentStart = lineBeforeCursor.indexOf("#");
    if (commentStart !== -1 && masked.slice(commentStart).trim() === "") {
        return null;
    }

    // 1. Check module/variable member access: `ident.prefix`
    const memberMatch = lineBeforeCursor.match(/([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z0-9_]*)$/);
    if (memberMatch) {
        const receiver = memberMatch[1];
        const memberPrefix = memberMatch[2];
        const startOffset = safeCursor - memberPrefix.length;

        const moduleFunctions = stdlibRegistry.byModule.get(receiver);
        if (moduleFunctions) {
            const items: EldritchCompletionItem[] = moduleFunctions
                .filter((fn) => fn.name.startsWith(memberPrefix) && !fn.deprecatedReplacement)
                .sort((a, b) => a.name.localeCompare(b.name))
                .map((fn) => ({
                    label: `${receiver}.${fn.name}`,
                    insertText: fn.name,
                    kind: "function",
                    detail: fn.signature,
                    documentation: fn.description,
                }));
            return {
                startOffset,
                endOffset: safeCursor,
                replacePrefix: memberPrefix,
                items,
            };
        }

        // Check inferred variable type for value methods (`str`, `list`, `dict`, `bytes`)
        const varType = inferVariableType(code, receiver);
        if (varType && stdlibRegistry.valueMethods.has(varType)) {
            const methods = stdlibRegistry.valueMethods.get(varType)!;
            const items: EldritchCompletionItem[] = methods
                .filter((m) => m.name.startsWith(memberPrefix))
                .map((m) => ({
                    label: `${varType}.${m.name}`,
                    insertText: m.name,
                    kind: "method",
                    detail: m.signature,
                    documentation: m.description,
                }));
            return {
                startOffset,
                endOffset: safeCursor,
                replacePrefix: memberPrefix,
                items,
            };
        }
        return null;
    }

    // 2. General identifier / keyword / function / parameter completion
    const identMatch = lineBeforeCursor.match(/([a-zA-Z_][a-zA-Z0-9_]*)$/);
    const prefix = identMatch ? identMatch[1] : "";
    if (!prefix) {
        return null;
    }

    const startOffset = safeCursor - prefix.length;
    const items: EldritchCompletionItem[] = [];
    const seen = new Set<string>();

    // Check if inside a function call to suggest keyword arguments `param=`
    const sigHelp = getEldritchSignatureHelp(code, safeCursor);
    if (sigHelp) {
        const sym =
            stdlibRegistry.functions.get(sigHelp.functionName) ||
            stdlibRegistry.builtins.get(sigHelp.functionName);
        if (sym) {
            for (const p of sym.params) {
                if (!p.isVariadic && !p.isKwargs && p.name.startsWith(prefix)) {
                    const label = `${p.name}=`;
                    if (!seen.has(label)) {
                        seen.add(label);
                        items.push({
                            label,
                            insertText: `${p.name}=`,
                            kind: "parameter",
                            detail: p.typeHint ? `${p.name}: ${p.typeHint}` : `parameter of ${sigHelp.functionName}`,
                        });
                    }
                }
            }
        }
    }

    // Stdlib modules
    for (const [mod, desc] of Object.entries(ELDRITCH_MODULES)) {
        if (mod.startsWith(prefix) && !seen.has(mod)) {
            seen.add(mod);
            items.push({
                label: mod,
                insertText: mod,
                kind: "module",
                detail: `module ${mod}`,
                documentation: desc,
            });
        }
    }

    // User-defined functions and variables in current buffer
    const defRegex = /^\s*def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(\([^)]*\))/gm;
    let m: RegExpExecArray | null;
    while ((m = defRegex.exec(code)) !== null) {
        const fnName = m[1];
        if (fnName.startsWith(prefix) && !seen.has(fnName)) {
            seen.add(fnName);
            items.push({
                label: fnName,
                insertText: fnName,
                kind: "function",
                detail: `${fnName}${m[2]}`,
                documentation: "User-defined function in script",
            });
        }
    }

    const varRegex = /^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=(?!=)/gm;
    while ((m = varRegex.exec(code)) !== null) {
        const vName = m[1];
        if (vName.startsWith(prefix) && vName !== prefix && !seen.has(vName)) {
            seen.add(vName);
            items.push({
                label: vName,
                insertText: vName,
                kind: "variable",
                detail: "local variable",
            });
        }
    }

    // Built-in functions
    for (const [bName, bInfo] of Object.entries(ELDRITCH_BUILTINS)) {
        if (bName.startsWith(prefix) && !seen.has(bName)) {
            seen.add(bName);
            items.push({
                label: bName,
                insertText: bName,
                kind: "builtin",
                detail: bInfo.signature,
                documentation: bInfo.description,
            });
        }
    }

    // Keywords & booleans
    for (const kw of [...ELDRITCH_KEYWORDS, ...ELDRITCH_BOOLEANS]) {
        if (kw.startsWith(prefix) && !seen.has(kw)) {
            seen.add(kw);
            items.push({
                label: kw,
                insertText: kw,
                kind: "keyword",
                detail: "keyword",
            });
        }
    }

    // Qualified stdlib functions matching prefix (e.g. typing `portal` or `shell` suggests `portal.create`, `sys.shell`)
    if (prefix.length >= 2) {
        for (const sym of Array.from(stdlibRegistry.functions.values())) {
            if (sym.deprecatedReplacement) continue;
            if (
                (sym.qualifiedName.startsWith(prefix) || sym.name.startsWith(prefix)) &&
                !seen.has(sym.qualifiedName)
            ) {
                seen.add(sym.qualifiedName);
                items.push({
                    label: sym.qualifiedName,
                    insertText: sym.qualifiedName,
                    kind: "function",
                    detail: sym.signature,
                    documentation: sym.description,
                });
            }
        }
    }

    return {
        startOffset,
        endOffset: safeCursor,
        replacePrefix: prefix,
        items: items.slice(0, 14),
    };
}

function inferVariableType(code: string, varName: string): string | null {
    const escaped = varName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const assignRegex = new RegExp(`^\\s*${escaped}\\s*=\\s*(.+)$`, "m");
    const match = code.match(assignRegex);
    if (!match) return null;
    const rhs = match[1].trim();
    if (/^(f|r)?["']/.test(rhs) || rhs.startsWith("str(")) return "str";
    if (/^b["']/.test(rhs) || rhs.startsWith("bytes(") || rhs.startsWith("file.read(")) return "bytes";
    if (rhs.startsWith("[") || rhs.startsWith("list(") || rhs.startsWith("file.list(") || rhs.startsWith("process.list(")) return "list";
    if (rhs.startsWith("{") || rhs.startsWith("dict(") || rhs.startsWith("sys.info(") || rhs.startsWith("user.info(")) return "dict";
    return null;
}

/**
 * Returns active function signature help when the cursor is inside `func(...)`.
 */
export function getEldritchSignatureHelp(
    code: string,
    cursorOffset: number
): EldritchSignatureHelp | null {
    const safeCursor = Math.max(0, Math.min(cursorOffset, code.length));
    const beforeCursor = code.slice(0, safeCursor);

    // Scan backwards from cursor to find the nearest unclosed '('
    let depth = 0;
    let commaCount = 0;
    let openParenIdx = -1;

    for (let i = beforeCursor.length - 1; i >= 0; i--) {
        const ch = beforeCursor[i];
        if (ch === ")" || ch === "]" || ch === "}") {
            depth++;
        } else if (ch === "(" || ch === "[" || ch === "{") {
            if (depth > 0) {
                depth--;
            } else if (ch === "(") {
                openParenIdx = i;
                break;
            }
        } else if (ch === "," && depth === 0) {
            commaCount++;
        } else if (ch === "\n" && depth === 0) {
            // Stop at line boundary unless inside nested parens
            break;
        }
    }

    if (openParenIdx === -1) return null;

    const beforeParen = beforeCursor.slice(0, openParenIdx);
    const calleeMatch = beforeParen.match(/([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)\s*$/);
    if (!calleeMatch) return null;

    const callee = calleeMatch[1];
    const sym =
        stdlibRegistry.functions.get(callee) ||
        stdlibRegistry.builtins.get(callee) ||
        (DEPRECATED_STDLIB_FUNCTIONS[callee]
            ? stdlibRegistry.functions.get(DEPRECATED_STDLIB_FUNCTIONS[callee])
            : undefined);

    if (!sym) return null;

    const paramLabels = sym.params.map((p) =>
        p.typeHint ? `${p.name}: ${p.typeHint}` : p.name
    );

    return {
        functionName: sym.qualifiedName,
        signature: sym.signature,
        description: sym.description,
        params: paramLabels,
        activeParam: Math.min(commaCount, Math.max(0, paramLabels.length - 1)),
    };
}

/**
 * Tokenizes a single line of Eldritch code for syntax highlighting in the Composer.
 * Preserves exact characters so the highlighted `<pre>` aligns 1:1 with `<textarea>`.
 */
export function tokenizeEldritchLine(line: string): HighlightToken[] {
    const tokens: HighlightToken[] = [];
    let i = 0;

    const pushToken = (text: string, type: HighlightTokenType) => {
        if (!text) return;
        const prev = tokens[tokens.length - 1];
        if (prev && prev.type === type) {
            prev.text += text;
        } else {
            tokens.push({ text, type });
        }
    };

    while (i < line.length) {
        const ch = line[i];

        // Comment
        if (ch === "#") {
            pushToken(line.slice(i), "comment");
            break;
        }

        // Strings (including f"...", b"...", r"...")
        if (
            ch === '"' ||
            ch === "'" ||
            ((ch === "f" || ch === "b" || ch === "r") &&
                (line[i + 1] === '"' || line[i + 1] === "'"))
        ) {
            const hasPrefix = ch === "f" || ch === "b" || ch === "r";
            const quote = hasPrefix ? line[i + 1] : ch;
            let end = i + (hasPrefix ? 2 : 1);
            while (end < line.length) {
                if (line[end] === "\\") {
                    end += 2;
                    continue;
                }
                if (line[end] === quote) {
                    end++;
                    break;
                }
                end++;
            }
            pushToken(line.slice(i, end), "string");
            i = end;
            continue;
        }

        // Numbers
        if (/[0-9]/.test(ch)) {
            let end = i + 1;
            while (end < line.length && /[0-9._xXa-fA-F]/.test(line[end])) {
                end++;
            }
            pushToken(line.slice(i, end), "number");
            i = end;
            continue;
        }

        // Identifiers, keywords, modules, builtins, methods
        if (/[a-zA-Z_]/.test(ch)) {
            let end = i + 1;
            while (end < line.length && /[a-zA-Z0-9_]/.test(line[end])) {
                end++;
            }
            const word = line.slice(i, end);

            if (ELDRITCH_KEYWORDS.includes(word)) {
                pushToken(word, "keyword");
            } else if (ELDRITCH_BOOLEANS.includes(word)) {
                pushToken(word, "boolean");
            } else if (
                line[end] === "." &&
                (ELDRITCH_MODULES[word] || word === "crypto" || word === "pivot")
            ) {
                pushToken(word, "module");
            } else if (i > 0 && line[i - 1] === ".") {
                pushToken(word, "function");
            } else if (ELDRITCH_BUILTINS[word] && line.slice(end).match(/^\s*\(/)) {
                pushToken(word, "builtin");
            } else if (line.slice(end).match(/^\s*\(/)) {
                pushToken(word, "function");
            } else if (ELDRITCH_MODULES[word]) {
                pushToken(word, "module");
            } else {
                pushToken(word, "plain");
            }
            i = end;
            continue;
        }

        // Punctuation & brackets
        if ("()[]{}:,.".includes(ch)) {
            pushToken(ch, "punctuation");
            i++;
            continue;
        }

        // Operators
        if ("=+-*/%<>!&|^~".includes(ch)) {
            pushToken(ch, "operator");
            i++;
            continue;
        }

        pushToken(ch, "plain");
        i++;
    }

    return tokens;
}
