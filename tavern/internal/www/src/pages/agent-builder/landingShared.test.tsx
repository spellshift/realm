import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import "@testing-library/jest-dom";
import { ChakraProvider } from "@chakra-ui/react";
import Table from "../../components/tavern-base-ui/table/Table";
import { buildTaskColumns } from "./landingShared";
import { BuildTaskRow } from "./types";

function row(overrides: Partial<BuildTaskRow>): BuildTaskRow {
    return {
        id: "t1",
        status: "Queued",
        profileName: "Default Imix",
        platform: "LINUX",
        format: "BIN",
        error: null,
        timestamp: "2m ago",
        assetId: null,
        assetName: null,
        artifactName: null,
        artifactSize: null,
        ...overrides,
    };
}

const renderTable = (rows: BuildTaskRow[], onCreateLink = vi.fn()) =>
    render(
        <ChakraProvider>
            <Table data={rows} columns={buildTaskColumns(onCreateLink)} />
        </ChakraProvider>
    );

describe("buildTaskColumns", () => {
    it("shows a live status badge and no artifact columns for in-progress rows", () => {
        renderTable([row({ id: "q1", status: "Queued" })]);
        expect(screen.getByText("Queued")).toBeInTheDocument();
        expect(screen.getAllByText("—")).toHaveLength(2); // Artifact + Size columns
        expect(screen.queryByLabelText("Download")).not.toBeInTheDocument();
        expect(screen.queryByLabelText("Create Link")).not.toBeInTheDocument();
    });

    it("shows download/create-link actions only for Built rows", () => {
        renderTable([
            row({
                id: "b1",
                status: "Built",
                assetId: "a1",
                assetName: "build/linux/bin/imix-default-imix",
                artifactName: "imix",
                artifactSize: "6.10 MB",
            }),
        ]);
        expect(screen.getByText("Built")).toBeInTheDocument();
        expect(screen.getByText("imix")).toBeInTheDocument();
        expect(screen.getByText("6.10 MB")).toBeInTheDocument();
        expect(screen.getByLabelText("Download")).toBeInTheDocument();
        expect(screen.getByLabelText("Create Link")).toBeInTheDocument();
    });

    it("surfaces the build error for Failed rows instead of download actions", () => {
        renderTable([row({ id: "f1", status: "Failed", error: "exit code 1: build step failed" })]);
        expect(screen.getByText("Failed")).toBeInTheDocument();
        expect(screen.getByText("Details")).toBeInTheDocument();
        expect(screen.queryByLabelText("Download")).not.toBeInTheDocument();
    });
});
