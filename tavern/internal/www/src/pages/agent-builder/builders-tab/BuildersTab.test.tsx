import { render, screen, within } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { describe, expect, it, vi } from "vitest";
import { Builder } from "../types";
import BuildersTab from "./BuildersTab";

const builder = (identifier: string, lastSeenAtRaw: string | null, lastSeenAt: string | null): Builder => ({
    id: identifier,
    identifier,
    hostname: identifier,
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
        expect(rows.map((row) => within(row).getByText(/-builder/).textContent)).toEqual([
            "newer-builder", "older-builder", "never-seen-builder",
        ]);
    });

    it("shows a fallback until a builder reports its hostname", () => {
        render(<BuildersTab
            builders={[{ ...builder("pending-builder", null, null), hostname: null }]}
            loading={false}
            onEnroll={vi.fn()}
        />);

        expect(screen.getByText("Unknown host")).toBeInTheDocument();
    });
});
