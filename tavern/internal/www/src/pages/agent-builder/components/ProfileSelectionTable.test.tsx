import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import ProfileSelectionTable from "./ProfileSelectionTable";
import { BuildProfile } from "../types";
import { SupportedTransports } from "../../../utils/enums";

const mockProfiles: BuildProfile[] = [
    {
        id: "profile-1",
        name: "Linux Prod Beacon",
        description: "Production Linux agent with persistence",
        buildImage: "spellshift/devcontainer:main",
        buildScript: "cargo build --release\ncp target/release/imix /out/",
        artifactPath: "/out/imix",
        prebuildscript: "",
        setupscript: "",
        postbuildscript: "",
        unique: "",
        hasPersistenceTome: true,
        tomeCount: 1,
        transports: [
            { id: "t-1", type: SupportedTransports.HTTP1, uri: "https://c2.example.com/api", interval: 10, extra: "" },
        ],
        tomes: [
            {
                id: "tome-1",
                tomeId: "101",
                tomeName: "Cron Persistence",
                tactic: "PERSISTENCE",
                params: "{}",
                paramSummary: "No params",
            },
        ],
    },
    {
        id: "profile-2",
        name: "Windows Diagnostic",
        description: "Ad-hoc diagnostic agent",
        buildImage: "spellshift/devcontainer:windows",
        buildScript: "",
        artifactPath: "/out/imix.exe",
        prebuildscript: "",
        setupscript: "",
        postbuildscript: "",
        unique: "",
        hasPersistenceTome: false,
        tomeCount: 0,
        transports: [],
        tomes: [],
    },
];

describe("ProfileSelectionTable", () => {
    it("renders profile names, transports, tomes, and selection indicators", () => {
        const onSelect = vi.fn();
        render(
            <ChakraProvider>
                <ProfileSelectionTable
                    profiles={mockProfiles}
                    selectedProfileId="profile-1"
                    onSelectProfile={onSelect}
                />
            </ChakraProvider>
        );

        expect(screen.getByText("Linux Prod Beacon")).toBeInTheDocument();
        expect(screen.getByText("Windows Diagnostic")).toBeInTheDocument();
        expect(screen.getByText("HTTP")).toBeInTheDocument();
        expect(screen.getByText("https://c2.example.com/api")).toBeInTheDocument();

        // Warning icon for missing persistence
        expect(screen.getByLabelText("No install warning")).toBeInTheDocument();
    });

    it("triggers onSelectProfile when clicking a row", () => {
        const onSelect = vi.fn();
        render(
            <ChakraProvider>
                <ProfileSelectionTable
                    profiles={mockProfiles}
                    selectedProfileId="profile-1"
                    onSelectProfile={onSelect}
                />
            </ChakraProvider>
        );

        fireEvent.click(screen.getByRole("button", { name: "Select profile Windows Diagnostic" }));
        expect(onSelect).toHaveBeenCalledWith(mockProfiles[1]);
    });

    it("expands a row to show additional profile details when chevron is clicked", () => {
        const onSelect = vi.fn();
        render(
            <ChakraProvider>
                <ProfileSelectionTable
                    profiles={mockProfiles}
                    selectedProfileId="profile-1"
                    onSelectProfile={onSelect}
                />
            </ChakraProvider>
        );

        // Before expansion, details are not visible
        expect(screen.queryByText("spellshift/devcontainer:main")).not.toBeInTheDocument();

        // Click expand button on profile 1
        const expandButtons = screen.getAllByRole("button", { name: "Expand row" });
        fireEvent.click(expandButtons[0]);

        // Details should now be visible
        expect(screen.getByText("spellshift/devcontainer:main")).toBeInTheDocument();
        expect(screen.getByText("/out/imix")).toBeInTheDocument();
        expect(screen.getByText("Production Linux agent with persistence")).toBeInTheDocument();
        expect(screen.getByText("Cron Persistence")).toBeInTheDocument();
        expect(screen.getByText("10s interval")).toBeInTheDocument();
    });
});
