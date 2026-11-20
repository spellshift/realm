import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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

    const goToTrigger = async () => {
        render(<CreateScheduledQuestModal isOpen setOpen={vi.fn()} />);
        fireEvent.change(screen.getByRole("textbox", { name: "Description" }), { target: { value: "Runs a test tome" } });
        fireEvent.click(screen.getByRole("button", { name: "Continue" }));
        await screen.findByRole("heading", { name: "Select a tome" });
        fireEvent.click(screen.getByRole("button", { name: "Select test tome" }));
        fireEvent.click(screen.getByRole("button", { name: "Continue" }));
        await screen.findByRole("heading", { name: "Choose a trigger" });
    };

    it.each([
        { choice: "Schedule", runOnSchedule: "0 * * * *", newHost: false, newBeacon: false },
        { choice: "New host", runOnSchedule: "", newHost: true, newBeacon: false },
        { choice: "New beacon", runOnSchedule: "", newHost: false, newBeacon: true },
    ])("saves the $choice trigger after confirmation", async ({ choice, runOnSchedule, newHost, newBeacon }) => {
        await goToTrigger();
        if (choice !== "Schedule") fireEvent.click(screen.getByRole("button", { name: choice }));

        fireEvent.click(screen.getByRole("button", { name: "Continue" }));
        await screen.findByRole("heading", { name: "Confirm scheduled quest" });
        expect(screen.getByText(choice, { selector: "dd" })).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "Create scheduled quest" }));

        await waitFor(() => expect(createScheduledTask).toHaveBeenCalledWith({
            variables: {
                input: expect.objectContaining({
                    tomeID: "tome-1",
                    description: "Runs a test tome",
                    runOnSchedule,
                    runOnFirstHostCallback: newHost,
                    runOnNewBeaconCallback: newBeacon,
                }),
            },
        }));
    });

    it("keeps the schedule field on the schedule trigger only", async () => {
        await goToTrigger();
        expect(screen.getByRole("textbox", { name: "Schedule (cron)" })).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "New host" }));
        expect(screen.queryByRole("textbox", { name: "Schedule (cron)" })).not.toBeInTheDocument();
        expect(screen.getByText("This quest runs only when a new host checks in for the first time.")).toBeInTheDocument();
    });
});
