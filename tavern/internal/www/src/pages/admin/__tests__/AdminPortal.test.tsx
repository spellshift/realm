import { describe, it, expect, beforeAll } from "vitest";
import { render, screen } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import { AdminPortal } from "../AdminPortal";
import { AuthorizationContext } from "../../../context/AuthorizationContext";
import { GET_USER_IDS_QUERY } from "../queries";
import { GET_BUILD_PROFILES, GET_BUILDERS } from "../../agent-builder/queries";

beforeAll(() => {
    if (typeof globalThis.ResizeObserver === "undefined") {
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        } as unknown as typeof ResizeObserver;
    }
});

const emptyUsersMock = {
    request: { query: GET_USER_IDS_QUERY, variables: { first: 14 } },
    result: {
        data: {
            users: {
                totalCount: 0,
                pageInfo: {
                    hasNextPage: false,
                    hasPreviousPage: false,
                    startCursor: null,
                    endCursor: null,
                },
                edges: [],
            },
        },
    },
};

const emptyProfilesMock = {
    request: { query: GET_BUILD_PROFILES },
    result: { data: { buildProfiles: { edges: [] }, tomes: { edges: [] } } },
};

const emptyBuildersMock = {
    request: { query: GET_BUILDERS },
    result: { data: { builders: { edges: [] } } },
};

const defaultMocks = [emptyUsersMock, emptyProfilesMock, emptyBuildersMock];

const adminAuthValue = {
    data: {
        me: {
            id: "1",
            name: "Admin User",
            isAdmin: true,
            isActivated: true,
            photoURL: "",
        } as any,
    },
    isLoading: false,
    error: undefined,
};

function renderAdminPage(mocks: ReadonlyArray<MockedResponse> = defaultMocks, initialEntry = "/admin") {
    return render(
        <ChakraProvider>
            <AuthorizationContext.Provider value={adminAuthValue}>
                <MemoryRouter initialEntries={[initialEntry]}>
                    <MockedProvider mocks={mocks} addTypename={false}>
                        <AdminPortal />
                    </MockedProvider>
                </MemoryRouter>
            </AuthorizationContext.Provider>
        </ChakraProvider>
    );
}

describe("AdminPortal -- tab coordination", () => {
    it("renders the 3 tabs (Users, Profiles, Builders) and does not render Agents tab", async () => {
        renderAdminPage();

        expect(screen.getByRole("tab", { name: /Users/ })).toBeInTheDocument();
        expect(screen.getByRole("tab", { name: /Profiles/ })).toBeInTheDocument();
        expect(screen.getByRole("tab", { name: /Builders/ })).toBeInTheDocument();
        expect(screen.queryByRole("tab", { name: /Agents/ })).not.toBeInTheDocument();
    });

    it("defaults to the Users tab", async () => {
        renderAdminPage();

        expect(screen.getByRole("tab", { name: /Users/ }).getAttribute("aria-selected")).toBe("true");
    });

    it("selects the profiles tab via search param", async () => {
        renderAdminPage(defaultMocks, "/admin?tab=profiles");

        expect(screen.getByRole("tab", { name: /Profiles/ }).getAttribute("aria-selected")).toBe("true");
        expect(await screen.findByText("No build profiles available")).toBeInTheDocument();
    });

    it("selects the builders tab via search param", async () => {
        renderAdminPage(defaultMocks, "/admin?tab=builders");

        expect(screen.getByRole("tab", { name: /Builders/ }).getAttribute("aria-selected")).toBe("true");
        expect(await screen.findByText("No builders available")).toBeInTheDocument();
    });

    it("switches to the Profiles tab on click", async () => {
        renderAdminPage();

        const profilesTab = screen.getByRole("tab", { name: /Profiles/ });
        profilesTab.click();

        expect(await screen.findByText("No build profiles available")).toBeInTheDocument();
        expect(profilesTab.getAttribute("aria-selected")).toBe("true");
    });
});
