import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { MemoryRouter } from "react-router-dom";
import { SupportedPlatforms, SupportedTransports } from "../../../utils/enums";
import { BuildProfile } from "../types";
import AgentBuilderWizard from "./AgentBuilderWizard";

const mockProfiles: BuildProfile[] = [
    {
        id: "1",
        name: "Default",
        description: "Default profile",
        transports: [{ id: "t1", type: SupportedTransports.GRPC, uri: "http://127.0.0.1:8000", interval: 30, jitter: 0 }],
        buildImage: "imix-builder:latest",
        tomes: [],
        tomeCount: 0,
        hasPersistenceTome: true,
        buildScript: "cargo build",
        artifactPath: "target/release/imix",
        prebuildscript: "",
        setupscript: "",
        postbuildscript: "",
    },
    {
        id: "2",
        name: "Prod grpc",
        description: "Production gRPC callbacks",
        transports: [{ id: "t2", type: SupportedTransports.GRPC, uri: "https://example.com:443", interval: 60, jitter: 0.1 }],
        buildImage: "imix-builder:latest",
        tomes: [],
        tomeCount: 0,
        hasPersistenceTome: true,
        buildScript: "cargo build",
        artifactPath: "target/release/imix",
        prebuildscript: "",
        setupscript: "",
        postbuildscript: "",
    },
];

vi.mock("@apollo/client", async (importOriginal) => {
    const original = await importOriginal<typeof import("@apollo/client")>();
    return {
        ...original,
        useMutation: () => [vi.fn(), { loading: false }],
    };
});

vi.mock("../queries", () => ({
    CREATE_BUILD_TASK: {},
    useBuildProfiles: () => ({ profiles: mockProfiles, loading: false, error: undefined }),
    useOnlineBuilderPlatforms: () => ({ platforms: [SupportedPlatforms.Linux], loading: false, error: undefined }),
}));

vi.mock("../../../context/AuthorizationContext", () => ({
    useAuthorization: () => ({ data: { me: { isAdmin: true } } }),
}));

describe("AgentBuilderWizard", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("pre-selects the profile matching initialProfileId", () => {
        render(
            <MemoryRouter>
                <AgentBuilderWizard showHeader={false} initialProfileId="2" />
            </MemoryRouter>
        );

        fireEvent.click(screen.getByRole("button", { name: "continue profile step" }));
        fireEvent.click(screen.getByRole("button", { name: "continue target step" }));

        expect(screen.getByText("Confirm build details")).toBeInTheDocument();
        expect(screen.getByText("Prod grpc")).toBeInTheDocument();
    });
});
