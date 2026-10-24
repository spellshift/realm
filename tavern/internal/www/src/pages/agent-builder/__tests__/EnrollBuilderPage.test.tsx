import { describe, it, expect } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import EnrollBuilderPage from "../EnrollBuilderPage";
import { REGISTER_BUILDER } from "../queries";

const registerBuilderInput = {
    supportedTargets: ["PLATFORM_LINUX", "PLATFORM_MACOS", "PLATFORM_WINDOWS"],
    upstream: "http://127.0.0.1:8000",
    pollInterval: 5,
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
            request: { query: REGISTER_BUILDER, variables: { input: registerBuilderInput } },
            result: { data: { registerBuilder: {
                builder: { id: "b1", identifier: "builder-a1b2c3" },
                config: "id: builder-a1b2c3\nupstream: http://127.0.0.1:8000\n",
            } } },
        }]);

        fireEvent.change(screen.getByLabelText("Executor"), { target: { value: "local" } });
        fireEvent.click(screen.getByRole("button", { name: "Enroll" }));
        await waitFor(() => expect(screen.getByText(/mTLS credentials are shown only once/)).toBeInTheDocument());
        expect(screen.getByText("builder-builder-a1b2c3.yaml")).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "Next" }));
        expect(screen.getByText((_, element) => element?.tagName.toLowerCase() === "pre" && !!element.textContent?.includes("--executor local"))).toBeInTheDocument();
    });

    it("keeps the form visible after a registration error", async () => {
        renderPage([{
            request: { query: REGISTER_BUILDER, variables: { input: registerBuilderInput } },
            error: new Error("permission denied"),
        }]);
        fireEvent.click(screen.getByRole("button", { name: "Enroll" }));
        await waitFor(() => expect(screen.getByText("Unable to enroll builder")).toBeInTheDocument());
        expect(screen.getByRole("button", { name: "Enroll" })).toBeInTheDocument();
    });
});
