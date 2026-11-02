import * as assert from "assert";
import { spawn, ChildProcessWithoutNullStreams } from "child_process";
import * as fs from "fs";
import * as path from "path";

class JsonRpcClient {
    private proc: ChildProcessWithoutNullStreams;
    private buffer = Buffer.alloc(0);
    private nextId = 1;
    private pending = new Map<number, { resolve: (val: any) => void; reject: (err: any) => void }>();
    private notifications: Array<{ method: string; params: any }> = [];
    private notificationListeners: Array<(msg: { method: string; params: any }) => void> = [];

    constructor(command: string, args: string[]) {
        this.proc = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
        this.proc.stdout.on("data", (chunk: Buffer) => this.onData(chunk));
    }

    private onData(chunk: Buffer): void {
        this.buffer = Buffer.concat([this.buffer, chunk]);
        while (true) {
            const headerEnd = this.buffer.indexOf("\r\n\r\n");
            if (headerEnd === -1) break;
            const header = this.buffer.slice(0, headerEnd).toString("ascii");
            const match = header.match(/Content-Length:\s*(\d+)/i);
            if (!match) {
                this.buffer = this.buffer.slice(headerEnd + 4);
                continue;
            }
            const len = parseInt(match[1], 10);
            const total = headerEnd + 4 + len;
            if (this.buffer.length < total) break;
            const body = this.buffer.slice(headerEnd + 4, total).toString("utf8");
            this.buffer = this.buffer.slice(total);
            const msg = JSON.parse(body);
            if (msg.id !== undefined && this.pending.has(msg.id)) {
                const handler = this.pending.get(msg.id)!;
                this.pending.delete(msg.id);
                if (msg.error) handler.reject(msg.error);
                else handler.resolve(msg.result);
            } else if (msg.method) {
                const notif = { method: msg.method, params: msg.params };
                this.notifications.push(notif);
                for (const l of this.notificationListeners) l(notif);
            }
        }
    }

    public request(method: string, params: any): Promise<any> {
        const id = this.nextId++;
        const payload = JSON.stringify({ jsonrpc: "2.0", id, method, params });
        const len = Buffer.byteLength(payload, "utf8");
        this.proc.stdin.write(`Content-Length: ${len}\r\n\r\n${payload}`);
        return new Promise((resolve, reject) => {
            this.pending.set(id, { resolve, reject });
        });
    }

    public notify(method: string, params: any): void {
        const payload = JSON.stringify({ jsonrpc: "2.0", method, params });
        const len = Buffer.byteLength(payload, "utf8");
        this.proc.stdin.write(`Content-Length: ${len}\r\n\r\n${payload}`);
    }

    public waitForNotification(method: string, timeoutMs = 5000): Promise<any> {
        const existing = this.notifications.find((n) => n.method === method);
        if (existing) {
            this.notifications = this.notifications.filter((n) => n !== existing);
            return Promise.resolve(existing.params);
        }
        return new Promise((resolve, reject) => {
            const timer = setTimeout(() => {
                reject(new Error(`Timed out waiting for notification ${method}`));
            }, timeoutMs);
            const listener = (notif: { method: string; params: any }) => {
                if (notif.method === method) {
                    clearTimeout(timer);
                    this.notificationListeners = this.notificationListeners.filter((l) => l !== listener);
                    resolve(notif.params);
                }
            };
            this.notificationListeners.push(listener);
        });
    }

    public async close(): Promise<void> {
        try {
            await this.request("shutdown", null);
            this.notify("exit", null);
        } catch {
            // ignore
        }
        this.proc.kill();
    }
}

async function testLspServer(label: string, command: string, args: string[], fullNativeChecks: boolean): Promise<void> {
    console.log(`[verify] Testing ${label}: ${command} ${args.join(" ")}`);
    const client = new JsonRpcClient(command, args);

    try {
        const initRes = await client.request("initialize", {
            processId: process.pid,
            rootUri: null,
            capabilities: {},
        });
        assert.ok(initRes.capabilities.completionProvider, `${label}: completionProvider missing`);
        assert.ok(initRes.capabilities.hoverProvider, `${label}: hoverProvider missing`);
        client.notify("initialized", {});

        const testUri = "file:///tmp/test_eldritch_verify.eldritch";
        const code = [
            "def greet(target: str) -> str:",
            "    \"\"\"Greet target host.\"\"\"",
            "    return f\"hello {target}\"",
            "",
            "msg = greet(\"box\")",
            "file.non_existent_fn()",
            "sys.get_os()",
            "file.",
            "sys.exec(\"ls\", ",
        ].join("\n");

        const diagPromise = client.waitForNotification("textDocument/publishDiagnostics");
        client.notify("textDocument/didOpen", {
            textDocument: {
                uri: testUri,
                languageId: "eldritch",
                version: 1,
                text: code,
            },
        });

        const diagParams = await diagPromise;
        assert.strictEqual(diagParams.uri, testUri);
        const codes = diagParams.diagnostics.map((d: any) => d.code);
        assert.ok(codes.includes("unknown-module-function"), `${label}: expected unknown-module-function diagnostic, got ${JSON.stringify(codes)}`);
        assert.ok(codes.includes("deprecated-function"), `${label}: expected deprecated-function diagnostic, got ${JSON.stringify(codes)}`);

        // Test completions after `file.` on line 7
        const compRes = await client.request("textDocument/completion", {
            textDocument: { uri: testUri },
            position: { line: 7, character: 5 },
        });
        const compItems = Array.isArray(compRes) ? compRes : compRes.items;
        const compLabels = compItems.map((c: any) => c.label);
        assert.ok(compLabels.includes("read"), `${label}: expected 'read' in file.* completions`);
        assert.ok(compLabels.includes("write"), `${label}: expected 'write' in file.* completions`);
        assert.ok(compLabels.includes("list"), `${label}: expected 'list' in file.* completions`);

        // Test top-level completions
        const topCompRes = await client.request("textDocument/completion", {
            textDocument: { uri: testUri },
            position: { line: 4, character: 0 },
        });
        const topItems = Array.isArray(topCompRes) ? topCompRes : topCompRes.items;
        const topLabels = topItems.map((c: any) => c.label);
        assert.ok(topLabels.includes("sys"), `${label}: expected 'sys' module in top-level completions`);
        assert.ok(topLabels.includes("pprint"), `${label}: expected 'pprint' builtin in top-level completions`);

        // Test hover on `sys`
        const hoverRes = await client.request("textDocument/hover", {
            textDocument: { uri: testUri },
            position: { line: 6, character: 1 },
        });
        assert.ok(hoverRes && hoverRes.contents, `${label}: expected hover contents for 'sys'`);

        if (fullNativeChecks) {
            // Test signatureHelp on `sys.exec("ls", `
            const sigRes = await client.request("textDocument/signatureHelp", {
                textDocument: { uri: testUri },
                position: { line: 8, character: 15 },
            });
            assert.ok(sigRes && sigRes.signatures && sigRes.signatures.length > 0, `${label}: expected signatureHelp`);
            assert.strictEqual(sigRes.activeParameter, 1, `${label}: expected activeParameter === 1`);

            // Test documentSymbol
            const symRes = await client.request("textDocument/documentSymbol", {
                textDocument: { uri: testUri },
            });
            assert.ok(Array.isArray(symRes) && symRes.some((s: any) => s.name === "greet"), `${label}: expected 'greet' in documentSymbol`);

            // Test go-to-definition on `greet("box")`
            const defRes = await client.request("textDocument/definition", {
                textDocument: { uri: testUri },
                position: { line: 4, character: 8 },
            });
            assert.ok(defRes && defRes.range.start.line === 0, `${label}: expected definition of greet on line 0`);
        }

        console.log(`[verify] ${label} passed all checks!`);
    } finally {
        await client.close();
    }
}

async function main(): Promise<void> {
    const vscodeRoot = path.resolve(__dirname, "../../..");

    // 1. Validate JSON assets
    const pkgJson = JSON.parse(fs.readFileSync(path.join(vscodeRoot, "package.json"), "utf8"));
    assert.strictEqual(pkgJson.name, "eldritch");
    assert.ok(pkgJson.contributes.languages[0].extensions.includes(".eldritch"));
    assert.ok(pkgJson.contributes.languages[0].extensions.includes(".tome"));

    const tmGrammar = JSON.parse(fs.readFileSync(path.join(vscodeRoot, "syntaxes/eldritch.tmLanguage.json"), "utf8"));
    assert.strictEqual(tmGrammar.scopeName, "source.eldritch");

    const langConfig = JSON.parse(fs.readFileSync(path.join(vscodeRoot, "syntaxes/eldritch.configuration.json"), "utf8"));
    assert.strictEqual(langConfig.comments.lineComment, "#");

    const snippets = JSON.parse(fs.readFileSync(path.join(vscodeRoot, "snippets/eldritch.snippets.json"), "utf8"));
    assert.ok(snippets["Function Definition"]);

    // 2. Test bundled native Rust eldritch-lsp binary
    const nativeBin = path.join(vscodeRoot, "bin/eldritch-lsp");
    assert.ok(fs.existsSync(nativeBin), `Bundled native binary missing at ${nativeBin}`);
    await testLspServer("Native Rust eldritch-lsp", nativeBin, ["--lsp"], true);

    // 3. Test bundled cross-platform Node fallback server
    const fallbackScript = path.join(vscodeRoot, "client/out/server.js");
    assert.ok(fs.existsSync(fallbackScript), `Bundled fallback server missing at ${fallbackScript}`);
    await testLspServer("Node Fallback eldritch-lsp", process.execPath, [fallbackScript, "--stdio"], false);

    console.log("[verify] All extension and LSP verification checks succeeded!");
}

main().catch((err) => {
    console.error("[verify] FAILED:", err);
    process.exit(1);
});
