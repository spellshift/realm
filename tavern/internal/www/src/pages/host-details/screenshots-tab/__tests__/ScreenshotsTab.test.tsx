import { InMemoryCache } from "@apollo/client";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import { relayStylePagination } from "@apollo/client/utilities";
import { ChakraProvider } from "@chakra-ui/react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import "@testing-library/jest-dom/vitest";
import { SortsProvider } from "../../../../context/SortContext";
import { ScreenshotsTab } from "..";
import { GET_HOST_SCREENSHOTS } from "../queries";

// jsdom has no layout, so simulate a fixed-size viewport window instead of a
// measured one. Importantly this must only render a *window* of items (not
// every item), otherwise VirtualizedCardList's own "near the bottom" infinite
// scroll check (triggers once the last rendered item is within 3 of the end)
// fires on mount for any small list and masks whether a real user action
// caused the fetch. The virtualizer must be referentially stable across
// renders, like the real hook's.
const VISIBLE_WINDOW = 3;
vi.mock("@tanstack/react-virtual", async () => {
    const { useRef } = await import("react");
    return {
        useVirtualizer: ({ count }: { count: number }) => {
            const virtualizer = useRef<any>({});
            const visible = Math.min(count, VISIBLE_WINDOW);
            Object.assign(virtualizer.current, {
                getVirtualItems: () => Array.from({ length: visible }, (_, index) => ({ index, start: index * 61, end: (index + 1) * 61 })),
                getTotalSize: () => count * 61,
                measureElement: () => undefined,
                range: { startIndex: 0, endIndex: visible - 1 },
            });
            return virtualizer.current;
        },
    };
});

const screenshot = (id: string, name: string, size: number, overrides = {}) => ({
    id, name, size,
    createdAt: "2026-11-07T14:03:00Z",
    lastModifiedAt: "2026-11-07T14:03:00Z",
    task: null,
    shellTask: null,
    ...overrides,
});

const first = screenshot("10", "first.png", 1024, { task: { id: "20", quest: { name: "Desktop survey" } } });
const second = screenshot("11", "second.png", 2048, { shellTask: { id: "21", input: "screenshot" } });
const third = screenshot("12", "third.png", 4096);

const page = (nodes: ReturnType<typeof screenshot>[], totalCount: number, hasNextPage: boolean, endCursor: string) => ({
    data: {
        hosts: { edges: [{ node: { __typename: "Host", id: "1", screenshots: {
            totalCount,
            pageInfo: { endCursor, hasNextPage },
            edges: nodes.map(node => ({ node })),
        } } }] },
    },
});

const request = (variables: Record<string, unknown> = {}) => ({
    query: GET_HOST_SCREENSHOTS,
    variables: { hostId: "1", first: 50, orderBy: [{ field: "CREATED_AT", direction: "DESC" }], ...variables },
});

// The first render in jsdom (Chakra, react-select, lazy module transforms) is slow on cold start.
const COLD_START = { timeout: 20000 };

const makeCache = () => new InMemoryCache({
    addTypename: false,
    typePolicies: {
        Query: { fields: { hosts: relayStylePagination(["where", "orderBy"]) } },
        Host: { fields: { screenshots: relayStylePagination(["orderBy"]) } },
    },
});

const renderTab = (mocks: MockedResponse[]) => render(
    <ChakraProvider>
        <MockedProvider mocks={mocks} addTypename={false} cache={makeCache()}>
            <MemoryRouter initialEntries={["/hosts/1?tab=screenshots"]}>
                <SortsProvider>
                    <Routes><Route path="/hosts/:hostId" element={<ScreenshotsTab />} /></Routes>
                </SortsProvider>
            </MemoryRouter>
        </MockedProvider>
    </ChakraProvider>
);

describe("ScreenshotsTab", () => {
    beforeEach(() => {
        sessionStorage.clear();
    });

    it("reviews host screenshots and selects a preview", async () => {
        const user = userEvent.setup();
        const sorted = vi.fn(() => page([second, first], 2, false, "cursor"));
        renderTab([
            { request: request(), result: page([first, second], 2, false, "cursor") },
            { request: request({ orderBy: [{ field: "SIZE", direction: "DESC" }] }), result: sorted },
        ]);

        expect(await screen.findByRole("button", { name: "Preview first.png" }, COLD_START)).toBeInTheDocument();
        expect(screen.getByText("Desktop survey")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Download all" })).toBeEnabled();
        expect(screen.getAllByRole("link", { name: "Download first.png" })).toHaveLength(2);

        await user.click(screen.getByRole("button", { name: "Preview second.png" }));
        expect(screen.getByAltText("second.png")).toBeInTheDocument();
        expect(screen.getByText("Shell task")).toBeInTheDocument();

        // Open the shared sort popover (reused from the files/tasks tabs) and
        // switch the field to Size, keeping the default Descending direction.
        await user.click(screen.getByRole("button", { name: "Sort (Created At)" }));
        await user.click(screen.getByText("Created At", { selector: "div" }));
        const sizeOption = await screen.findByText("Size", { selector: "div[id^=\"react-select\"]" });
        await user.click(sizeOption);
        await waitFor(() => expect(sorted).toHaveBeenCalled());

        // The selection is persisted via the shared SortContext, not local
        // component state.
        const stored = JSON.parse(sessionStorage.getItem("realm-sorting-v1.1") || "{}");
        expect(stored.Screenshots).toEqual({ field: "SIZE", direction: "DESC" });
    }, 30000);

    it("loads the next page and advances to its first screenshot only once Next reaches the last loaded item", async () => {
        const user = userEvent.setup();

        // A page large enough that VirtualizedCardList's windowed render (only
        // VISIBLE_WINDOW items) never crosses its own "near the bottom"
        // threshold (within 3 of the end). This isolates the assertion below
        // to the "Next screenshot" button's own pagination logic rather than
        // the sidebar's independent infinite-scroll auto-load.
        const firstPage = Array.from({ length: 6 }, (_, i) => screenshot(String(i), `s${i}.png`, 100));
        const nextPage = vi.fn(() => page([third], 7, false, "cursor-2"));
        renderTab([
            { request: request(), result: page(firstPage, 7, true, "cursor-1") },
            { request: request({ after: "cursor-1" }), result: nextPage },
        ]);

        await user.click(await screen.findByRole("button", { name: "Preview s0.png" }, COLD_START));
        expect(screen.getByText("1 of 7")).toBeInTheDocument();

        // nothing should have fetched page two yet merely from mounting/selecting
        expect(nextPage).not.toHaveBeenCalled();

        // walk from the first to the last *loaded* screenshot (index 5 of 6 loaded)
        for (let i = 0; i < 5; i++) {
            await user.click(screen.getByRole("button", { name: "Next screenshot" }));
        }
        expect(screen.getByText("6 of 7")).toBeInTheDocument();
        expect(nextPage).not.toHaveBeenCalled();

        // this click is the one that must cross the page boundary
        await user.click(screen.getByRole("button", { name: "Next screenshot" }));

        await waitFor(() => expect(screen.getByText("7 of 7")).toBeInTheDocument());
        expect(nextPage).toHaveBeenCalledTimes(1);
        expect(screen.getByAltText("third.png")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Next screenshot" })).toBeDisabled();
    }, 30000);

    it("renders the toolbar with no search input and Download all to the left of the sort control", async () => {
        renderTab([
            { request: request(), result: page([first, second], 2, false, "cursor") },
        ]);

        const downloadAllButton = await screen.findByRole("button", { name: "Download all" }, COLD_START);
        const sortButton = screen.getByRole("button", { name: "Sort (Created At)" });

        expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
        expect(screen.queryByPlaceholderText(/search/i)).not.toBeInTheDocument();

        // Download all is the left-most action; sort is right-most.
        expect(downloadAllButton.compareDocumentPosition(sortButton) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

        // The screenshot-count/sort summary line above the thumbnail list is gone.
        expect(screen.queryByText(/sorted by/i)).not.toBeInTheDocument();
    }, 30000);

    it("loads a previously persisted sort from the shared SortContext on mount", async () => {
        sessionStorage.setItem("realm-sorting-v1.1", JSON.stringify({
            Hosts: { field: "CREATED_AT", direction: "DESC" },
            Quests: { field: "CREATED_AT", direction: "DESC" },
            Tasks: { field: "LAST_MODIFIED_AT", direction: "DESC" },
            Assets: { field: "CREATED_AT", direction: "DESC" },
            Processes: { field: "PROCESS_START_TIME", direction: "ASC" },
            Files: { field: "NAME", direction: "ASC" },
            Credentials: { field: "PRINCIPAL", direction: "ASC" },
            Screenshots: { field: "NAME", direction: "ASC" },
        }));

        renderTab([
            { request: request({ orderBy: [{ field: "NAME", direction: "ASC" }] }), result: page([first, second], 2, false, "cursor") },
        ]);

        expect(await screen.findByRole("button", { name: "Sort (Name)" }, COLD_START)).toBeInTheDocument();
    }, 30000);
});
