import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import Table from "../../../../components/tavern-base-ui/table/Table";
import { buildTaskColumns } from "../AgentsTable";
import { BuildTaskRow } from "../../types";
import { SupportedPlatforms, SupportedFormats } from "../../../../utils/enums";

function row(overrides: Partial<BuildTaskRow>): BuildTaskRow {
    return {
        id: "t1",
        status: "Queued",
        profileName: "Default Imix",
        platform: SupportedPlatforms.Linux,
        format: SupportedFormats.Bin,
        error: null,
        exitCode: null,
        creator: null,
        timestamp: "2m ago",
        assetId: null,
        assetName: null,
        artifactName: null,
        artifactSize: null,
        artifactHash: null,
        ...overrides,
    };
}

const renderTable = (rows: BuildTaskRow[], onCreateLink = vi.fn(), onDeleteAsset = vi.fn()) =>
    render(
        <ChakraProvider>
            <Table data={rows} columns={buildTaskColumns(onCreateLink, undefined, onDeleteAsset)} />
        </ChakraProvider>
    );

describe("buildTaskColumns", () => {
    it("shows the profile name and no artifact/hash columns for in-progress rows", () => {
        renderTable([row({ id: "q1", status: "Queued" })]);
        expect(screen.getByText("Profile: Default Imix")).toBeInTheDocument();
        expect(screen.queryByLabelText("Download")).not.toBeInTheDocument();
        expect(screen.queryByLabelText("Create Link")).not.toBeInTheDocument();
        expect(screen.queryByLabelText("Delete")).not.toBeInTheDocument();
    });

    it("shows download/create-link/delete actions, size, and hash only for Built rows", () => {
        renderTable([
            row({
                id: "b1",
                status: "Built",
                assetId: "a1",
                assetName: "build/linux/bin/imix-default-imix",
                artifactName: "imix-default-imix",
                artifactSize: "6.10 MB",
                artifactHash: "abcdef0123456789",
            }),
        ]);
        expect(screen.getByText("imix-default-imix")).toBeInTheDocument();
        expect(screen.getByText("Profile: Default Imix")).toBeInTheDocument();
        expect(screen.getByText("6.10 MB")).toBeInTheDocument();
        expect(screen.getByText("abcdef012345...")).toBeInTheDocument();
        expect(screen.getByLabelText("Download")).toBeInTheDocument();
        expect(screen.getByLabelText("Create Link")).toBeInTheDocument();
        expect(screen.getByLabelText("Delete")).toBeInTheDocument();
    });

    it("surfaces the build failure reason inline instead of a Details tooltip", () => {
        renderTable([row({ id: "f1", status: "Failed", error: "build step failed", exitCode: 1 })]);
        expect(screen.getByText("Build failed (exit code 1): build step failed")).toBeInTheDocument();
        expect(screen.queryByLabelText("Download")).not.toBeInTheDocument();
        expect(screen.queryByText("Details")).not.toBeInTheDocument();
    });

    it("shows a dash in the hash column when no artifact hash is present", () => {
        renderTable([row({ id: "h1", status: "Queued" })]);
        expect(screen.getAllByText("—").length).toBeGreaterThan(0);
    });

    it("renders friendly labels for the raw gRPC platform enum values", () => {
        renderTable([row({ id: "w1", platform: SupportedPlatforms.Windows, format: SupportedFormats.Cdylib })]);
        expect(screen.getByText("Windows")).toBeInTheDocument();
    });

    it("tags the highlighted row with a New badge", () => {
        render(
            <ChakraProvider>
                <Table data={[row({ id: "n1" })]} columns={buildTaskColumns(vi.fn(), "n1", vi.fn())} />
            </ChakraProvider>
        );
        expect(screen.getByText("New")).toBeInTheDocument();
    });

    it("renders Creator and Actions column headers", () => {
        renderTable([row({ id: "c1" })]);
        expect(screen.getByText("Creator")).toBeInTheDocument();
        expect(screen.getByText("Actions")).toBeInTheDocument();
    });

    it("renders user information in the creator column when present", () => {
        renderTable([
            row({
                id: "c2",
                creator: {
                    id: "u1",
                    name: "Alice Operator",
                    photoURL: "",
                    isActivated: true,
                    isAdmin: false,
                },
            }),
        ]);
        expect(screen.getByText("Alice Operator")).toBeInTheDocument();
    });
});
