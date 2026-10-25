import { describe, it, expect } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import EnrollBuilderPage from "../EnrollBuilderPage";
import { GET_BUILDERS, REGISTER_BUILDER } from "../queries";

const registerBuilderInput = {
    supportedTargets: ["PLATFORM_LINUX", "PLATFORM_MACOS", "PLATFORM_WINDOWS"],
    upstream: "http://127.0.0.1:8000",
    pollInterval: 5,
    executor: "docker",
};

const offlineBuildersMock = {
    request: { query: GET_BUILDERS },
    result: { data: { builders: { edges: [
        { node: { id: "b1", identifier: "builder-a1b2c3", supportedTargets: ["PLATFORM_LINUX"], upstream: "http://127.0.0.1:8000", pollInterval: 5, lastSeenAt: null } },
        { node: { id: "b2", identifier: "another-builder", supportedTargets: ["PLATFORM_LINUX"], upstream: "http://127.0.0.1:8000", pollInterval: 5, lastSeenAt: new Date().toISOString() } },
    ] } } },
};

function renderPage(mocks: ReadonlyArray<MockedResponse>) {
    return render(
        <MemoryRouter>
            <ChakraProvider>
                <MockedProvider mocks={mocks} addTypename={false}>
                    <EnrollBuilderPage />
                </MockedProvider>
            </ChakraProvider>
        </MemoryRouter>
    );
}

describe("EnrollBuilderPage", () => {
    it("registers a builder and shows its config and selected executor command", async () => {
        renderPage([{
            request: { query: REGISTER_BUILDER, variables: { input: { ...registerBuilderInput, executor: "local" } } },
            result: { data: { registerBuilder: {
                builder: { id: "b1", identifier: "builder-a1b2c3" },
                config: "id: builder-a1b2c3\nupstream: http://127.0.0.1:8000\n",
            } } },
        }, offlineBuildersMock]);

        fireEvent.change(screen.getByLabelText("Executor"), { target: { value: "local" } });
        fireEvent.click(screen.getByRole("button", { name: "Configure" }));
        await waitFor(() => expect(screen.getByText(/mTLS credentials are shown only once/)).toBeInTheDocument());
        expect(screen.getByText("builder-builder-a1b2c3.yaml")).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "Next" }));
        expect(screen.getByText((_, element) => element?.tagName.toLowerCase() === "pre" && !!element.textContent?.includes("--config builder-builder-a1b2c3.yaml"))).toBeInTheDocument();
        expect(screen.queryByText("Wait for builder to register")).not.toBeInTheDocument();
        await waitFor(() => expect(screen.getByText(/Waiting for the builder to connect/)).toBeInTheDocument());
        expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
        fireEvent.click(screen.getByRole("button", { name: "Previous" }));
        expect(screen.getByText(/mTLS credentials are shown only once/)).toBeInTheDocument();
    });

    it("keeps the form visible after a registration error", async () => {
        renderPage([{
            request: { query: REGISTER_BUILDER, variables: { input: registerBuilderInput } },
            error: new Error("permission denied"),
        }]);
        fireEvent.click(screen.getByRole("button", { name: "Configure" }));
        await waitFor(() => expect(screen.getByText("Unable to enroll builder")).toBeInTheDocument());
        expect(screen.getByRole("button", { name: "Configure" })).toBeInTheDocument();
    });

    it("enables Next when the enrolled builder calls back", async () => {
        renderPage([{
            request: { query: REGISTER_BUILDER, variables: { input: registerBuilderInput } },
            result: { data: { registerBuilder: {
                builder: { id: "b1", identifier: "builder-a1b2c3" },
                config: "executor: docker\n",
            } } },
        }, {
            request: { query: GET_BUILDERS },
            result: { data: { builders: { edges: [{ node: {
                id: "b1", identifier: "builder-a1b2c3", supportedTargets: ["PLATFORM_LINUX"],
                upstream: "http://127.0.0.1:8000", pollInterval: 5, lastSeenAt: new Date().toISOString(),
            } }] } } },
        }]);

        fireEvent.click(screen.getByRole("button", { name: "Configure" }));
        await waitFor(() => expect(screen.getByText(/mTLS credentials are shown only once/)).toBeInTheDocument());
        fireEvent.click(screen.getByRole("button", { name: "Next" }));
        await waitFor(() => expect(screen.getByRole("button", { name: "Next" })).toBeEnabled());
    });
});
