import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { CreateScheduledQuestModal } from "../CreateScheduledQuestModal";

const createScheduledTask = vi.fn().mockResolvedValue({ data: { createScheduledTask: { id: "1" } } });
const toast = vi.fn();
let mutationLoading = false;

vi.mock("@apollo/client", async () => {
    const actual = await vi.importActual<typeof import("@apollo/client")>("@apollo/client");
    return { ...actual, useMutation: () => [createScheduledTask, { loading: mutationLoading }] };
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
        selectable ? (
            <div>
                <button type="button" onClick={() => onSelectTome?.({ id: "tome-1", paramDefs: "[]" })}>
                    Select test tome
                </button>
                <button
                    type="button"
                    onClick={() => onSelectTome?.({
                        id: "tome-with-param",
                        paramDefs: JSON.stringify([{ name: "target", label: "Target", type: "string", placeholder: "" }]),
                    })}
                >
                    Select parameterized tome
                </button>
            </div>
        ) : <div>Test tome</div>
    ),
}));

describe("CreateScheduledQuestModal", () => {
    beforeEach(() => {
        createScheduledTask.mockClear();
        toast.mockClear();
        mutationLoading = false;
    });

    const openTrigger = (scheduledHost?: { id: string; name: string }) => {
        render(<CreateScheduledQuestModal isOpen setOpen={vi.fn()} scheduledHost={scheduledHost} />);
        expect(screen.getByRole("heading", { name: "Choose a trigger" })).toBeInTheDocument();
        expect(screen.getByText("Choose when this quest should run")).toBeInTheDocument();
        expect(screen.queryByRole("textbox", { name: "Quest name" })).not.toBeInTheDocument();
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
        const nameField = screen.getByRole("textbox", { name: "Quest name" });
        expect(screen.getByRole("heading", { name: "Trigger" })).toBeInTheDocument();
        const tomeHeading = screen.getByRole("heading", { name: "Tome" });
        expect(tomeHeading.compareDocumentPosition(nameField) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    };

    it("places form action buttons above the form fields on every step", async () => {
        openTrigger();
        const triggerContinue = screen.getByRole("button", { name: "Continue" });
        const triggerGroup = screen.getByRole("radiogroup", { name: "Quest trigger" });
        expect(triggerContinue.compareDocumentPosition(triggerGroup) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

        await goToTome();
        const tomeContinue = screen.getByRole("button", { name: "Continue" });
        const selectTomeButton = screen.getByRole("button", { name: "Select test tome" });
        expect(tomeContinue).toBeDisabled();
        expect(tomeContinue.compareDocumentPosition(selectTomeButton) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

        fireEvent.click(selectTomeButton);
        expect(tomeContinue).toBeEnabled();
        fireEvent.click(tomeContinue);
        await screen.findByRole("heading", { name: "Confirm scheduled quest" });

        const submitButton = screen.getByRole("button", { name: "Create scheduled quest" });
        const triggerHeading = screen.getByRole("heading", { name: "Trigger" });
        expect(submitButton.compareDocumentPosition(triggerHeading) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    });

    it.each([
        { choice: "Schedule", runOnSchedule: "0 * * * *", newHost: false, newBeacon: false },
        { choice: "New host", runOnSchedule: "", newHost: true, newBeacon: false },
        { choice: "New beacon", runOnSchedule: "", newHost: false, newBeacon: true },
    ])("saves the $choice trigger after confirmation", async ({ choice, runOnSchedule, newHost, newBeacon }) => {
        openTrigger();
        if (choice !== "Schedule") fireEvent.click(screen.getByRole("radio", { name: choice }));

        await goToConfirm();
        expect(screen.getByText(choice, { selector: "dd" })).toBeInTheDocument();
        expect(screen.getByText("All hosts", { selector: "dd" })).toBeInTheDocument();
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
        expect(createScheduledTask.mock.calls[0][0].variables.input.scheduledHostIDs).toBeUndefined();
    });

    it("scopes the scheduled task to a host when opened with scheduledHost", async () => {
        openTrigger({ id: "host-42", name: "web-prod-01" });
        await goToConfirm();

        expect(screen.getByText("web-prod-01", { selector: "dd" })).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "Create scheduled quest" }));

        await waitFor(() => expect(createScheduledTask).toHaveBeenCalledWith({
            variables: {
                input: expect.objectContaining({
                    scheduledHostIDs: ["host-42"],
                }),
            },
        }));
    });

    it("keeps the schedule field on the schedule trigger only and marks required fields", async () => {
        openTrigger();
        const cronInput = screen.getByRole("textbox", { name: "Schedule (cron)" });
        expect(cronInput).toBeRequired();
        fireEvent.click(screen.getByRole("radio", { name: "New host" }));
        expect(screen.queryByRole("textbox", { name: "Schedule (cron)" })).not.toBeInTheDocument();
        expect(screen.getByText("This quest runs only when a new host checks in for the first time.")).toBeInTheDocument();
    });

    it("rejects out-of-range cron minutes and day-of-week 7 (rejected by robfig/cron/v3)", async () => {
        openTrigger();
        const cronInput = screen.getByRole("textbox", { name: "Schedule (cron)" });

        fireEvent.change(cronInput, { target: { value: "99 * * * *" } });
        fireEvent.click(screen.getByRole("button", { name: "Continue" }));
        expect(await screen.findByText("Enter a valid five-field cron schedule, such as 0 * * * *.")).toBeInTheDocument();

        fireEvent.change(cronInput, { target: { value: "0 0 * * 7" } });
        fireEvent.click(screen.getByRole("button", { name: "Continue" }));
        expect(await screen.findByText("Enter a valid five-field cron schedule, such as 0 * * * *.")).toBeInTheDocument();
        expect(screen.queryByRole("heading", { name: "Select a tome" })).not.toBeInTheDocument();
        expect(createScheduledTask).not.toHaveBeenCalled();
    });

    it("accepts stepped ranges and weekday/month names supported by robfig/cron/v3", async () => {
        openTrigger();
        const cronInput = screen.getByRole("textbox", { name: "Schedule (cron)" });
        fireEvent.change(cronInput, { target: { value: "0-30/5 9-17/2 * JAN-DEC MON-FRI" } });
        await goToConfirm();

        expect(screen.getByText("0-30/5 9-17/2 * JAN-DEC MON-FRI", { selector: "dd" })).toBeInTheDocument();
    });

    it("disables Continue on Tome step when required parameters are blank or whitespace-only", async () => {
        openTrigger();
        await goToTome();

        const continueButton = screen.getByRole("button", { name: "Continue" });
        expect(continueButton).toBeDisabled();

        fireEvent.click(screen.getByRole("button", { name: "Select parameterized tome" }));
        expect(continueButton).toBeDisabled();

        const paramInput = screen.getByLabelText(/Target/);
        fireEvent.change(paramInput, { target: { value: "   " } });
        expect(continueButton).toBeDisabled();

        fireEvent.change(paramInput, { target: { value: "10.0.0.1" } });
        expect(continueButton).toBeEnabled();
    });

    it("saves an edited quest name from Confirm", async () => {
        openTrigger();
        await goToConfirm();
        const nameInput = screen.getByRole("textbox", { name: "Quest name" });
        expect(nameInput).toBeRequired();
        fireEvent.change(nameInput, { target: { value: "  Scheduled scan  " } });
        fireEvent.click(screen.getByRole("button", { name: "Create scheduled quest" }));

        await waitFor(() => expect(createScheduledTask).toHaveBeenCalledWith({
            variables: { input: expect.objectContaining({ name: "Scheduled scan" }) },
        }));
    });

    it("rejects a blank quest name on Confirm", async () => {
        openTrigger();
        await goToConfirm();
        fireEvent.change(screen.getByRole("textbox", { name: "Quest name" }), { target: { value: "   " } });
        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "Create scheduled quest" }));
        });

        expect(screen.getByText("Quest name is required")).toBeInTheDocument();
        expect(createScheduledTask).not.toHaveBeenCalled();
    });

    it("guards against form submission via Enter while loading", async () => {
        const { rerender } = render(<CreateScheduledQuestModal isOpen setOpen={vi.fn()} />);
        await goToConfirm();

        mutationLoading = true;
        rerender(<CreateScheduledQuestModal isOpen setOpen={vi.fn()} />);

        const form = screen.getByRole("textbox", { name: "Quest name" }).closest("form")!;
        await act(async () => {
            fireEvent.submit(form);
        });

        expect(createScheduledTask).not.toHaveBeenCalled();
    });
});
