import { describe, it, expect, beforeAll, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import AgentBuilderWizard from "./AgentBuilderWizard";
import { GET_BUILD_PROFILES, GET_BUILDERS } from "../queries";

vi.mock("@tanstack/react-virtual", async () => {
    const React = await import("react");
    return {
        useVirtualizer: ({ count }: { count: number }) => {
            const virtualizer = React.useMemo(() => ({
                getVirtualItems: () =>
                    Array.from({ length: count }, (_, index) => ({
                        index,
                        key: index,
                        start: index * 80,
                        size: 80,
                        end: (index + 1) * 80,
                    })),
                getTotalSize: () => count * 80,
                measureElement: () => {},
                range: { startIndex: 0, endIndex: Math.max(0, count - 1) },
            }), [count]);
            return virtualizer;
        },
    };
});

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
    result: {
        data: {
            buildProfiles: { edges: [] },
            tomes: { edges: [] },
        },
    },
};

const emptyBuildersMock = {
    request: { query: GET_BUILDERS },
    result: {
        data: {
            builders: { edges: [] },
        },
    },
};

const populatedProfilesMock = {
    request: { query: GET_BUILD_PROFILES },
    result: {
        data: {
            buildProfiles: {
                edges: [
                    {
                        node: {
                            __typename: "BuildProfile",
                            id: "profile-1",
                            name: "Linux Default Beacon",
                            description: "Standard Linux agent",
                            buildImage: "spellshift/devcontainer:main",
                            buildScript: "{{.BuildCommand}}",
                            artifactPath: "{{.ArtifactPath}}",
                            prebuildscript: "",
                            setupscript: "",
                            postbuildscript: "",
                            unique: "",
                            transports: [{ __typename: "BuildProfileTransport", uri: "http://127.0.0.1:8000", interval: 5, type: "GRPC", extra: "" }],
                            tomes: [],
                        },
                    },
                ],
            },
            tomes: { edges: [] },
        },
    },
};

describe("AgentBuilderWizard", () => {
    it("renders empty state asking an admin to add a profile and omits New profile button when no profiles exist", async () => {
        render(
            <MemoryRouter>
                <ChakraProvider>
                    <MockedProvider mocks={[emptyProfilesMock, emptyBuildersMock]} addTypename={false}>
                        <AgentBuilderWizard />
                    </MockedProvider>
                </ChakraProvider>
            </MemoryRouter>
        );

        await waitFor(() => {
            expect(screen.getByText("No build profiles available")).toBeInTheDocument();
        });

        expect(screen.getByText("Please ask an admin to add a profile in the admin panel.")).toBeInTheDocument();

        // Ensure "New profile" and "Add build profile" buttons are NOT rendered on the form
        expect(screen.queryByRole("button", { name: "New profile" })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Add build profile" })).not.toBeInTheDocument();
    });

    it("renders profile options without New profile button when profiles exist", async () => {
        render(
            <MemoryRouter>
                <ChakraProvider>
                    <MockedProvider mocks={[populatedProfilesMock, emptyBuildersMock]} addTypename={false}>
                        <AgentBuilderWizard />
                    </MockedProvider>
                </ChakraProvider>
            </MemoryRouter>
        );

        await waitFor(() => {
            expect(screen.getByText("Linux Default Beacon")).toBeInTheDocument();
        });

        expect(screen.queryByRole("button", { name: "New profile" })).not.toBeInTheDocument();
    });
});
