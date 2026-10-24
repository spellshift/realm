import { describe, it, expect, beforeAll, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import { MockedProvider, MockedResponse } from "@apollo/client/testing";
import NewBuildProfile from "../NewBuildProfile";
import { CREATE_BUILD_PROFILE, GET_BUILD_PROFILES, profileInput } from "../queries";
import { BuildProfile, DEFAULT_SETUP_SCRIPT, DEFAULT_TRANSPORT, Tome } from "../types";

beforeAll(() => {
    if (typeof globalThis.ResizeObserver === "undefined") {
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        } as unknown as typeof ResizeObserver;
    }
});

const mockCatalog: Tome[] = [
    {
        id: "1",
        name: "Test Tome",
        description: "Test description",
        tactic: "DISCOVERY",
        paramDefs: "[]",
    },
];

const mockCreatedProfile = {
    id: "profile-123",
    name: "Stealth DNS",
    description: "",
    buildImage: "spellshift/devcontainer:main",
    buildScript: "{{.BuildCommand}}",
    artifactPath: "{{.ArtifactPath}}",
    prebuildscript: "echo 'no prebuild set'",
    setupscript: DEFAULT_SETUP_SCRIPT,
    postbuildscript: "echo 'no postbuild set'",
    unique: "",
    transports: [{ uri: "http://127.0.0.1:8000", interval: 5, type: "GRPC", extra: "" }],
    tomes: [],
};

const defaultProfile: BuildProfile = {
    id: "",
    name: "Stealth DNS",
    description: "",
    buildImage: "spellshift/devcontainer:main",
    buildScript: "{{.BuildCommand}}",
    artifactPath: "{{.ArtifactPath}}",
    prebuildscript: "echo 'no prebuild set'",
    setupscript: DEFAULT_SETUP_SCRIPT,
    postbuildscript: "echo 'no postbuild set'",
    unique: "",
    transports: [DEFAULT_TRANSPORT],
    tomes: [],
    tomeCount: 0,
    hasPersistenceTome: false,
};

const createProfileMock: MockedResponse = {
    request: {
        query: CREATE_BUILD_PROFILE,
        variables: {
            input: profileInput(defaultProfile),
        },
    },
    result: {
        data: {
            createBuildProfile: mockCreatedProfile,
        },
    },
};

const getBuildProfilesMock: MockedResponse = {
    request: {
        query: GET_BUILD_PROFILES,
    },
    result: {
        data: {
            buildProfiles: { edges: [{ node: mockCreatedProfile }] },
            tomes: { edges: [] },
        },
    },
};

function renderNewBuildProfile({
    mocks = [createProfileMock, getBuildProfilesMock, getBuildProfilesMock],
    onSave = vi.fn(),
    onCancel = vi.fn(),
}: {
    mocks?: MockedResponse[];
    onSave?: (profile: any) => Promise<void> | void;
    onCancel?: () => void;
}) {
    return render(
        <ChakraProvider>
            <MockedProvider mocks={mocks} addTypename={false}>
                <NewBuildProfile catalog={mockCatalog} onSave={onSave} onCancel={onCancel} />
            </MockedProvider>
        </ChakraProvider>
    );
}

describe("NewBuildProfile -- Async Profile Save Error Handling", () => {
    it("awaits onSave and passes the created profile", async () => {
        let resolveSave: () => void = () => {};
        const onSave = vi.fn().mockImplementation(() => new Promise<void>((resolve) => {
            resolveSave = resolve;
        }));
        const onCancel = vi.fn();

        renderNewBuildProfile({ onSave, onCancel });

        // Fill out required name field
        const nameInput = screen.getByPlaceholderText("e.g. Stealth DNS");
        fireEvent.change(nameInput, { target: { value: "Stealth DNS" } });

        const saveButton = screen.getByRole("button", { name: "Save profile" });
        fireEvent.click(saveButton);

        await waitFor(() => {
            expect(onSave).toHaveBeenCalledTimes(1);
        });

        expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
            id: "profile-123",
            name: "Stealth DNS",
        }));

        resolveSave();
    });

    it("surfaces async onSave errors using the existing AlertError pattern without duplicate creation on retry", async () => {
        const onSave = vi.fn()
            .mockRejectedValueOnce(new Error("Network error during profile refetch"))
            .mockResolvedValueOnce(undefined);
        const onCancel = vi.fn();

        renderNewBuildProfile({ onSave, onCancel });

        // Fill out required name field
        const nameInput = screen.getByPlaceholderText("e.g. Stealth DNS");
        fireEvent.change(nameInput, { target: { value: "Stealth DNS" } });

        const saveButton = screen.getByRole("button", { name: "Save profile" });
        fireEvent.click(saveButton);

        // Expect AlertError to display the error from async onSave
        await waitFor(() => {
            expect(screen.getByText("Unable to save profile")).toBeInTheDocument();
            expect(screen.getByText("Network error during profile refetch")).toBeInTheDocument();
        });

        expect(onCancel).not.toHaveBeenCalled();

        // Retry saving: it should retry onSave without calling createProfile again
        fireEvent.click(saveButton);

        await waitFor(() => {
            expect(onSave).toHaveBeenCalledTimes(2);
        });
    });
});
