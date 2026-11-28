import { render, screen, within } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { describe, expect, it, vi } from "vitest";
import { Builder } from "../types";
import BuildersTab from "./BuildersTab";

const builder = (
    identifier: string,
    lastSeenAtRaw: string | null,
    lastSeenAt: string | null,
    hostname: string | null = `${identifier}.local`,
): Builder => ({
    id: identifier,
    identifier,
    hostname,
    supportedTargets: [],
    upstream: "http://127.0.0.1:8000",
    pollInterval: 5,
    lastSeenAt,
    lastSeenAtRaw,
    online: false,
});

describe("BuildersTab", () => {
    it("sorts by the actual last seen timestamp, with never seen builders last", () => {
        render(<BuildersTab
            builders={[
                builder("older-builder", "2026-11-25T12:00:00Z", "3 days ago"),
                builder("never-seen-builder", null, null),
                builder("newer-builder", "2026-11-28T11:00:00Z", "1 hour ago"),
            ]}
            loading={false}
            onEnroll={vi.fn()}
        />);

        expect(screen.getByRole("columnheader", { name: "Hostname" })).toBeInTheDocument();
        const rows = screen.getAllByRole("row").slice(1);
        expect(rows.map((row) => within(row).getByText(/\.local$/).textContent)).toEqual([
            "newer-builder.local", "older-builder.local", "never-seen-builder.local",
        ]);
    });

    it("sorts RFC3339Nano fractional seconds and timezone offsets chronologically", () => {
        render(<BuildersTab
            builders={[
                builder("whole-second", "2026-11-28T11:00:00Z", "just now"),
                builder("ms-150", "2026-11-28T11:00:00.15Z", "just now"),
                builder("ms-200", "2026-11-28T11:00:00.2Z", "just now"),
                builder("offset-newest", "2026-11-28T06:00:01-05:00", "just now"),
            ]}
            loading={false}
            onEnroll={vi.fn()}
        />);

        const rows = screen.getAllByRole("row").slice(1);
        expect(rows.map((row) => within(row).getByText(/\.local$/).textContent)).toEqual([
            "offset-newest.local", "ms-200.local", "ms-150.local", "whole-second.local",
        ]);
    });

    it("shows a fallback for null or blank hostnames while preserving identifier visibility", () => {
        render(<BuildersTab
            builders={[
                builder("pending-builder-1", null, null, null),
                builder("pending-builder-2", null, null, "   "),
            ]}
            loading={false}
            onEnroll={vi.fn()}
        />);

        expect(screen.getAllByText("Unknown host")).toHaveLength(2);
        expect(screen.getByText("pending-builder-1")).toBeInTheDocument();
        expect(screen.getByText("pending-builder-2")).toBeInTheDocument();
    });
});
