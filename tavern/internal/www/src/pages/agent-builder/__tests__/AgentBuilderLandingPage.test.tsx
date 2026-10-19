import { describe, it, expect, beforeAll } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import AgentBuilderLandingPage from "../AgentBuilderLandingPage";
import { GET_BUILD_PROFILES, GET_BUILDERS, GET_BUILD_TASKS, REGISTER_BUILDER } from "../api";

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

const registerBuilderInput = {
    supportedTargets: ["PLATFORM_LINUX", "PLATFORM_MACOS", "PLATFORM_WINDOWS"],
    upstream: "http://127.0.0.1:8000",
    pollInterval: 5,
};

function renderPage(mocks: ReadonlyArray<MockedResponse>, initialEntry = "/scratch-agent-builder?tab=builders") {
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

describe("AgentBuilderLandingPage -- inlined enroll builder modal", () => {
    it("registers a builder and shows the one-time config without a separate modal component", async () => {
        const mocks = [
            emptyProfilesMock,
            emptyTasksMock,
            emptyBuildersMock,
            emptyBuildersMock, // tolerate useBuilders' poll re-fetch during the test
            {
                request: { query: REGISTER_BUILDER, variables: { input: registerBuilderInput } },
                result: {
                    data: {
                        registerBuilder: {
                            builder: { id: "b1", identifier: "builder-a1b2c3" },
                            config: "id: builder-a1b2c3\nupstream: http://127.0.0.1:8000\n",
                        },
                    },
                },
            },
            emptyBuildersMock,
        ];
        renderPage(mocks);

        const enrollButton = await screen.findByRole("button", { name: /enroll builder/i });
        fireEvent.click(enrollButton);

        const dialog = await screen.findByRole("dialog");
        fireEvent.click(within(dialog).getByRole("button", { name: "Enroll" }));

        await waitFor(() => expect(within(dialog).getByText(/mTLS credentials are only shown once/)).toBeInTheDocument());
        expect(within(dialog).getByText((_, element) => element?.tagName.toLowerCase() === "pre" && !!element.textContent?.includes("builder-a1b2c3"))).toBeInTheDocument();
    });

    it("surfaces a registerBuilder error without closing the enroll form", async () => {
        const mocks = [
            emptyProfilesMock,
            emptyTasksMock,
            emptyBuildersMock,
            emptyBuildersMock,
            {
                request: { query: REGISTER_BUILDER, variables: { input: registerBuilderInput } },
                error: new Error("permission denied"),
            },
        ];
        renderPage(mocks);

        const enrollButton = await screen.findByRole("button", { name: /enroll builder/i });
        fireEvent.click(enrollButton);

        const dialog = await screen.findByRole("dialog");
        fireEvent.click(within(dialog).getByRole("button", { name: "Enroll" }));

        await waitFor(() => expect(within(dialog).getByText("Unable to enroll builder")).toBeInTheDocument());
        expect(within(dialog).getByRole("button", { name: "Enroll" })).toBeInTheDocument();
    });

    it("resets the enroll form state when Cancel is clicked", async () => {
        const mocks = [emptyProfilesMock, emptyTasksMock, emptyBuildersMock, emptyBuildersMock];
        renderPage(mocks);

        const enrollButton = await screen.findByRole("button", { name: /enroll builder/i });
        fireEvent.click(enrollButton);
        const dialog = await screen.findByRole("dialog");

        fireEvent.click(within(dialog).getByLabelText("Remove Linux"));
        fireEvent.click(within(dialog).getByLabelText("Remove macOS"));
        fireEvent.click(within(dialog).getByLabelText("Remove Windows"));
        await waitFor(() => expect(within(dialog).getByRole("button", { name: "Enroll" })).toBeDisabled());

        fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));

        // Cancel resets enrollTargets back to all platforms -- whether or
        // not the dialog has finished its (CSS-driven, untestable in
        // jsdom) close transition, the underlying form state should be
        // back to its defaults.
        await waitFor(() => expect(within(dialog).getByLabelText("Remove Linux")).toBeInTheDocument());
        expect(within(dialog).getByRole("button", { name: "Enroll" })).not.toBeDisabled();
    });
});
