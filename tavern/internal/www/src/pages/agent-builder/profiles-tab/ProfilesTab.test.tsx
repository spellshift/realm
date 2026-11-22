import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { SupportedTransports } from "../../../utils/enums";
import { BuildProfile } from "../types";
import { ProfilesTab } from "./ProfilesTab";

const makeProfile = (overrides: Partial<BuildProfile> = {}): BuildProfile => ({
    id: "1",
    name: "Default",
    description: "",
    transports: [
        {
            id: "t1",
            type: SupportedTransports.GRPC,
            uri: "http://127.0.0.1:8000",
            interval: 30,
            jitter: 0,
        },
    ],
    buildImage: "imix-builder:latest",
    tomes: [],
    tomeCount: 0,
    hasPersistenceTome: false,
    buildScript: "cargo build",
    artifactPath: "target/release/imix",
    prebuildscript: "",
    setupscript: "",
    postbuildscript: "",
    ...overrides,
});

describe("ProfilesTab", () => {
    it("renders a Generate asset button for all profiles including Default and invokes onGenerateAsset", () => {
        const defaultProfile = makeProfile({ id: "1", name: "Default" });
        const customProfile = makeProfile({ id: "2", name: "Prod grpc", description: "Production gRPC callbacks" });
        const onGenerateAsset = vi.fn();

        render(
            <ProfilesTab
                profiles={[defaultProfile, customProfile]}
                loading={false}
                onAddProfile={vi.fn()}
                onEditProfile={vi.fn()}
                onDeleteProfile={vi.fn()}
                onGenerateAsset={onGenerateAsset}
            />
        );

        const generateButtons = screen.getAllByRole("button", { name: "Generate asset" });
        expect(generateButtons).toHaveLength(2);
        expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(1);
        expect(screen.getAllByRole("button", { name: "Delete" })).toHaveLength(1);

        fireEvent.click(generateButtons[0]);
        expect(onGenerateAsset).toHaveBeenCalledWith(defaultProfile);

        fireEvent.click(generateButtons[1]);
        expect(onGenerateAsset).toHaveBeenCalledWith(customProfile);
    });
});
