import { describe, it, expect, vi, beforeAll } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import EnrollBuilderModal from "./EnrollBuilderModal";
import { REGISTER_BUILDER } from "../queries";

beforeAll(() => {
    if (typeof globalThis.ResizeObserver === "undefined") {
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        } as unknown as typeof ResizeObserver;
    }
});

const registerBuilderInput = {
    supportedTargets: ["PLATFORM_LINUX", "PLATFORM_MACOS", "PLATFORM_WINDOWS"],
    upstream: "http://127.0.0.1:8000",
    pollInterval: 5,
};

function renderModal(mocks: ReadonlyArray<MockedResponse>, { onClose = vi.fn(), onEnrolled = vi.fn() } = {}) {
    return {
        onClose,
        onEnrolled,
        ...render(
            <ChakraProvider>
                <MockedProvider mocks={mocks} addTypename={false}>
                    <EnrollBuilderModal isOpen onClose={onClose} onEnrolled={onEnrolled} />
                </MockedProvider>
            </ChakraProvider>
        ),
    };
}

describe("EnrollBuilderModal", () => {
    it("registers a builder and shows the one-time config", async () => {
        const mocks = [
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
        ];
        const onEnrolled = vi.fn();
        renderModal(mocks, { onEnrolled });

        const dialog = await screen.findByRole("dialog");
        fireEvent.click(within(dialog).getByRole("button", { name: "Enroll" }));

        await waitFor(() => expect(within(dialog).getByText(/mTLS credentials are only shown once/)).toBeInTheDocument());
        expect(within(dialog).getByText((_, element) => element?.tagName.toLowerCase() === "pre" && !!element.textContent?.includes("builder-a1b2c3"))).toBeInTheDocument();
        expect(onEnrolled).toHaveBeenCalledTimes(1);
    });

    it("surfaces a registerBuilder error without closing the enroll form", async () => {
        const mocks = [
            {
                request: { query: REGISTER_BUILDER, variables: { input: registerBuilderInput } },
                error: new Error("permission denied"),
            },
        ];
        renderModal(mocks);

        const dialog = await screen.findByRole("dialog");
        fireEvent.click(within(dialog).getByRole("button", { name: "Enroll" }));

        await waitFor(() => expect(within(dialog).getByText("Unable to enroll builder")).toBeInTheDocument());
        expect(within(dialog).getByRole("button", { name: "Enroll" })).toBeInTheDocument();
    });

    it("resets the enroll form state when Cancel is clicked", async () => {
        const onClose = vi.fn();
        renderModal([], { onClose });
        const dialog = await screen.findByRole("dialog");

        fireEvent.click(within(dialog).getByLabelText("Remove Linux"));
        fireEvent.click(within(dialog).getByLabelText("Remove macOS"));
        fireEvent.click(within(dialog).getByLabelText("Remove Windows"));
        await waitFor(() => expect(within(dialog).getByRole("button", { name: "Enroll" })).toBeDisabled());

        fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));

        expect(onClose).toHaveBeenCalledTimes(1);
        // Cancel resets enrollTargets back to all platforms -- whether or
        // not the dialog has finished its (CSS-driven, untestable in
        // jsdom) close transition, the underlying form state should be
        // back to its defaults.
        await waitFor(() => expect(within(dialog).getByLabelText("Remove Linux")).toBeInTheDocument());
        expect(within(dialog).getByRole("button", { name: "Enroll" })).not.toBeDisabled();
    });
});
