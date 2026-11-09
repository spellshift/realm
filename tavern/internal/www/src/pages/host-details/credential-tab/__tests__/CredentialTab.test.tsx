import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { ApolloError } from "@apollo/client";
import React from "react";
import CredentialTab from "../CredentialTab";
import Credential from "../Credential";
import { groupCredentialsByPrincipalAndKind, useCredsTable } from "../useCredsTable";
import { SortsProvider } from "../../../../context/SortContext";

vi.mock("../useCredsTable", async () => {
    const actual = await vi.importActual<typeof import("../useCredsTable")>("../useCredsTable");
    return {
        ...actual,
        useCredsTable: vi.fn(),
    };
});

vi.mock("../CredentialTable", () => ({
    default: ({ data, showLastModified }: any) => (
        <div data-testid={showLastModified ? "credential-table-grouped" : "credential-table"}>
            Rows: {data.length}
        </div>
    ),
}));

function renderCredentialTab(route = "/hosts/1") {
    return render(
        <MemoryRouter initialEntries={[route]}>
            <SortsProvider>
                <Routes>
                    <Route path="/hosts/:hostId" element={<CredentialTab />} />
                    <Route path="/hosts" element={<CredentialTab />} />
                </Routes>
            </SortsProvider>
        </MemoryRouter>
    );
}

describe("CredentialTab", () => {
    const mockSetSearch = vi.fn();
    const mockSetGroupByPrincipal = vi.fn();

    const defaultHookReturn = {
        loading: false,
        creds: [
            {
                id: "1",
                createdAt: "2026-01-01T00:00:00Z",
                lastModifiedAt: "2026-01-01T00:00:00Z",
                principal: "root",
                kind: "PASSWORD",
                secret: "hunter2",
            },
        ],
        totalCount: 1,
        error: undefined,
        search: "",
        setSearch: mockSetSearch,
        groupByPrincipal: false,
        setGroupByPrincipal: mockSetGroupByPrincipal,
    };

    beforeEach(() => {
        sessionStorage.clear();
        vi.clearAllMocks();
        vi.mocked(useCredsTable).mockReturnValue(defaultHookReturn);
    });

    it("renders sticky header with Credentials title, count, group switch, sorting controls, and search without top label", () => {
        renderCredentialTab();

        expect(screen.getByText("Credentials")).toBeInTheDocument();
        expect(screen.getByText("(1)")).toBeInTheDocument();
        expect(screen.getByText("Group by Principal/Kind")).toBeInTheDocument();
        expect(screen.getByText("Sort (Principal)")).toBeInTheDocument();
        expect(screen.getByPlaceholderText("Search by Principal")).toBeInTheDocument();
        expect(screen.queryByText("Search by Principal", { selector: "label" })).not.toBeInTheDocument();
        expect(screen.getByTestId("credential-table")).toBeInTheDocument();
    });

    it("renders error state with 'Error loading data' and error message subheader", () => {
        vi.mocked(useCredsTable).mockReturnValue({
            ...defaultHookReturn,
            error: new ApolloError({ errorMessage: "Failed to fetch" }),
            totalCount: undefined,
        });

        renderCredentialTab();

        expect(screen.getByText("Error loading data")).toBeInTheDocument();
        expect(screen.getByText("Failed to fetch")).toBeInTheDocument();
        expect(screen.queryByTestId("credential-table")).not.toBeInTheDocument();
    });

    it("renders loading state with 'Loading data...'", () => {
        vi.mocked(useCredsTable).mockReturnValue({
            ...defaultHookReturn,
            loading: true,
            totalCount: undefined,
        });

        renderCredentialTab();

        expect(screen.getByText("Loading data...")).toBeInTheDocument();
        expect(screen.queryByTestId("credential-table")).not.toBeInTheDocument();
    });

    it("renders noMatches state with Clear filters button when search has no results", async () => {
        const user = userEvent.setup();
        vi.mocked(useCredsTable).mockReturnValue({
            ...defaultHookReturn,
            creds: [],
            totalCount: 0,
            search: "nonexistent",
        });

        renderCredentialTab();

        expect(screen.getByText("No data matching your filters")).toBeInTheDocument();
        const clearBtn = screen.getByRole("button", { name: "Clear filters" });
        expect(clearBtn).toBeInTheDocument();

        await user.click(clearBtn);
        expect(mockSetSearch).toHaveBeenCalledWith("");
    });

    it("renders noData state when there are no credentials and no search filter", () => {
        vi.mocked(useCredsTable).mockReturnValue({
            ...defaultHookReturn,
            creds: [],
            totalCount: 0,
            search: "",
        });

        renderCredentialTab();

        expect(screen.getByText("No credentials reported")).toBeInTheDocument();
        expect(screen.queryByTestId("credential-table")).not.toBeInTheDocument();
    });

    it("renders CredentialTableGrouped when groupByPrincipal is enabled", () => {
        vi.mocked(useCredsTable).mockReturnValue({
            ...defaultHookReturn,
            groupByPrincipal: true,
        });

        renderCredentialTab();

        expect(screen.getByTestId("credential-table-grouped")).toBeInTheDocument();
        expect(screen.queryByTestId("credential-table")).not.toBeInTheDocument();
    });
});

describe("Credential component", () => {
    it("masks secret with a capped length and toggles visibility with accessible aria-label", async () => {
        const user = userEvent.setup();
        const longSecret = "a".repeat(64);
        render(<Credential secret={longSecret} />);

        expect(screen.getByText("*".repeat(32))).toBeInTheDocument();
        const showButton = screen.getByRole("button", { name: "Show secret" });
        expect(showButton).toBeInTheDocument();

        await user.click(showButton);
        expect(screen.getByText(longSecret)).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Hide secret" })).toBeInTheDocument();
    });
});

describe("groupCredentialsByPrincipalAndKind", () => {
    it("combines earliest createdAt with most recent lastModifiedAt and secret per (principal, kind)", () => {
        const grouped = groupCredentialsByPrincipalAndKind([
            {
                id: "1",
                createdAt: "2026-01-01T00:00:00Z",
                lastModifiedAt: "2026-01-02T00:00:00Z",
                principal: "root",
                kind: "PASSWORD",
                secret: "old-pass",
            },
            {
                id: "2",
                createdAt: "2026-01-05T00:00:00Z",
                lastModifiedAt: "2026-01-10T00:00:00Z",
                principal: "root",
                kind: "PASSWORD",
                secret: "new-pass",
            },
        ]);

        expect(grouped).toHaveLength(1);
        expect(grouped[0].id).toBe("2");
        expect(grouped[0].createdAt).toBe("2026-01-01T00:00:00Z");
        expect(grouped[0].lastModifiedAt).toBe("2026-01-10T00:00:00Z");
        expect(grouped[0].secret).toBe("new-pass");
    });
});
