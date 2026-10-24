import { describe, it, expect, beforeAll } from "vitest";
import { render, screen } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import AgentBuilderLandingPage from "../AgentBuilderLandingPage";
import { GET_BUILD_PROFILES, GET_BUILDERS, GET_BUILD_TASKS } from "../queries";

beforeAll(() => {
    if (typeof globalThis.ResizeObserver === "undefined") {
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        } as unknown as typeof ResizeObserver;
    }
});

const emptyProfilesMock = {
    request: { query: GET_BUILD_PROFILES },
    result: { data: { buildProfiles: { edges: [] }, tomes: { edges: [] } } },
};
const emptyTasksMock = {
    request: { query: GET_BUILD_TASKS },
    result: { data: { buildTasks: { edges: [] } } },
};
const emptyBuildersMock = {
    request: { query: GET_BUILDERS },
    result: { data: { builders: { edges: [] } } },
};

function renderPage(mocks: ReadonlyArray<MockedResponse>, initialEntry = "/builder") {
    return render(
        <ChakraProvider>
            <MemoryRouter initialEntries={[initialEntry]}>
                <MockedProvider mocks={mocks} addTypename={false}>
                    <AgentBuilderLandingPage />
                </MockedProvider>
            </MemoryRouter>
        </ChakraProvider>
    );
}

describe("AgentBuilderLandingPage -- tab coordination", () => {
    it("defaults to the Agents tab and shows its empty state", async () => {
        renderPage([emptyProfilesMock, emptyTasksMock, emptyBuildersMock]);

        expect(await screen.findByText("No built agents yet")).toBeInTheDocument();
        expect(screen.getByRole("tab", { name: /Agents/ }).getAttribute("aria-selected")).toBe("true");
    });

    it("selects the tab named by the `tab` search param", async () => {
        renderPage([emptyProfilesMock, emptyTasksMock, emptyBuildersMock], "/builder?tab=builders");

        expect(await screen.findByText("No builders available")).toBeInTheDocument();
        expect(screen.getByRole("tab", { name: /Builders/ }).getAttribute("aria-selected")).toBe("true");
    });

    it("switches to the Profiles tab on click", async () => {
        renderPage([emptyProfilesMock, emptyTasksMock, emptyBuildersMock]);

        await screen.findByText("No built agents yet");
        screen.getByRole("tab", { name: /Profiles/ }).click();

        expect(await screen.findByText("No build profiles available")).toBeInTheDocument();
    });
});
