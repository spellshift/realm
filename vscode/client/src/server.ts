import * as fs from "fs";
import * as path from "path";
import { URL } from "url";
import eldritchDocs from "./eldritch-docs.json";

export interface ParamSpec {
    name: string;
    ty: string;
    defaultValue?: string;
    variadic?: boolean;
    kwargs?: boolean;
}

export interface FuncSpec {
    module?: string;
    name: string;
    params: ParamSpec[];
    returnType: string;
    doc: string;
    deprecatedReplacement?: string;
}

export interface ModuleSpec {
    name: string;
    description: string;
    deprecated: boolean;
    functions: Map<string, FuncSpec>;
}

export interface TomeParamDef {
    name: string;
    type: string;
    label: string;
    placeholder: string;
    default?: string;
}

export interface Position {
    line: number;
    character: number;
}

export interface Range {
    start: Position;
    end: Position;
}

export interface Diagnostic {
    range: Range;
    severity: number; // 1=Error, 2=Warning, 3=Information, 4=Hint
    code: string;
    source: string;
    message: string;
    tags?: number[]; // 1=Unnecessary, 2=Deprecated
}

const MODULE_DESCRIPTIONS: Record<string, [string, boolean]> = {
    aes: ["AES encryption and decryption for bytes and files.", false],
    agent: ["Agent metadata, configuration, transport, and callback control.", false],
    assets: ["Embedded tome asset inspection, reading, and copying.", false],
    base64: ["Base64 encoding and decoding.", false],
    chain: ["Multi-agent C2 chaining.", false],
    dns: ["DNS resolution and hostname lookup functions.", false],
    file: ["File system inspection, manipulation, and traversal operations.", false],
    gzip: ["Gzip compression and decompression for bytes and files.", false],
    hash: ["Cryptographic hashing (MD5, SHA-1, SHA-256, and file hashing).", false],
    http: ["HTTP/HTTPS client requests and file downloads.", false],
    json: ["JSON serialization, deserialization, and validation.", false],
    linux: ["Linux-specific system inspection (`uname`).", false],
    net: ["Network interfaces, active connections, and TCP/UDP clients.", false],
    portal: ["Bi-directional portal stream creation.", false],
    process: ["Process enumeration, termination, PID, environment, and working directory.", false],
    random: ["Cryptographically secure random generation (bool, bytes, int, string, uuid).", false],
    regex: ["Regular expression matching and replacement.", false],
    registry: ["Windows Registry reading and writing.", false],
    report: ["Structured reporting of files, credentials, and process lists to Tavern.", false],
    scan: ["Network discovery (ARP) and TCP port scanning.", false],
    ssh: ["SSH command execution, file copy, and agent deployment.", false],
    sys: ["System OS info, hostname, shell execution, and binary execution.", false],
    template: ["Jinja2 template rendering from strings and files.", false],
    time: ["Time sleep, epoch conversion, and timestamp formatting.", false],
    user: ["Current user info and system user account enumeration.", false],
    windows: ["Windows-specific DLL injection/reflection and token impersonation.", false],
    crypto: ["DEPRECATED in 0.4.1: split into `aes`, `base64`, `hash`, and `json`.", true],
    pivot: ["DEPRECATED in 0.4.1: split into `scan`, `portal`, `net`, and `ssh`.", true],
};

const REMOVED_040_FUNCTIONS: Record<string, string> = {
    "pivot.bind_proxy": "Removed in Eldritch 0.4.1 (unimplemented proposal). Use `portal.create` instead.",
    "pivot.port_forward": "Removed in Eldritch 0.4.1 (unimplemented proposal). Use `portal.create` instead.",
    "pivot.smb_exec": "Removed in Eldritch 0.4.1 (unimplemented proposal).",
};

const DEPRECATED_041_FUNCTIONS: Record<string, string> = {
    "agent.get_callback_interval": "agent.callback_interval",
    "agent.get_config": "agent.config",
    "agent.get_transport": "agent.transport",
    "assets.read_binary": "assets.read",
    "crypto.aes_decrypt": "aes.decrypt",
    "crypto.aes_decrypt_file": "aes.decrypt_file",
    "crypto.aes_encrypt": "aes.encrypt",
    "crypto.aes_encrypt_file": "aes.encrypt_file",
    "crypto.decode_b64": "base64.decode",
    "crypto.decode_utf16le": "bytes.decode(\"utf16le\")",
    "crypto.encode_b64": "base64.encode",
    "crypto.encode_utf16le": "str.encode(\"utf16le\")",
    "crypto.from_json": "json.decode",
    "crypto.hash_file": "hash.file",
    "crypto.is_json": "json.is_valid",
    "crypto.md5": "hash.md5",
    "crypto.sha1": "hash.sha1",
    "crypto.sha256": "hash.sha256",
    "crypto.to_json": "json.encode",
    "file.append_binary": "file.append",
    "file.compress": "gzip.compress_file",
    "file.decompress": "gzip.decompress_file",
    "file.pwd": "process.cwd",
    "file.read_binary": "file.read",
    "file.temp_file": "file.tmp",
    "file.template": "template.render_file",
    "file.template_str": "template.render",
    "file.write_binary": "file.write",
    "pivot.arp_scan": "scan.arp",
    "pivot.create_portal": "portal.create",
    "pivot.ncat": "net.tcp / net.udp",
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
    "time.format_to_epoch": "time.to_epoch",
    "time.format_to_readable": "time.format",
};

const BUILTIN_SPECS: Array<[string, Array<[string, string, string?]>, string, string]> = [
    ["abs", [["x", "int | float"]], "int | float", "Return the absolute value of a number."],
    ["all", [["iterable", "Iterable"]], "bool", "Return `True` if all elements of the iterable are truthy."],
    ["any", [["iterable", "Iterable"]], "bool", "Return `True` if any element of the iterable is truthy."],
    ["assert", [["condition", "Any"], ["message", "Optional[str]", "None"]], "None", "Assert that `condition` is truthy, raising a runtime error otherwise."],
    ["assert_eq", [["left", "Any"], ["right", "Any"], ["message", "Optional[str]", "None"]], "None", "Assert that `left == right`, raising a runtime error if they differ."],
    ["bool", [["x", "Any", "False"]], "bool", "Convert a value to a boolean (`True` or `False`)."],
    ["bytes", [["source", "Any", "b\"\""]], "Bytes", "Construct an immutable `Bytes` buffer from a UTF-8 string, list of byte ints, or bytes."],
    ["chr", [["i", "int"]], "str", "Return the string representing a character whose Unicode code point is `i`."],
    ["dict", [["*args", "Any"], ["**kwargs", "Any"]], "Dict", "Create a new dictionary from key-value pairs or keyword arguments."],
    ["dir", [["value", "Optional[Any]", "None"]], "List[str]", "Return a sorted list of attribute names or names in the current scope."],
    ["divmod", [["a", "int | float"], ["b", "int | float"]], "Tuple[int | float, int | float]", "Return the pair `(a // b, a % b)`."],
    ["enumerate", [["iterable", "Iterable"], ["start", "int", "0"]], "List[Tuple[int, Any]]", "Return a list of `(index, element)` tuples from `iterable` starting at `start`."],
    ["eprint", [["*args", "Any"]], "None", "Print values to `stderr`."],
    ["eval", [["source", "str"]], "Any", "Evaluate an Eldritch expression string in the current environment."],
    ["fail", [["message", "str"]], "None", "Immediately abort execution with an error `message`."],
    ["filter", [["function", "Callable | None"], ["iterable", "Iterable"]], "List[Any]", "Construct a list from those elements of `iterable` for which `function` returns true."],
    ["float", [["x", "int | float | str", "0.0"]], "float", "Convert a number or string to a floating-point number."],
    ["getattr", [["object", "Any"], ["name", "str"], ["default", "Optional[Any]", "None"]], "Any", "Return the value of the named attribute of `object`."],
    ["hasattr", [["object", "Any"], ["name", "str"]], "bool", "Return `True` if `object` has an attribute with the given `name`."],
    ["hash", [["value", "Any"]], "int", "Return the integer hash value of `value`."],
    ["hex", [["x", "int"]], "str", "Convert an integer number to a lowercase hexadecimal string prefixed with `0x`."],
    ["int", [["x", "int | float | str | bool", "0"], ["base", "int", "10"]], "int", "Convert a number or string to an integer."],
    ["isinstance", [["object", "Any"], ["classinfo", "str | Tuple[str, ...]"]], "bool", "Return `True` if `object` is an instance of the specified type."],
    ["len", [["s", "Sized"]], "int", "Return the number of items in a container (`str`, `Bytes`, `List`, `Tuple`, `Dict`, `Set`)."],
    ["list", [["iterable", "Optional[Iterable]", "[]"]], "List[Any]", "Create a mutable list from an iterable."],
    ["map", [["function", "Callable"], ["iterable", "Iterable"]], "List[Any]", "Apply `function` to every item of `iterable` and return a list of the results."],
    ["max", [["*args", "Any"]], "Any", "Return the largest item in an iterable or the largest of two or more arguments."],
    ["min", [["*args", "Any"]], "Any", "Return the smallest item in an iterable or the smallest of two or more arguments."],
    ["oct", [["x", "int"]], "str", "Convert an integer number to an octal string prefixed with `0o`."],
    ["ord", [["c", "str"]], "int", "Return the Unicode code point for a one-character string."],
    ["pprint", [["value", "Any"], ["indent", "int", "2"]], "None", "Pretty-print an Eldritch value."],
    ["print", [["*args", "Any"]], "None", "Print values to `stdout`."],
    ["range", [["start_or_stop", "int"], ["stop", "Optional[int]", "None"], ["step", "int", "1"]], "List[int]", "Return a list of integers in the specified range."],
    ["repr", [["object", "Any"]], "str", "Return a string containing a printable representation of an object."],
    ["reversed", [["seq", "Iterable"]], "List[Any]", "Return a new list with elements of `seq` in reverse order."],
    ["set", [["iterable", "Optional[Iterable]", "set()"]], "Set[Any]", "Create a new `Set` object, optionally populated with elements from `iterable`."],
    ["sorted", [["iterable", "Iterable"], ["key", "Optional[Callable]", "None"], ["reverse", "bool", "False"]], "List[Any]", "Return a new sorted list from the items in `iterable`."],
    ["str", [["object", "Any", "\"\""]], "str", "Return a string version of `object` (converts `Bytes` assuming UTF-8)."],
    ["sum", [["iterable", "Iterable"], ["start", "int | float", "0"]], "int | float", "Sum `start` and the items of an `iterable` from left to right."],
    ["tuple", [["iterable", "Optional[Iterable]", "()"]], "Tuple", "Create an immutable tuple from an iterable."],
    ["type", [["object", "Any"]], "str", "Return the type name of `object` as a string."],
    ["zip", [["*iterables", "Iterable"]], "List[Tuple]", "Iterate over several iterables in parallel, producing tuples with an item from each one."],
];

const KEYWORDS = [
    "def", "if", "elif", "else", "for", "in", "return", "break", "continue", "pass",
    "and", "or", "not", "is", "lambda", "True", "False", "None",
];

function parseSignatureParams(sig: string): { params: ParamSpec[]; returnType: string } {
    const openIdx = sig.indexOf("(");
    const closeIdx = sig.lastIndexOf(")");
    let returnType = "Any";
    if (closeIdx !== -1) {
        const after = sig.slice(closeIdx + 1);
        const arrowIdx = after.indexOf("->");
        if (arrowIdx !== -1) {
            returnType = after.slice(arrowIdx + 2).trim() || "Any";
        }
    }
    if (openIdx === -1 || closeIdx === -1 || closeIdx < openIdx) {
        return { params: [], returnType };
    }
    const inner = sig.slice(openIdx + 1, closeIdx).trim();
    if (!inner) {
        return { params: [], returnType };
    }
    const parts: string[] = [];
    let current = "";
    let depth = 0;
    for (const ch of inner) {
        if (ch === "[" || ch === "(" || ch === "{") depth++;
        else if (ch === "]" || ch === ")" || ch === "}") depth--;
        else if (ch === "," && depth === 0) {
            parts.push(current.trim());
            current = "";
            continue;
        }
        current += ch;
    }
    if (current.trim()) parts.push(current.trim());

    const params: ParamSpec[] = [];
    for (const p of parts) {
        let defaultValue: string | undefined;
        let left = p;
        const eqIdx = p.indexOf("=");
        if (eqIdx !== -1) {
            left = p.slice(0, eqIdx).trim();
            defaultValue = p.slice(eqIdx + 1).trim();
        }
        let name = left;
        let ty = "Any";
        const colonIdx = left.indexOf(":");
        if (colonIdx !== -1) {
            name = left.slice(0, colonIdx).trim();
            ty = left.slice(colonIdx + 1).trim();
        }
        const kwargs = name.startsWith("**");
        const variadic = !kwargs && name.startsWith("*");
        const cleanName = name.replace(/^\*+/, "");
        if (cleanName) {
            params.push({ name: cleanName, ty, defaultValue, variadic, kwargs });
        }
    }
    return { params, returnType };
}

export class StdlibCatalog {
    public modules = new Map<string, ModuleSpec>();
    public builtins = new Map<string, FuncSpec>();

    constructor() {
        for (const [name, [description, deprecated]] of Object.entries(MODULE_DESCRIPTIONS)) {
            this.modules.set(name, { name, description, deprecated, functions: new Map() });
        }
        const docsEntries = eldritchDocs as Record<string, { signature?: string; description?: string }>;
        for (const [key, entry] of Object.entries(docsEntries)) {
            if (!key.includes(".")) continue;
            const [modName, fnName] = key.split(".", 2);
            const { params, returnType } = parseSignatureParams(entry.signature || `${modName}.${fnName}()`);
            const mod = this.modules.get(modName);
            if (mod) {
                mod.functions.set(fnName, {
                    module: modName,
                    name: fnName,
                    params,
                    returnType,
                    doc: entry.description || "",
                });
            }
        }
        for (const [name, rawParams, returnType, doc] of BUILTIN_SPECS) {
            const params: ParamSpec[] = rawParams.map(([pName, ty, def]) => ({
                name: pName.replace(/^\*+/, ""),
                ty,
                defaultValue: def,
                variadic: pName.startsWith("*") && !pName.startsWith("**"),
                kwargs: pName.startsWith("**"),
            }));
            this.builtins.set(name, { name, params, returnType, doc });
        }
    }
}

export function formatSignature(spec: FuncSpec): string {
    const prefix = spec.module ? `${spec.module}.${spec.name}` : spec.name;
    const params = spec.params
        .map((p) => {
            const stars = p.kwargs ? "**" : p.variadic ? "*" : "";
            const def = p.defaultValue !== undefined ? ` = ${p.defaultValue}` : "";
            return `${stars}${p.name}: ${p.ty}${def}`;
        })
        .join(", ");
    return `${prefix}(${params}) -> ${spec.returnType}`;
}

export function loadSiblingMetadata(uriStr?: string): TomeParamDef[] {
    if (!uriStr || !uriStr.startsWith("file://")) return [];
    try {
        const filePath = new URL(uriStr).pathname;
        const dir = path.dirname(filePath);
        for (const candidate of ["metadata.yml", "metadata.yaml"]) {
            const full = path.join(dir, candidate);
            if (fs.existsSync(full)) {
                const text = fs.readFileSync(full, "utf8");
                const defs: TomeParamDef[] = [];
                const lines = text.split(/\r?\n/);
                let current: Partial<TomeParamDef> | null = null;
                for (const line of lines) {
                    const nameMatch = line.match(/^\s*-\s*name:\s*(.+)$/);
                    if (nameMatch) {
                        if (current && current.name) defs.push(current as TomeParamDef);
                        current = {
                            name: nameMatch[1].trim().replace(/^['"]|['"]$/g, ""),
                            type: "string",
                            label: "",
                            placeholder: "",
                        };
                        continue;
                    }
                    if (current) {
                        const fieldMatch = line.match(/^\s+(type|label|placeholder|default):\s*(.*)$/);
                        if (fieldMatch) {
                            const key = fieldMatch[1] as keyof TomeParamDef;
                            current[key] = fieldMatch[2].trim().replace(/^['"]|['"]$/g, "");
                        }
                    }
                }
                if (current && current.name) defs.push(current as TomeParamDef);
                return defs;
            }
        }
    } catch {
        // ignore
    }
    return [];
}

export function analyzeEldritchText(
    text: string,
    catalog: StdlibCatalog,
    uriStr?: string
): {
    diagnostics: Diagnostic[];
    userFuncs: Map<string, { line: number; params: string[]; doc: string }>;
    userVars: Map<string, { line: number }>;
    paramDefs: TomeParamDef[];
} {
    const diagnostics: Diagnostic[] = [];
    const userFuncs = new Map<string, { line: number; params: string[]; doc: string }>();
    const userVars = new Map<string, { line: number }>();
    const paramDefs = loadSiblingMetadata(uriStr);
    const lines = text.split(/\r?\n/);

    // Pass 1: collect functions and top-level/local assignments
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const defMatch = line.match(/^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)/);
        if (defMatch) {
            const fnName = defMatch[1];
            const rawParams = defMatch[2]
                .split(",")
                .map((s) => s.trim().split(":")[0].split("=")[0].replace(/^\*+/, "").trim())
                .filter(Boolean);
            userFuncs.set(fnName, { line: i, params: rawParams, doc: `User-defined function \`${fnName}\`` });
            for (const p of rawParams) {
                userVars.set(p, { line: i });
            }
        }
        const forMatch = line.match(/^\s*for\s+([A-Za-z_][A-Za-z0-9_,\s]*)\s+in\s+/);
        if (forMatch) {
            for (const v of forMatch[1].split(",").map((s) => s.trim()).filter(Boolean)) {
                userVars.set(v, { line: i });
            }
        }
        const assignMatch = line.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=]+)?=/);
        if (assignMatch && !line.trim().startsWith("if ") && !line.trim().startsWith("elif ")) {
            const varName = assignMatch[1];
            if (!userVars.has(varName)) {
                userVars.set(varName, { line: i });
            }
            if (catalog.modules.has(varName)) {
                const col = line.indexOf(varName);
                diagnostics.push({
                    range: { start: { line: i, character: col }, end: { line: i, character: col + varName.length } },
                    severity: 2,
                    code: "shadowed-stdlib",
                    source: "eldritch-lsp",
                    message: `Variable \`${varName}\` shadows the \`${varName}\` standard library module.`,
                });
            }
        }
    }

    // Pass 2: check stdlib calls, removed functions, deprecated functions, and input_params keys
    const callRegex = /\b([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)\s*\(/g;
    for (let i = 0; i < lines.length; i++) {
        const rawLine = lines[i];
        const codePart = rawLine.split("#")[0];
        let match: RegExpExecArray | null;
        callRegex.lastIndex = 0;
        while ((match = callRegex.exec(codePart)) !== null) {
            const modName = match[1];
            const fnName = match[2];
            const startCol = match.index;
            const endCol = startCol + modName.length + 1 + fnName.length;
            const qualified = `${modName}.${fnName}`;

            if (REMOVED_040_FUNCTIONS[qualified]) {
                diagnostics.push({
                    range: { start: { line: i, character: startCol }, end: { line: i, character: endCol } },
                    severity: 1,
                    code: "removed-function",
                    source: "eldritch-lsp",
                    message: REMOVED_040_FUNCTIONS[qualified],
                });
                continue;
            }

            if (DEPRECATED_041_FUNCTIONS[qualified]) {
                diagnostics.push({
                    range: { start: { line: i, character: startCol }, end: { line: i, character: endCol } },
                    severity: 2,
                    code: "deprecated-function",
                    source: "eldritch-lsp",
                    message: `\`${qualified}\` is deprecated in Eldritch 0.4.1. Use \`${DEPRECATED_041_FUNCTIONS[qualified]}\` instead.`,
                    tags: [2],
                });
                continue;
            }

            const mod = catalog.modules.get(modName);
            if (mod && !userVars.has(modName)) {
                const fnSpec = mod.functions.get(fnName);
                if (!fnSpec) {
                    diagnostics.push({
                        range: { start: { line: i, character: startCol }, end: { line: i, character: endCol } },
                        severity: 1,
                        code: "unknown-module-function",
                        source: "eldritch-lsp",
                        message: `Module \`${modName}\` has no function \`${fnName}\`.`,
                    });
                } else if (mod.deprecated || fnSpec.deprecatedReplacement) {
                    diagnostics.push({
                        range: { start: { line: i, character: startCol }, end: { line: i, character: endCol } },
                        severity: 2,
                        code: "deprecated-function",
                        source: "eldritch-lsp",
                        message: `\`${qualified}\` is deprecated in Eldritch 0.4.1.`,
                        tags: [2],
                    });
                }
            }
        }

        if (paramDefs.length > 0) {
            const paramRegex = /\binput_params\s*(?:\[\s*['"]([^'"]+)['"]\s*\]|\.get\s*\(\s*['"]([^'"]+)['"])/g;
            let pm: RegExpExecArray | null;
            while ((pm = paramRegex.exec(codePart)) !== null) {
                const key = pm[1] || pm[2];
                if (key && !paramDefs.some((d) => d.name === key)) {
                    const valid = paramDefs.map((d) => `\`${d.name}\``).join(", ");
                    diagnostics.push({
                        range: {
                            start: { line: i, character: pm.index },
                            end: { line: i, character: pm.index + pm[0].length },
                        },
                        severity: 2,
                        code: "unknown-input-param",
                        source: "eldritch-lsp",
                        message: `Parameter \`${key}\` is not declared in sibling \`metadata.yml\` (declared: ${valid}).`,
                    });
                }
            }
        }
    }

    return { diagnostics, userFuncs, userVars, paramDefs };
}

function startStdioServer(): void {
    const catalog = new StdlibCatalog();
    const documents = new Map<string, string>();
    let buffer = Buffer.alloc(0);

    function sendMessage(msg: any): void {
        const json = JSON.stringify(msg);
        const byteLen = Buffer.byteLength(json, "utf8");
        process.stdout.write(`Content-Length: ${byteLen}\r\n\r\n${json}`);
    }

    function publishDiagnostics(uri: string, text: string): void {
        const { diagnostics } = analyzeEldritchText(text, catalog, uri);
        sendMessage({
            jsonrpc: "2.0",
            method: "textDocument/publishDiagnostics",
            params: { uri, diagnostics },
        });
    }

    function handleMessage(msg: any): void {
        const { id, method, params } = msg;
        if (method === "initialize") {
            sendMessage({
                jsonrpc: "2.0",
                id,
                result: {
                    capabilities: {
                        textDocumentSync: 1,
                        completionProvider: {
                            resolveProvider: false,
                            triggerCharacters: [".", "[", "'", "\""],
                        },
                        hoverProvider: true,
                        signatureHelpProvider: {
                            triggerCharacters: ["(", ","],
                        },
                        definitionProvider: true,
                        documentSymbolProvider: true,
                    },
                    serverInfo: {
                        name: "eldritch-lsp-fallback",
                        version: "0.4.1",
                    },
                },
            });
        } else if (method === "shutdown") {
            sendMessage({ jsonrpc: "2.0", id, result: null });
        } else if (method === "exit") {
            process.exit(0);
        } else if (method === "textDocument/didOpen") {
            const doc = params?.textDocument;
            if (doc?.uri && typeof doc.text === "string") {
                documents.set(doc.uri, doc.text);
                publishDiagnostics(doc.uri, doc.text);
            }
        } else if (method === "textDocument/didChange") {
            const uri = params?.textDocument?.uri;
            const changes = params?.contentChanges;
            if (uri && Array.isArray(changes) && changes.length > 0) {
                const text = changes[changes.length - 1].text;
                documents.set(uri, text);
                publishDiagnostics(uri, text);
            }
        } else if (method === "textDocument/didSave") {
            const uri = params?.textDocument?.uri;
            if (uri && documents.has(uri)) {
                publishDiagnostics(uri, documents.get(uri)!);
            }
        } else if (method === "textDocument/didClose") {
            const uri = params?.textDocument?.uri;
            if (uri) {
                documents.delete(uri);
                sendMessage({
                    jsonrpc: "2.0",
                    method: "textDocument/publishDiagnostics",
                    params: { uri, diagnostics: [] },
                });
            }
        } else if (method === "textDocument/completion") {
            const uri = params?.textDocument?.uri;
            const pos: Position = params?.position || { line: 0, character: 0 };
            const text = (uri && documents.get(uri)) || "";
            const line = (text.split(/\r?\n/)[pos.line] || "").slice(0, pos.character);
            const items: any[] = [];

            const dotMatch = line.match(/([A-Za-z_][A-Za-z0-9_]*)\.[A-Za-z_0-9]*$/);
            const paramBracketMatch = line.match(/input_params\[['"]?[A-Za-z0-9_-]*$/);

            if (paramBracketMatch) {
                const paramDefs = loadSiblingMetadata(uri);
                for (const p of paramDefs) {
                    items.push({
                        label: p.name,
                        kind: 12,
                        detail: `input_params (${p.type})`,
                        documentation: p.label,
                    });
                }
            } else if (dotMatch) {
                const mod = catalog.modules.get(dotMatch[1]);
                if (mod) {
                    for (const fn of mod.functions.values()) {
                        items.push({
                            label: fn.name,
                            kind: 3,
                            detail: formatSignature(fn),
                            documentation: { kind: "markdown", value: fn.doc },
                        });
                    }
                }
            } else {
                for (const mod of catalog.modules.values()) {
                    items.push({
                        label: mod.name,
                        kind: 9,
                        detail: mod.deprecated ? `${mod.name} (deprecated)` : `${mod.name} module`,
                        documentation: { kind: "markdown", value: mod.description },
                    });
                }
                for (const b of catalog.builtins.values()) {
                    items.push({
                        label: b.name,
                        kind: 3,
                        detail: formatSignature(b),
                        documentation: { kind: "markdown", value: b.doc },
                    });
                }
                for (const kw of KEYWORDS) {
                    items.push({ label: kw, kind: 14, detail: "keyword" });
                }
            }
            sendMessage({ jsonrpc: "2.0", id, result: items });
        } else if (method === "textDocument/hover") {
            const uri = params?.textDocument?.uri;
            const pos: Position = params?.position || { line: 0, character: 0 };
            const text = (uri && documents.get(uri)) || "";
            const line = text.split(/\r?\n/)[pos.line] || "";
            let start = pos.character;
            let end = pos.character;
            while (start > 0 && /[A-Za-z0-9_]/.test(line[start - 1])) start--;
            while (end < line.length && /[A-Za-z0-9_]/.test(line[end])) end++;
            const word = line.slice(start, end);
            let hoverMd: string | null = null;
            if (start > 1 && line[start - 1] === ".") {
                let rStart = start - 1;
                while (rStart > 0 && /[A-Za-z0-9_]/.test(line[rStart - 1])) rStart--;
                const modName = line.slice(rStart, start - 1);
                const fnSpec = catalog.modules.get(modName)?.functions.get(word);
                if (fnSpec) {
                    hoverMd = `\`\`\`eldritch\n${formatSignature(fnSpec)}\n\`\`\`\n\n${fnSpec.doc}`;
                }
            } else if (catalog.modules.has(word)) {
                const mod = catalog.modules.get(word)!;
                hoverMd = `### Module \`${mod.name}\`\n\n${mod.description}`;
            } else if (catalog.builtins.has(word)) {
                const b = catalog.builtins.get(word)!;
                hoverMd = `\`\`\`eldritch\n${formatSignature(b)}\n\`\`\`\n\n${b.doc}`;
            }
            sendMessage({
                jsonrpc: "2.0",
                id,
                result: hoverMd
                    ? { contents: { kind: "markdown", value: hoverMd } }
                    : null,
            });
        } else if (id !== undefined) {
            sendMessage({ jsonrpc: "2.0", id, result: null });
        }
    }

    process.stdin.on("data", (chunk: Buffer) => {
        buffer = Buffer.concat([buffer, chunk]);
        while (true) {
            const headerEnd = buffer.indexOf("\r\n\r\n");
            if (headerEnd === -1) break;
            const headerStr = buffer.slice(0, headerEnd).toString("ascii");
            const lenMatch = headerStr.match(/Content-Length:\s*(\d+)/i);
            if (!lenMatch) {
                buffer = buffer.slice(headerEnd + 4);
                continue;
            }
            const contentLen = parseInt(lenMatch[1], 10);
            const totalLen = headerEnd + 4 + contentLen;
            if (buffer.length < totalLen) break;
            const bodyStr = buffer.slice(headerEnd + 4, totalLen).toString("utf8");
            buffer = buffer.slice(totalLen);
            try {
                handleMessage(JSON.parse(bodyStr));
            } catch {
                // ignore malformed json
            }
        }
    });
}

if (require.main === module) {
    startStdioServer();
}
