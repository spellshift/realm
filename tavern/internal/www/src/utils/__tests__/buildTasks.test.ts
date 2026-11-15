import { describe, expect, it } from "vitest";
import { BuildTask, taskStatus } from "../buildTasks";

const base: BuildTask = {
    id: "1", targetOs: "LINUX", targetFormat: "BIN", createdAt: "2026-01-01T00:00:00Z",
    claimedAt: null, startedAt: null, finishedAt: null, exitCode: null, error: null,
    profile: { id: "p", name: "p" }, profileAtCreation: null, artifact: null,
};
const art = { id: "a", name: "n", size: 1, hash: "h" };
const finished = "2026-01-01T00:00:00Z";
const finishedMs = new Date(finished).getTime();

describe("taskStatus", () => {
    it("derives in-progress states from timestamps", () => {
        expect(taskStatus(base)).toBe("Queued");
        expect(taskStatus({ ...base, claimedAt: finished })).toBe("Claimed");
        expect(taskStatus({ ...base, claimedAt: finished, startedAt: finished })).toBe("Building");
    });
    it("derives terminal states", () => {
        const done = { ...base, finishedAt: finished };
        expect(taskStatus({ ...done, exitCode: 1 })).toBe("Failed");
        expect(taskStatus({ ...done, exitCode: 0, artifact: art })).toBe("Built");
        expect(taskStatus({ ...done, exitCode: 0, error: "artifact extraction failed" }, finishedMs + 1000)).toBe("Failed");
        expect(taskStatus({ ...done, exitCode: 0 }, finishedMs + 1000)).toBe("Awaiting artifact");
        expect(taskStatus({ ...done, exitCode: 0 }, finishedMs + 10 * 60 * 1000)).toBe("Failed");
    });
});
