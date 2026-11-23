import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { CreateScheduledQuestModal } from "../CreateScheduledQuestModal";

const createScheduledTask = vi.fn().mockResolvedValue({ data: { createScheduledTask: { id: "1" } } });
const toast = vi.fn();

vi.mock("@apollo/client", async () => {
    const actual = await vi.importActual<typeof import("@apollo/client")>("@apollo/client");
    return { ...actual, useMutation: () => [createScheduledTask, { loading: false }] };
});

vi.mock("@chakra-ui/react", async () => {
    const actual = await vi.importActual<typeof import("@chakra-ui/react")>("@chakra-ui/react");
    return { ...actual, useToast: () => toast };
});

vi.mock("../../tavern-base-ui/Modal", () => ({
    default: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock("../tome-selection/TomeTableWrapper", () => ({
    TomeTableWrapper: ({ selectable, onSelectTome }: { selectable?: boolean; onSelectTome?: (tome: { id: string; paramDefs: string }) => void }) => (
        selectable
            ? <button type="button" onClick={() => onSelectTome?.({ id: "tome-1", paramDefs: "[]" })}>Select test tome</button>
            : <div>Test tome</div>
    ),
}));

describe("CreateScheduledQuestModal", () => {
    beforeEach(() => {
        createScheduledTask.mockClear();
        toast.mockClear();
    });

    const openTrigger = () => {
        render(<CreateScheduledQuestModal isOpen setOpen={vi.fn()} />);
        expect(screen.getByRole("heading", { name: "Choose a trigger" })).toBeInTheDocument();
        expect(screen.getByText("Choose when this quest should run")).toBeInTheDocument();
        expect(screen.queryByRole("textbox", { name: "Quest Name" })).not.toBeInTheDocument();
        expect(screen.queryByRole("textbox", { name: "Description" })).not.toBeInTheDocument();
    };

    const goToTome = async () => {
        fireEvent.click(screen.getByRole("button", { name: "Continue" }));
        await screen.findByRole("heading", { name: "Select a tome" });
        expect(screen.getByText("Choose a tome to execute on configured triggers")).toBeInTheDocument();
    };

    const goToConfirm = async () => {
        await goToTome();
        fireEvent.click(screen.getByRole("button", { name: "Select test tome" }));
        fireEvent.click(screen.getByRole("button", { name: "Continue" }));
        await screen.findByRole("heading", { name: "Confirm scheduled quest" });
        const nameField = screen.getByRole("textbox", { name: "Quest Name" });
        expect(screen.getByRole("heading", { name: "Trigger" })).toBeInTheDocument();
        const tomeHeading = screen.getByRole("heading", { name: "Tome" });
        expect(tomeHeading.compareDocumentPosition(nameField) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    };

    it.each([
        { choice: "Schedule", runOnSchedule: "0 * * * *", newHost: false, newBeacon: false },
        { choice: "New host", runOnSchedule: "", newHost: true, newBeacon: false },
        { choice: "New beacon", runOnSchedule: "", newHost: false, newBeacon: true },
    ])("saves the $choice trigger after confirmation", async ({ choice, runOnSchedule, newHost, newBeacon }) => {
        openTrigger();
        if (choice !== "Schedule") fireEvent.click(screen.getByRole("button", { name: choice }));

        await goToConfirm();
        expect(screen.getByText(choice, { selector: "dd" })).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "Create scheduled quest" }));

        await waitFor(() => expect(createScheduledTask).toHaveBeenCalledWith({
            variables: {
                input: expect.objectContaining({
                    tomeID: "tome-1",
                    description: "",
                    runOnSchedule,
                    runOnFirstHostCallback: newHost,
                    runOnNewBeaconCallback: newBeacon,
                }),
            },
        }));
    });

    it("keeps the schedule field on the schedule trigger only", async () => {
        openTrigger();
        expect(screen.getByRole("textbox", { name: "Schedule (cron)" })).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "New host" }));
        expect(screen.queryByRole("textbox", { name: "Schedule (cron)" })).not.toBeInTheDocument();
        expect(screen.getByText("This quest runs only when a new host checks in for the first time.")).toBeInTheDocument();
    });

    it("rejects an out-of-range cron minute before confirmation", async () => {
        openTrigger();
        fireEvent.change(screen.getByRole("textbox", { name: "Schedule (cron)" }), { target: { value: "99 * * * *" } });
        fireEvent.click(screen.getByRole("button", { name: "Continue" }));

        expect(await screen.findByText("Enter a valid five-field cron schedule, such as 0 * * * *.")).toBeInTheDocument();
        expect(screen.queryByRole("heading", { name: "Confirm scheduled quest" })).not.toBeInTheDocument();
        expect(createScheduledTask).not.toHaveBeenCalled();
    });

    it("saves an edited quest name from Confirm", async () => {
        openTrigger();
        await goToConfirm();
        fireEvent.change(screen.getByRole("textbox", { name: "Quest Name" }), { target: { value: "  Scheduled scan  " } });
        fireEvent.click(screen.getByRole("button", { name: "Create scheduled quest" }));

        await waitFor(() => expect(createScheduledTask).toHaveBeenCalledWith({
            variables: { input: expect.objectContaining({ name: "Scheduled scan" }) },
        }));
    });

    it("rejects a blank quest name on Confirm", async () => {
        openTrigger();
        await goToConfirm();
        fireEvent.change(screen.getByRole("textbox", { name: "Quest Name" }), { target: { value: "   " } });
        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "Create scheduled quest" }));
        });

        expect(screen.getByText("Quest name is required")).toBeInTheDocument();
        expect(createScheduledTask).not.toHaveBeenCalled();
    });
});
