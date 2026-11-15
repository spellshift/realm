import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import AssetsTable, { buildStatusLabel } from "../AssetsTable";

const useQuery = vi.fn();
vi.mock("@apollo/client", async (importOriginal) => {
    const original = await importOriginal<typeof import("@apollo/client")>();
    return {
        ...original,
        useMutation: () => [vi.fn(), { loading: false }],
        useQuery: (...args: unknown[]) => useQuery(...args),
    };
});
vi.mock("../../../context/AuthorizationContext", () => ({
    useAuthorization: () => ({ data: { me: { isAdmin: true } } }),
}));

const asset = (id: string, name: string) => ({
    id, name, size: 10, hash: "abcdef1234567890", createdAt: "2026-01-01T00:00:00Z", lastModifiedAt: "2026-01-01T00:00:00Z",
    creator: null, tomes: { totalCount: 0 }, links: { totalCount: 0 },
});
const ASSETS = [
    asset("1", "build/imix-queued"),
    asset("2", "build/imix-building"),
    asset("3", "build/imix-failed"),
    asset("4", "build/imix-built"),
    asset("5", "build/imix-legacy"),
    asset("6", "plain-file.txt"),
];

// Render each asset's name cell directly; the virtualized table's own
// layout/fetching isn't what's under test.
vi.mock("../../../components/tavern-base-ui/virtualized-table/VirtualizedTable", () => ({
    VirtualizedTable: ({ columns }: { columns: Array<{ key: string; render: (a: unknown) => React.ReactNode }> }) => {
        const nameCol = columns.find((c) => c.key === "name")!;
        return <div>{ASSETS.map((a) => <div key={a.id} data-testid={`row-${a.id}`}>{nameCol.render(a)}</div>)}</div>;
    },
}));

const task = (id: string, assetId: string | null, overrides: Record<string, unknown> = {}) => ({
    node: {
        id, claimedAt: null, startedAt: null, finishedAt: null, exitCode: null, error: null,
        artifact: assetId ? { id: assetId, name: "x", size: 1, hash: "h" } : null,
        ...overrides,
    },
});

const renderTable = () => render(
    <ChakraProvider><MemoryRouter>
        <AssetsTable assetIds={ASSETS.map((a) => a.id)} onCreateLink={vi.fn()} onAssetUpdate={vi.fn()} />
    </MemoryRouter></ChakraProvider>,
);

describe("AssetsTable build status", () => {
    beforeEach(() => {
        useQuery.mockReturnValue({
            data: {
                buildTasks: {
                    edges: [
                        task("t1", "1"),
                        task("t2", "2", { claimedAt: "2026-01-01", startedAt: "2026-01-01" }),
                        task("t3", "3", { finishedAt: "2026-01-01", exitCode: 2, error: "boom" }),
                        task("t4", "4", { finishedAt: "2026-01-01", exitCode: 0 }),
                        task("t5", null),
                    ],
                },
            },
        });
    });

    it("renders the ant icon (not the bug icon) for agent rows only", () => {
        renderTable();
        expect(screen.getAllByTestId("agent-ant-icon")).toHaveLength(5);
        expect(screen.queryByTestId("agent-bug-icon")).toBeNull();
        expect(screen.getByTestId("row-6").querySelector('[data-testid="agent-ant-icon"]')).toBeNull();
    });

    it("shows a status marker only for unfinished or failed builds", () => {
        renderTable();
        const marker = (id: string) => screen.getByTestId(`row-${id}`).querySelector('[data-testid="agent-build-status"]');
        expect(marker("1")).toHaveAttribute("data-status", "Queued");
        expect(marker("2")).toHaveAttribute("data-status", "Building");
        expect(marker("3")).toHaveAttribute("data-status", "Failed");
        expect(marker("4")).toBeNull(); // built
        expect(marker("5")).toBeNull(); // legacy agent binary, no BuildTask
        expect(marker("6")).toBeNull(); // plain upload
        expect(screen.getAllByTestId("agent-build-status")).toHaveLength(3);
    });

    it("renders the copy icon outside the filename tooltip", () => {
        renderTable();
        const row = screen.getByTestId("row-4");
        expect(row.querySelectorAll('[data-testid="asset-copy-icon"]')).toHaveLength(1);
        const nameSpan = screen.getByText("build/imix-built");
        expect(nameSpan.contains(row.querySelector('[data-testid="asset-copy-icon"]'))).toBe(false);
    });
});

describe("buildStatusLabel", () => {
    it("maps statuses to tooltip text", () => {
        const base = { error: null, exitCode: null };
        expect(buildStatusLabel({ ...base, status: "Queued" })).toBe("Queued");
        expect(buildStatusLabel({ ...base, status: "Claimed" })).toBe("Queued");
        expect(buildStatusLabel({ ...base, status: "Building" })).toBe("Building");
        expect(buildStatusLabel({ ...base, status: "Awaiting artifact" })).toBe("Building");
        expect(buildStatusLabel({ status: "Failed", error: "boom", exitCode: 2 })).toBe("Build failed (exit code 2): boom");
    });
});
