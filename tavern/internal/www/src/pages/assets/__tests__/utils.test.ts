import { describe, it, expect } from "vitest";
import { isAgentAsset, truncateAssetName } from "../utils";

describe("isAgentAsset", () => {
    it("identifies build/ paths as agent assets", () => {
        expect(isAgentAsset("build/linux/bin/imix-default-random")).toBe(true);
        expect(isAgentAsset("build/windows/exe/imix-profile-test")).toBe(true);
        expect(isAgentAsset("build/darwin/bin/imix-macos-test")).toBe(true);
    });

    it("identifies imix- substrings as agent assets", () => {
        expect(isAgentAsset("payloads/imix-test")).toBe(true);
        expect(isAgentAsset("imix-agent-binary")).toBe(true);
    });

    it("returns false for regular assets", () => {
        expect(isAgentAsset("tools/mimikatz.exe")).toBe(false);
        expect(isAgentAsset("scripts/enumeration.sh")).toBe(false);
        expect(isAgentAsset("documents/notes.txt")).toBe(false);
        expect(isAgentAsset(null)).toBe(false);
        expect(isAgentAsset("")).toBe(false);
    });
});

describe("truncateAssetName", () => {
    it("keeps short names intact", () => {
        expect(truncateAssetName("notes.txt")).toBe("notes.txt");
    });

    it("prioritizes filename for long path names", () => {
        const result = truncateAssetName("build/linux/bin/imix-default-profile-long-name-12345");
        expect(result).toContain("imix-default");
    });
});
