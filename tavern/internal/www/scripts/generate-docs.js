const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "../../../..");
const userGuideDocsPath = path.join(repoRoot, "docs/_docs/user-guide/eldritch.md");
const agentsMdPath = path.join(repoRoot, "AGENTS.md");
const migrateScriptPath = path.join(repoRoot, "bin/migrate/eldritch_migrate_0.4.1.py");
const stdlibRootDir = path.join(repoRoot, "implants/lib/eldritch/stdlib");
const eldritchCoreSrcDir = path.join(repoRoot, "implants/lib/eldritch/eldritch-core/src");
const builtinsDir = path.join(eldritchCoreSrcDir, "interpreter/builtins");
const methodsModPath = path.join(eldritchCoreSrcDir, "interpreter/methods/mod.rs");
const methodsDocPath = path.join(eldritchCoreSrcDir, "docs/methods.rs");
const tokenRsPath = path.join(eldritchCoreSrcDir, "token.rs");
const outputPath = path.resolve(__dirname, "../src/assets/eldritch-docs.json");

function mapRustTypeToEldritch(rustType) {
    const t = (rustType || "").trim().replace(/\s+/g, " ");
    if (!t || t === "()") return "None";
    if (t === "String" || t === "&str") return "str";
    if (["i64", "i32", "u64", "u32", "usize", "isize"].includes(t)) return "int";
    if (t === "f64" || t === "f32") return "float";
    if (t === "bool") return "bool";
    if (t === "Vec<u8>" || t === "&[u8]") return "Bytes";
    if (t === "Value") return "Any";
    if (t === "TaskWrapper") return "Dict";

    if (t.startsWith("Option<") && t.endsWith(">")) {
        const inner = t.slice(7, -1).trim();
        return `Optional<${mapRustTypeToEldritch(inner)}>`;
    }
    if (t.startsWith("Vec<") && t.endsWith(">")) {
        const inner = t.slice(4, -1).trim();
        return `List<${mapRustTypeToEldritch(inner)}>`;
    }
    if (t.startsWith("BTreeMap<") && t.endsWith(">")) {
        const inner = t.slice(9, -1).trim();
        const commaIdx = inner.indexOf(",");
        if (commaIdx !== -1) {
            const k = mapRustTypeToEldritch(inner.slice(0, commaIdx));
            const v = mapRustTypeToEldritch(inner.slice(commaIdx + 1));
            return `Dict<${k}, ${v}>`;
        }
        return "Dict";
    }
    return t;
}

function normalizeDocType(rawType) {
    if (!rawType) return "";
    let t = rawType.trim().replace(/^`|`$/g, "");
    t = t
        .replace(/\bOption<([^>]+)>/g, "Optional<$1>")
        .replace(/\bString\b/g, "str")
        .replace(/\bInt\b/g, "int")
        .replace(/\bFloat\b/g, "float")
        .replace(/\bBool\b/g, "bool");
    return t;
}

function parseDocMetadata(docText) {
    const paramTypes = {};
    let returnType = null;
    const lines = (docText || "").split("\n");
    let section = null;

    for (const rawLine of lines) {
        const line = rawLine.trim();
        if (/^\*\*Parameters\*\*/i.test(line)) {
            section = "params";
            continue;
        }
        if (/^\*\*Returns\*\*/i.test(line)) {
            section = "returns";
            continue;
        }
        if (/^\*\*[A-Za-z]+\*\*/.test(line)) {
            section = "other";
            continue;
        }

        if (section === "params") {
            const m = line.match(/^-\s*`?(\*{0,2}[a-zA-Z0-9_]+)`?\s*\(`?([^)`]+)`?\)/);
            if (m) {
                paramTypes[m[1]] = normalizeDocType(m[2]);
            }
        } else if (section === "returns" && !returnType) {
            const m = line.match(/^-\s*`([^`]+)`/);
            if (m) {
                returnType = normalizeDocType(m[1]);
            }
        }
    }

    return { paramTypes, returnType };
}

function parseSignatureString(qualifiedName, signature, fallbackReturnType = "Any") {
    const clean = (signature || "").trim();
    const openIdx = clean.indexOf("(");
    if (openIdx === -1) {
        return { params: [], returnType: fallbackReturnType };
    }

    let depth = 0;
    let closeIdx = -1;
    for (let i = openIdx; i < clean.length; i++) {
        const ch = clean[i];
        if (ch === "(" || ch === "[" || ch === "<") depth++;
        else if (ch === ")" || ch === "]" || ch === ">") {
            depth--;
            if (depth === 0 && ch === ")") {
                closeIdx = i;
                break;
            }
        }
    }

    if (closeIdx === -1) {
        return { params: [], returnType: fallbackReturnType };
    }

    const paramsPart = clean.slice(openIdx + 1, closeIdx).trim();
    const afterClose = clean.slice(closeIdx + 1).trim();
    const returnMatch = afterClose.match(/^->\s*(.+)$/);
    const returnType = returnMatch ? normalizeDocType(returnMatch[1].trim()) : fallbackReturnType;

    if (!paramsPart) {
        return { params: [], returnType };
    }

    const rawSegments = [];
    let current = "";
    let segDepth = 0;
    for (let i = 0; i < paramsPart.length; i++) {
        const ch = paramsPart[i];
        if (ch === "(" || ch === "[" || ch === "<" || ch === "{") segDepth++;
        else if (ch === ")" || ch === "]" || ch === ">" || ch === "}") segDepth--;
        if (ch === "," && segDepth === 0) {
            rawSegments.push(current.trim());
            current = "";
        } else {
            current += ch;
        }
    }
    if (current.trim()) {
        rawSegments.push(current.trim());
    }

    const params = [];
    for (const seg of rawSegments) {
        const cleanedSeg = seg.replace(/^\[+|\]+$/g, "").trim();
        if (!cleanedSeg) continue;
        let left = cleanedSeg;
        let defaultValue;
        const eqIdx = cleanedSeg.indexOf("=");
        if (eqIdx !== -1) {
            left = cleanedSeg.slice(0, eqIdx).trim();
            defaultValue = cleanedSeg.slice(eqIdx + 1).trim();
        }
        let name = left;
        let type = "Any";
        const colonIdx = left.indexOf(":");
        if (colonIdx !== -1) {
            name = left.slice(0, colonIdx).trim();
            type = normalizeDocType(left.slice(colonIdx + 1).trim());
        }
        if (
            defaultValue === undefined &&
            (type.startsWith("Optional<") ||
                type.startsWith("Option<") ||
                type.startsWith("Optional[") ||
                seg.startsWith("["))
        ) {
            defaultValue = "None";
        }
        const paramObj = { name, type };
        if (defaultValue !== undefined) {
            paramObj.defaultValue = defaultValue;
        }
        params.push(paramObj);
    }

    return { params, returnType };
}

function formatSignature(qualifiedName, params, returnType) {
    const paramStrs = params.map((p) => {
        const base = p.type ? `${p.name}: ${p.type}` : p.name;
        return p.defaultValue !== undefined ? `${base} = ${p.defaultValue}` : base;
    });
    return `${qualifiedName}(${paramStrs.join(", ")}) -> ${returnType || "Any"}`;
}

function parseUserGuideMarkdown() {
    const mdDocs = {};
    if (!fs.existsSync(userGuideDocsPath)) {
        return mdDocs;
    }

    const content = fs.readFileSync(userGuideDocsPath, "utf8");
    const lines = content.split("\n");
    let currentLibrary = null;
    let currentFunction = null;
    let buffer = [];

    const saveCurrent = () => {
        if (!currentFunction) return;
        const text = buffer.join("\n").trim();
        let signature = "";
        let description = text;

        const codeBlockRegex = /^```(?:python|eldritch)?\n([\s\S]*?)\n```/;
        const match = text.match(codeBlockRegex);
        if (match) {
            signature = match[1].trim();
            description = text.slice(match[0].length).trim();
        } else if (currentFunction !== currentLibrary) {
            signature = currentFunction + "()";
        }

        mdDocs[currentFunction] = { signature, description };
        buffer = [];
    };

    for (const line of lines) {
        if (line.startsWith("# ")) {
            saveCurrent();
            currentLibrary = null;
            currentFunction = null;
            continue;
        }
        if (line.startsWith("## ")) {
            saveCurrent();
            const header = line.slice(3).trim();
            if (header === "Standard Library" || header === "Built-in Functions") {
                currentLibrary = header === "Built-in Functions" ? "" : null;
                currentFunction = null;
            } else {
                currentLibrary = header;
                currentFunction = header;
            }
            continue;
        }
        if (line.startsWith("### ")) {
            saveCurrent();
            const funcName = line.slice(4).trim();
            if (currentLibrary !== null) {
                currentFunction = currentLibrary ? `${currentLibrary}.${funcName}` : funcName;
            } else {
                currentFunction = null;
            }
            continue;
        }
        if (currentFunction) {
            buffer.push(line);
        }
    }
    saveCurrent();
    return mdDocs;
}

function parseKeywordsFromTokenRs() {
    const keywords = [];
    const booleans = [];
    if (!fs.existsSync(tokenRsPath)) {
        return { keywords, booleans };
    }
    const src = fs.readFileSync(tokenRsPath, "utf8");
    const fnIdx = src.indexOf("pub fn from_keyword");
    if (fnIdx === -1) {
        return { keywords, booleans };
    }
    const fnBody = src.slice(fnIdx);
    const re = /"([A-Za-z_]+)"\s*=>\s*Some\(TokenKind::[A-Za-z]+\)/g;
    let m;
    while ((m = re.exec(fnBody)) !== null) {
        const word = m[1];
        if (word === "True" || word === "False" || word === "None") {
            booleans.push(word);
        } else {
            keywords.push(word);
        }
    }
    return { keywords, booleans };
}

function parseAgentsChangelog() {
    const renames = {};
    const removed = {};
    if (!fs.existsSync(agentsMdPath)) {
        return { renames, removed };
    }
    const src = fs.readFileSync(agentsMdPath, "utf8");
    const lines = src.split("\n");
    let section = null;
    for (const rawLine of lines) {
        const line = rawLine.trim();
        if (line.startsWith("### Renamed Functions")) {
            section = "renamed";
            continue;
        }
        if (line.startsWith("### Removed Documentation Proposals")) {
            section = "removed";
            continue;
        }
        if (line.startsWith("#")) {
            section = null;
            continue;
        }
        if (section === "renamed") {
            const m = line.match(/^-\s*`([a-zA-Z0-9_.]+)`\s*→\s*`([^`]+)`/);
            if (m) {
                const oldName = m[1].trim();
                const newTarget = m[2].trim();
                renames[oldName] = newTarget;
            }
        } else if (section === "removed") {
            const matches = line.match(/`([a-zA-Z0-9_]+\.[a-zA-Z0-9_]+)`/g) || [];
            for (const rawMatch of matches) {
                const fnName = rawMatch.slice(1, -1);
                removed[fnName] = "Removed from Eldritch 0.4.1 (unimplemented proposal).";
            }
        }
    }

    if (fs.existsSync(migrateScriptPath)) {
        const migrateSrc = fs.readFileSync(migrateScriptPath, "utf8");
        const renamesIdx = migrateSrc.indexOf("RENAMES:");
        const stdlibIdx = migrateSrc.indexOf("STDLIB_METHODS:");
        if (renamesIdx !== -1 && stdlibIdx !== -1) {
            const slice = migrateSrc.slice(renamesIdx, stdlibIdx);
            const pairRe = /\(\s*"([a-zA-Z0-9_.]+)"\s*,\s*"([a-zA-Z0-9_.]+)"\s*\)/g;
            let pm;
            while ((pm = pairRe.exec(slice)) !== null) {
                if (!renames[pm[1]]) {
                    renames[pm[1]] = pm[2];
                }
            }
        }
    }

    return { renames, removed };
}

function parseRustStdlib(mdDocs, changelog) {
    const entries = {};
    if (!fs.existsSync(stdlibRootDir)) {
        return entries;
    }

    const crateDirs = fs.readdirSync(stdlibRootDir).sort();
    for (const dir of crateDirs) {
        const libRsPath = path.join(stdlibRootDir, dir, "src/lib.rs");
        if (!fs.existsSync(libRsPath)) continue;

        const src = fs.readFileSync(libRsPath, "utf8");
        const modMatch = src.match(/#\[eldritch_library\("([^"]+)"\)\]/);
        if (!modMatch) continue;
        const modName = modMatch[1];

        // Extract module doc comment between #[eldritch_library] (or before it) and `pub trait`
        const traitIdx = src.indexOf("pub trait ", modMatch.index);
        if (traitIdx === -1) continue;
        const headerSlice = src.slice(0, traitIdx);
        const modDocLines = [];
        for (const line of headerSlice.split("\n")) {
            const trimmed = line.trim();
            if (trimmed.startsWith("///")) {
                modDocLines.push(trimmed.replace(/^\/\/\/\s?/, ""));
            }
        }
        const rustModDoc = modDocLines.join("\n").trim();
        const mdModDoc = mdDocs[modName]?.description || "";
        const modDescription = mdModDoc || rustModDoc || `Eldritch \`${modName}\` standard library module.`;

        const braceStart = src.indexOf("{", traitIdx);
        const braceEnd = src.lastIndexOf("}");
        const traitLines = src.slice(braceStart + 1, braceEnd).split("\n");

        const traitItems = [];
        let currentItemLines = [];
        for (const line of traitLines) {
            currentItemLines.push(line);
            const trimmed = line.trim();
            const codeOnly = trimmed.startsWith("//") ? "" : trimmed.replace(/\/\/.*$/, "").trim();
            if (codeOnly.endsWith(";")) {
                traitItems.push(currentItemLines.join("\n"));
                currentItemLines = [];
            }
        }

        let totalMethods = 0;
        let deprecatedMethods = 0;

        for (const item of traitItems) {
            if (!item.includes("#[eldritch_method")) continue;
            const attrMatch = item.match(/#\[eldritch_method(?:\("([^"]+)"\))?\]/);
            const depMatch = item.match(/#\[eldritch_deprecated\(\s*"((?:\\.|[^"\\])*)"\s*\)\]/);

            const docLines = [];
            const codeLines = [];
            for (const line of item.split("\n")) {
                const trimmed = line.trim();
                if (trimmed.startsWith("///")) {
                    docLines.push(trimmed.replace(/^\/\/\/\s?/, ""));
                } else if (!trimmed.startsWith("//") && !trimmed.startsWith("#[")) {
                    codeLines.push(line.replace(/\/\/.*$/, ""));
                }
            }

            const codeStr = codeLines.join("\n").trim();
            const fnMatch = codeStr.match(
                /fn\s+(?:r#)?([a-zA-Z0-9_]+)\s*\(([\s\S]*?)\)\s*(?:->\s*([\s\S]+?))?\s*;$/
            );
            if (!fnMatch) continue;

            const fnName = attrMatch && attrMatch[1] ? attrMatch[1] : fnMatch[1];
            const rawParamsStr = fnMatch[2].trim();
            const rawReturnStr = (fnMatch[3] || "()").trim();
            const qualifiedName = `${modName}.${fnName}`;

            const rustDoc = docLines.join("\n").trim();
            const docMeta = parseDocMetadata(rustDoc);

            // Parse Rust parameters
            const rawParamParts = [];
            let cur = "";
            let depth = 0;
            for (let i = 0; i < rawParamsStr.length; i++) {
                const ch = rawParamsStr[i];
                if (ch === "<" || ch === "(" || ch === "[") depth++;
                else if (ch === ">" || ch === ")" || ch === "]") depth--;
                if (ch === "," && depth === 0) {
                    rawParamParts.push(cur.trim());
                    cur = "";
                } else {
                    cur += ch;
                }
            }
            if (cur.trim()) rawParamParts.push(cur.trim());

            const params = [];
            for (const part of rawParamParts) {
                if (!part || part === "&self" || part === "&mut self" || part === "self") continue;
                const colonIdx = part.indexOf(":");
                if (colonIdx === -1) continue;
                const pName = part.slice(0, colonIdx).trim().replace(/^r#/, "");
                const pRustType = part.slice(colonIdx + 1).trim();
                if (pRustType.includes("Interpreter")) continue;

                const isOptional = pRustType.startsWith("Option<") && pRustType.endsWith(">");
                const docParamName = pName === "fn_val" && docMeta.paramTypes["fn"] ? "fn" : pName;
                const mappedType =
                    docMeta.paramTypes[docParamName] ||
                    docMeta.paramTypes[pName] ||
                    mapRustTypeToEldritch(pRustType);

                const paramEntry = {
                    name: docParamName,
                    type: mappedType,
                };
                if (isOptional) {
                    paramEntry.defaultValue = "None";
                }
                params.push(paramEntry);
            }

            // Parse Rust return type Result<Inner, String>
            let innerReturnRust = rawReturnStr;
            const resMatch = rawReturnStr.match(/^Result<([\s\S]+),\s*String>$/);
            if (resMatch) {
                innerReturnRust = resMatch[1].trim();
            }
            const returnType = docMeta.returnType || mapRustTypeToEldritch(innerReturnRust);

            const mdEntry = mdDocs[qualifiedName];
            const signature = formatSignature(qualifiedName, params, returnType);
            const description =
                (mdEntry && mdEntry.description) ||
                rustDoc ||
                `Eldritch \`${qualifiedName}\` function.`;

            let isDeprecated = false;
            let replacement;
            if (depMatch) {
                isDeprecated = true;
                const depMsg = depMatch[1].replace(/\\"/g, '"');
                const useMatch = depMsg.match(/Please use ([^.]+(?:\.[^.\s]+)+(?:\([^)]*\))?(?:\s*&\s*[^.\s]+(?:\.[^.\s]+)+)?)\s+instead/i);
                if (useMatch) {
                    replacement = useMatch[1].split(/\s*&\s*/)[0].trim();
                } else if (changelog.renames[qualifiedName]) {
                    replacement = changelog.renames[qualifiedName].split(/\s*&\s*/)[0].trim();
                }
            }

            totalMethods++;
            if (isDeprecated) {
                deprecatedMethods++;
            }

            const fnEntry = {
                signature,
                description,
                kind: "function",
                module: modName,
                params,
                returnType,
                deprecated: isDeprecated,
            };
            if (replacement) {
                fnEntry.replacement = replacement;
            }
            entries[qualifiedName] = fnEntry;
        }

        const isModuleDeprecated = totalMethods > 0 && deprecatedMethods === totalMethods;
        entries[modName] = {
            signature: "",
            description: modDescription,
            kind: "module",
            deprecated: isModuleDeprecated,
        };
    }

    // Add any historical renamed functions from AGENTS.md that were already removed from Rust traits
    for (const [oldName, newTarget] of Object.entries(changelog.renames)) {
        if (entries[oldName]) continue;
        const dotIdx = oldName.indexOf(".");
        const modName = dotIdx !== -1 ? oldName.slice(0, dotIdx) : "";
        const targetEntry = entries[newTarget];
        entries[oldName] = {
            signature: targetEntry
                ? targetEntry.signature.replace(newTarget, oldName)
                : `${oldName}() -> Any`,
            description: `Deprecated in Eldritch 0.4.1; please use \`${newTarget}\` instead.`,
            kind: "function",
            module: modName,
            params: targetEntry ? targetEntry.params : [],
            returnType: targetEntry ? targetEntry.returnType : "Any",
            deprecated: true,
            deprecatedOnly: true,
            replacement: newTarget,
        };
    }

    // Attach removed functions to their respective module entries
    for (const [removedFn, reason] of Object.entries(changelog.removed)) {
        const dotIdx = removedFn.indexOf(".");
        if (dotIdx === -1) continue;
        const modName = removedFn.slice(0, dotIdx);
        if (entries[modName]) {
            if (!entries[modName].removedFunctions) {
                entries[modName].removedFunctions = {};
            }
            entries[modName].removedFunctions[removedFn] = reason;
        }
    }

    return entries;
}

function parseRustBuiltins(mdDocs, keywordsData) {
    const entries = {};
    if (!fs.existsSync(builtinsDir)) {
        return entries;
    }

    // Read mod.rs to discover registered builtin names
    const modRsPath = path.join(builtinsDir, "mod.rs");
    const modSrc = fs.existsSync(modRsPath) ? fs.readFileSync(modRsPath, "utf8") : "";
    const registeredNames = new Set(["eval"]);
    const regRe = /\(\s*"([a-zA-Z0-9_]+)"\s*,/g;
    let m;
    while ((m = regRe.exec(modSrc)) !== null) {
        registeredNames.add(m[1]);
    }
    const stubRe = /"([a-zA-Z0-9_]+)"/g;
    const stubsIdx = modSrc.indexOf("pub fn get_stubs");
    if (stubsIdx !== -1) {
        const stubsSlice = modSrc.slice(stubsIdx);
        while ((m = stubRe.exec(stubsSlice)) !== null) {
            registeredNames.add(m[1]);
        }
    }

    // Parse each builtin .rs file for doc comments
    const fileDocs = {};
    for (const file of fs.readdirSync(builtinsDir).sort()) {
        if (!file.endsWith(".rs") || file === "mod.rs") continue;
        const filePath = path.join(builtinsDir, file);
        const src = fs.readFileSync(filePath, "utf8");
        const docLines = [];
        for (const line of src.split("\n")) {
            const trimmed = line.trim();
            if (trimmed.startsWith("///")) {
                docLines.push(trimmed.replace(/^\/\/\/\s?/, ""));
            } else if (trimmed.startsWith("pub fn ") || trimmed.startsWith("pub(crate) fn ")) {
                break;
            }
        }
        if (docLines.length === 0) continue;
        const firstLine = docLines[0];
        const sigMatch = firstLine.match(/^`([a-zA-Z0-9_]+)\(([^`]*)`(?:\s*or\s*`([^`]+)`)?:\s*(.*)$/);
        let fnName = file.replace(/\.rs$/, "");
        if (fnName === "builtins_fn") fnName = "builtins";
        if (fnName === "eval_builtin") fnName = "eval";
        if (fnName === "type_") fnName = "type";

        let rawSig = `${fnName}()`;
        let summary = firstLine;
        if (sigMatch) {
            fnName = sigMatch[1];
            rawSig = sigMatch[3] ? sigMatch[3] : `${sigMatch[1]}(${sigMatch[2]}`;
            summary = sigMatch[4].trim();
        }

        const fullDoc = [summary, ...docLines.slice(1)].join("\n").trim();
        const docMeta = parseDocMetadata(fullDoc);
        fileDocs[fnName] = {
            rawSig,
            description: fullDoc,
            docMeta,
        };
    }

    for (const name of Array.from(registeredNames).sort()) {
        const rustInfo = fileDocs[name];
        const mdInfo = mdDocs[name];
        const returnType =
            (rustInfo && rustInfo.docMeta.returnType) ||
            (name === "bool"
                ? "bool"
                : name === "int"
                  ? "int"
                  : name === "float"
                    ? "float"
                    : ["str", "chr", "hex", "repr", "type"].includes(name)
                      ? "str"
                      : name === "bytes"
                        ? "Bytes"
                        : [
                                "list",
                                "dir",
                                "libs",
                                "builtins",
                                "range",
                                "enumerate",
                                "filter",
                                "map",
                                "reversed",
                                "sorted",
                                "zip",
                            ].includes(name)
                          ? "list"
                          : name === "dict"
                            ? "dict"
                            : name === "set"
                              ? "set"
                              : name === "tuple"
                                ? "tuple"
                                : ["len", "ord"].includes(name)
                                  ? "int"
                                  : ["abs"].includes(name)
                                    ? "int | float"
                                    : ["all", "any"].includes(name)
                                      ? "bool"
                                      : ["print", "eprint", "pprint", "tprint", "assert", "assert_eq", "fail"].includes(
                                              name
                                          )
                                        ? "None"
                                        : "Any");

        let sigSource =
            (rustInfo && `${rustInfo.rawSig} -> ${returnType}`) ||
            (mdInfo && mdInfo.signature) ||
            `${name}() -> ${returnType}`;
        if (!sigSource.includes("->")) {
            sigSource = `${sigSource} -> ${returnType}`;
        }

        const parsedSig = parseSignatureString(name, sigSource, returnType);
        if (rustInfo && rustInfo.docMeta.paramTypes) {
            for (const p of parsedSig.params) {
                if (rustInfo.docMeta.paramTypes[p.name] && p.type === "Any") {
                    p.type = rustInfo.docMeta.paramTypes[p.name];
                }
            }
        }

        const signature = formatSignature(name, parsedSig.params, parsedSig.returnType);
        const description =
            (mdInfo && mdInfo.description) ||
            (rustInfo && rustInfo.description) ||
            `Built-in \`${name}\` function.`;

        const entry = {
            signature,
            description,
            kind: "builtin",
            params: parsedSig.params,
            returnType: parsedSig.returnType,
        };

        if (name === "builtins") {
            entry.keywords = keywordsData.keywords;
            entry.booleans = keywordsData.booleans;
        }

        entries[name] = entry;
    }

    return entries;
}

function parseRustValueMethods() {
    const entries = {};

    // 1. Parse docs/methods.rs (`//: type.method` blocks)
    const methodDocs = {};
    if (fs.existsSync(methodsDocPath)) {
        const src = fs.readFileSync(methodsDocPath, "utf8");
        const lines = src.split("\n");
        let currentKey = null;
        let currentLines = [];

        const flush = () => {
            if (!currentKey) return;
            const text = currentLines.join("\n").trim();
            const meta = parseDocMetadata(text);
            methodDocs[currentKey] = {
                description: text,
                paramTypes: meta.paramTypes,
                returnType: meta.returnType || "Any",
            };
            currentLines = [];
        };

        for (const rawLine of lines) {
            const trimmed = rawLine.trim();
            if (trimmed.startsWith("//:")) {
                const content = trimmed.replace(/^\/\/:\s?/, "");
                if (/^[a-z]+\.[a-z0-9_]+$/.test(content)) {
                    flush();
                    currentKey = content;
                    continue;
                }
                if (currentKey) {
                    currentLines.push(content);
                }
            } else if (!trimmed) {
                flush();
                currentKey = null;
            }
        }
        flush();
    }

    // 2. Also read interpreter/methods/mod.rs (`get_native_methods`) to ensure all runtime methods are included
    if (fs.existsSync(methodsModPath)) {
        const modSrc = fs.readFileSync(methodsModPath, "utf8");
        const typeMap = [
            { pattern: /Value::String\(_\)\s*=>\s*vec!\[([\s\S]*?)\]/, typeName: "str" },
            { pattern: /Value::Bytes\(_\)\s*=>\s*vec!\[([\s\S]*?)\]/, typeName: "bytes" },
            { pattern: /Value::List\(_\)\s*=>\s*vec!\[([\s\S]*?)\]/, typeName: "list" },
            { pattern: /Value::Dictionary\(_\)\s*=>\s*vec!\[([\s\S]*?)\]/, typeName: "dict" },
            { pattern: /Value::Set\(_\)\s*=>\s*vec!\[([\s\S]*?)\]/, typeName: "set" },
        ];
        for (const { pattern, typeName } of typeMap) {
            const match = modSrc.match(pattern);
            if (!match) continue;
            const strRe = /"([a-zA-Z0-9_]+)"/g;
            let sm;
            while ((sm = strRe.exec(match[1])) !== null) {
                const key = `${typeName}.${sm[1]}`;
                if (!methodDocs[key]) {
                    methodDocs[key] = {
                        description: `\`${key}\` method on \`${typeName}\`.`,
                        paramTypes: {},
                        returnType: "Any",
                    };
                }
            }
        }
    }

    for (const [qualifiedName, info] of Object.entries(methodDocs)) {
        const dotIdx = qualifiedName.indexOf(".");
        const typeName = qualifiedName.slice(0, dotIdx);
        const params = [];
        for (const [pName, pType] of Object.entries(info.paramTypes)) {
            const isOptional =
                pType.startsWith("Optional<") || pType.startsWith("Option<");
            const paramEntry = { name: pName, type: pType };
            if (isOptional) {
                paramEntry.defaultValue = "None";
            }
            params.push(paramEntry);
        }
        const returnType = info.returnType || "Any";
        const signature = formatSignature(qualifiedName, params, returnType);

        entries[qualifiedName] = {
            signature,
            description: info.description,
            kind: "method",
            module: typeName,
            params,
            returnType,
        };
    }

    return entries;
}

function generateDocs() {
    console.log("Generating Eldritch documentation and LSP language metadata from Rust sources...");

    const mdDocs = parseUserGuideMarkdown();
    const keywordsData = parseKeywordsFromTokenRs();
    const changelog = parseAgentsChangelog();

    const stdlibEntries = parseRustStdlib(mdDocs, changelog);
    const builtinEntries = parseRustBuiltins(mdDocs, keywordsData);
    const methodEntries = parseRustValueMethods();

    const allDocs = {
        ...stdlibEntries,
        ...builtinEntries,
        ...methodEntries,
    };

    const outputDir = path.dirname(outputPath);
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }

    fs.writeFileSync(outputPath, JSON.stringify(allDocs, null, 2));
    console.log(
        `Generated documentation for ${Object.keys(allDocs).length} items at ${outputPath}`
    );
}

generateDocs();
