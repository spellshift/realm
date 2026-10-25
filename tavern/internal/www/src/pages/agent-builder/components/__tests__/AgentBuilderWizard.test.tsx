import { describe, it, expect, beforeAll, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import AgentBuilderWizard from "../AgentBuilderWizard";
import { GET_BUILD_PROFILES, GET_ONLINE_BUILDER_PLATFORMS } from "../../queries";
import { AuthorizationContext } from "../../../../context/AuthorizationContext";

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
    request: { query: GET_ONLINE_BUILDER_PLATFORMS },
    result: {
        data: {
            onlineBuilderPlatforms: [],
        },
    },
};

const profilesErrorMock = {
    request: { query: GET_BUILD_PROFILES },
    error: new Error("boom"),
};

const windowsBuilderMock = {
    request: { query: GET_ONLINE_BUILDER_PLATFORMS },
    result: { data: { onlineBuilderPlatforms: ["PLATFORM_WINDOWS"] } },
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

function renderWizard(profilesMock: MockedResponse, buildersMock: MockedResponse = emptyBuildersMock, isAdmin = false) {
    return render(
        <MemoryRouter>
            <ChakraProvider>
                <AuthorizationContext.Provider value={{ data: { me: { isAdmin } as never }, isLoading: false, error: undefined }}>
                    <MockedProvider mocks={[profilesMock, buildersMock]} addTypename={false}>
                        <AgentBuilderWizard />
                    </MockedProvider>
                </AuthorizationContext.Provider>
            </ChakraProvider>
        </MemoryRouter>
    );
}

describe("AgentBuilderWizard", () => {
    it("renders empty state asking an admin to add a profile and omits New profile button when no profiles exist", async () => {
        renderWizard(emptyProfilesMock, windowsBuilderMock);

        await waitFor(() => {
            expect(screen.getByText("No build profiles available")).toBeInTheDocument();
        });

        expect(screen.getByText("Please ask an admin to add a profile in the admin panel.")).toBeInTheDocument();

        // Ensure "New profile" and "Add build profile" buttons are NOT rendered on the form
        expect(screen.queryByRole("button", { name: "New profile" })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Add build profile" })).not.toBeInTheDocument();
    });

    it("renders profile options without New profile button when profiles exist", async () => {
        renderWizard(populatedProfilesMock, windowsBuilderMock);

        await waitFor(() => {
            expect(screen.getByText("Linux Default Beacon")).toBeInTheDocument();
        });

        expect(screen.queryByRole("button", { name: "New profile" })).not.toBeInTheDocument();
    });

    it("shows a full-page empty state without the wizard for a non-admin when no builder is online", async () => {
        renderWizard(populatedProfilesMock);

        await waitFor(() => expect(screen.getByText("No online builders available")).toBeInTheDocument());
        expect(screen.getByText("Ask an admin to enroll and start a builder before generating an agent.")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Enroll builder" })).not.toBeInTheDocument();
        expect(screen.queryByText("Select build profile")).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "continue profile step" })).not.toBeInTheDocument();
    });

    it("still surfaces a build profile load error while the no-online-builders empty state is shown", async () => {
        renderWizard(profilesErrorMock, emptyBuildersMock);
        await waitFor(() => expect(screen.getByText("No online builders available")).toBeInTheDocument());
        expect(screen.getByText("Unable to load build profiles")).toBeInTheDocument();
    });

    it("shows enrollment guidance on the first step according to admin access", async () => {
        renderWizard(populatedProfilesMock, emptyBuildersMock, true);
        await waitFor(() => expect(screen.getByText("No online builders available")).toBeInTheDocument());
        expect(screen.getByRole("button", { name: "Enroll builder" })).toBeInTheDocument();
        expect(screen.queryByText("Select build profile")).not.toBeInTheDocument();
    });

    it("keeps target compatibility gating when another platform has an online builder", async () => {
        renderWizard(populatedProfilesMock, windowsBuilderMock);
        await waitFor(() => expect(screen.getByText("Linux Default Beacon")).toBeInTheDocument());
        expect(screen.queryByText("No online builders available")).not.toBeInTheDocument();

        fireEvent.click(screen.getAllByRole("row")[1]);
        fireEvent.click(screen.getByRole("button", { name: "continue profile step" }));
        fireEvent.click(screen.getAllByRole("row")[1]);
        fireEvent.click(screen.getByRole("button", { name: "continue target step" }));

        expect(screen.getByText("No compatible online builder")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "submit agent build" })).toBeDisabled();
    });

    it("shows static transport details on confirmation", async () => {
        const profileWithExtra = structuredClone(populatedProfilesMock);
        profileWithExtra.result.data.buildProfiles.edges[0].node.transports[0].extra = "proxy=enabled";
        renderWizard(profileWithExtra, windowsBuilderMock);
        await waitFor(() => expect(screen.getByText("Linux Default Beacon")).toBeInTheDocument());
        fireEvent.click(screen.getAllByRole("row")[1]);
        fireEvent.click(screen.getByRole("button", { name: "continue profile step" }));
        fireEvent.click(screen.getAllByRole("row")[1]);
        fireEvent.click(screen.getByRole("button", { name: "continue target step" }));

        expect(screen.getByText("http://127.0.0.1:8000")).toBeInTheDocument();
        expect(screen.getByText("5s interval")).toBeInTheDocument();
        expect(screen.getByText("proxy=enabled")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /Toggle details for/ })).not.toBeInTheDocument();
        expect(screen.queryByText("Beacon interval:")).not.toBeInTheDocument();
    });
});
