import { describe, it, expect, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { ChakraProvider } from "@chakra-ui/react";
import Table from "../../../../components/tavern-base-ui/table/Table";
import { getProfileColumns } from "../ProfilesTable";
import { BuildProfile, DEFAULT_BUILD_PROFILE_NAME } from "../../types";
import { SupportedTransports } from "../../../../utils/enums";

const editableProfile: BuildProfile = {
    id: "profile-1",
    name: "Linux Prod Beacon",
    description: "Production Linux agent",
    buildImage: "spellshift/devcontainer:main",
    buildScript: "{{.BuildCommand}}",
    artifactPath: "{{.ArtifactPath}}",
    prebuildscript: "",
    setupscript: "",
    postbuildscript: "",
    unique: "",
    hasPersistenceTome: true,
    tomeCount: 0,
    transports: [
        { id: "first", type: SupportedTransports.HTTP1, uri: "https://example.test", interval: 5 },
        { id: "second", type: SupportedTransports.GRPC, uri: "grpc://example.test", interval: 5 },
    ],
    tomes: [],
};

const defaultProfile: BuildProfile = {
    ...editableProfile,
    id: "profile-default",
    name: DEFAULT_BUILD_PROFILE_NAME,
};

function renderTable(profiles: BuildProfile[], onEdit = vi.fn(), onDelete = vi.fn()) {
    const columns = getProfileColumns(onEdit, onDelete);
    render(
        <ChakraProvider>
            <Table data={profiles} columns={columns} />
        </ChakraProvider>
    );
    return { onEdit, onDelete };
}

describe("ProfilesTable", () => {
    it("renders edit and delete buttons for a non-default profile", () => {
        renderTable([editableProfile]);
        expect(screen.getByRole("columnheader", { name: "Transport" })).toBeInTheDocument();
        expect(screen.getByText("HTTP")).toBeInTheDocument();
        expect(screen.queryByText("gRPC")).not.toBeInTheDocument();
        expect(screen.getByRole("columnheader", { name: "Actions" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Edit" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Delete" })).toBeInTheDocument();
    });

    it("hides edit and delete buttons for the default profile", () => {
        renderTable([defaultProfile]);
        expect(screen.queryByRole("button", { name: "Edit" })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Delete" })).not.toBeInTheDocument();
        expect(screen.getByText("Default profile")).toBeInTheDocument();
    });

    it("calls onEdit/onDelete with the clicked profile", async () => {
        const { onEdit, onDelete } = renderTable([editableProfile]);
        const { fireEvent } = await import("@testing-library/react");
        fireEvent.click(screen.getByRole("button", { name: "Edit" }));
        expect(onEdit).toHaveBeenCalledWith(editableProfile);
        fireEvent.click(screen.getByRole("button", { name: "Delete" }));
        expect(onDelete).toHaveBeenCalledWith(editableProfile);
    });

    it("shows edit/delete for non-default rows but not the default row in the same table", () => {
        renderTable([editableProfile, defaultProfile]);
        const rows = screen.getAllByRole("row").slice(1); // skip header row
        expect(within(rows[0]).getByRole("button", { name: "Edit" })).toBeInTheDocument();
        expect(within(rows[1]).queryByRole("button", { name: "Edit" })).not.toBeInTheDocument();
    });
});
