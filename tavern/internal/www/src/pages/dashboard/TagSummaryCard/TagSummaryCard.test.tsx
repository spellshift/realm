import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { TagSummaryCard } from "./TagSummaryCard";

const mocks = vi.hoisted(() => ({
    isAdmin: true,
    rows: [] as Array<{ tagName: string }>,
    hostCount: 1,
    subnet: vi.fn(),
    roles: vi.fn(),
    refetch: vi.fn(),
    toast: vi.fn(),
}));

vi.mock("@apollo/client", async (importOriginal) => ({
    ...await importOriginal<typeof import("@apollo/client")>(),
    useMutation: (query: { definitions: Array<{ name?: { value: string } }> }) => [
        query.definitions[0]?.name?.value === "GroupHostsBySubnet" ? mocks.subnet : mocks.roles,
    ],
    useApolloClient: () => ({ refetchQueries: mocks.refetch }),
}));
vi.mock("@chakra-ui/react", () => ({
    useToast: () => mocks.toast,
    Checkbox: ({ children, isChecked, onChange, isDisabled }: {
        children: React.ReactNode;
        isChecked: boolean;
        onChange: React.ChangeEventHandler<HTMLInputElement>;
        isDisabled: boolean;
    }) => (
        <label><input type="checkbox" checked={isChecked} onChange={onChange} disabled={isDisabled} />{children}</label>
    ),
}));
vi.mock("../../../components/tavern-base-ui/FormTextField", () => ({
    default: ({ label, htmlFor, ...props }: React.InputHTMLAttributes<HTMLInputElement> & { label: string; htmlFor: string }) => (
        <label htmlFor={htmlFor}>{label}<input {...props} id={htmlFor} /></label>
    ),
}));
vi.mock("../../../context/AuthorizationContext", () => ({
    useAuthorization: () => ({ data: { me: { isAdmin: mocks.isAdmin } } }),
}));
vi.mock("./useTagSummaryData", () => ({
    useTagSummaryData: () => ({ rows: mocks.rows, hostCount: mocks.hostCount, loading: false }),
}));
vi.mock("./useTagSummaryColumns", () => ({ useTagSummaryColumns: () => [] }));
vi.mock("../../../components/tavern-base-ui/table/Table", () => ({
    Table: () => <div>Tag table</div>,
}));

describe("Dashboard tag summary auto tagging", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mocks.isAdmin = true;
        mocks.rows = [];
        mocks.hostCount = 1;
        mocks.subnet.mockResolvedValue({ data: { groupHostsBySubnet: [{ id: "1" }] } });
        mocks.roles.mockResolvedValue({ data: { runHostAutogroup: [{ id: "2" }] } });
        mocks.refetch.mockResolvedValue(undefined);
    });

    it("shows the action only to admins when the tag summary is empty", () => {
        const { rerender } = render(<TagSummaryCard />);
        expect(screen.getByRole("button", { name: "Auto tag all", exact: true })).toBeInTheDocument();

        mocks.isAdmin = false;
        rerender(<TagSummaryCard />);
        expect(screen.queryByRole("button", { name: "Auto tag all", exact: true })).not.toBeInTheDocument();

        mocks.isAdmin = true;
        mocks.rows = [{ tagName: "group-a" }];
        rerender(<TagSummaryCard />);
        expect(screen.queryByRole("button", { name: "Auto tag all", exact: true })).not.toBeInTheDocument();
    });

    it("greys out both auto-tag controls when there are no hosts", () => {
        mocks.hostCount = 0;
        render(<TagSummaryCard />);

        expect(screen.getByText("No hosts found")).toBeInTheDocument();
        expect(screen.getByText("Add a host before auto tagging.")).toBeInTheDocument();
        const runButton = screen.getByRole("button", { name: "Auto tag all", exact: true });
        const optionsButton = screen.getByRole("button", { name: "Advanced auto tag options" });
        expect(runButton).toBeDisabled();
        expect(optionsButton).toBeDisabled();

        fireEvent.click(runButton);
        fireEvent.click(optionsButton);
        expect(mocks.subnet).not.toHaveBeenCalled();
        expect(mocks.roles).not.toHaveBeenCalled();
        expect(screen.queryByText("Advanced auto tagging")).not.toBeInTheDocument();
    });

    it("runs group and automatic service tagging in order", async () => {
        const calls: string[] = [];
        mocks.subnet.mockImplementation(async () => {
            calls.push("groups");
            return { data: { groupHostsBySubnet: [{ id: "1" }] } };
        });
        mocks.roles.mockImplementation(async () => {
            calls.push("services");
            return { data: { runHostAutogroup: [{ id: "2" }] } };
        });
        render(<TagSummaryCard />);
        fireEvent.click(screen.getByRole("button", { name: "Auto tag all", exact: true }));
        await waitFor(() => expect(calls).toEqual(["groups", "services"]));
        expect(mocks.roles).toHaveBeenCalledWith({ variables: { k: null } });
        await waitFor(() => expect(mocks.refetch).toHaveBeenCalled());
    });

    it("can refresh group tags without creating service tags", async () => {
        render(<TagSummaryCard />);
        fireEvent.click(screen.getByRole("button", { name: "Advanced auto tag options" }));
        fireEvent.click(await screen.findByRole("checkbox", { name: /Services/ }));
        fireEvent.click(screen.getByRole("button", { name: "Run selected" }));
        await waitFor(() => expect(mocks.subnet).toHaveBeenCalledTimes(1));
        expect(mocks.roles).not.toHaveBeenCalled();
        await waitFor(() => expect(screen.queryByRole("button", { name: "Run selected" })).not.toBeInTheDocument());
    });

    it("passes a chosen service count and rejects invalid counts", async () => {
        render(<TagSummaryCard />);
        fireEvent.click(screen.getByRole("button", { name: "Advanced auto tag options" }));
        fireEvent.click(await screen.findByRole("checkbox", { name: /Groups/ }));
        const count = await screen.findByRole("spinbutton", { name: "Number of service tags (optional)" });
        fireEvent.change(count, { target: { value: "0" } });
        fireEvent.click(screen.getByRole("button", { name: "Run selected" }));
        expect(mocks.roles).not.toHaveBeenCalled();

        fireEvent.change(count, { target: { value: "3" } });
        fireEvent.click(screen.getByRole("button", { name: "Run selected" }));
        await waitFor(() => expect(mocks.roles).toHaveBeenCalledWith({ variables: { k: 3 } }));
        expect(mocks.roles).toHaveBeenCalledTimes(1);
        await waitFor(() => expect(screen.queryByRole("button", { name: "Run selected" })).not.toBeInTheDocument());
    });

    it("requires at least one advanced tag type", async () => {
        render(<TagSummaryCard />);
        fireEvent.click(screen.getByRole("button", { name: "Advanced auto tag options" }));
        fireEvent.click(await screen.findByRole("checkbox", { name: /Groups/ }));
        fireEvent.click(screen.getByRole("checkbox", { name: /Services/ }));
        expect(screen.getByRole("button", { name: "Run selected" })).toBeDisabled();
        expect(screen.queryByRole("spinbutton", { name: "Number of service tags (optional)" })).not.toBeInTheDocument();
    });
});
