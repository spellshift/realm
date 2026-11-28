import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { CreateQuestDropdown } from "../CreateQuestDropdown";

const openModal = vi.fn();

vi.mock("../../../context/CreateQuestModalContext", () => ({
    useCreateQuestModal: () => ({ openModal }),
}));

describe("CreateQuestDropdown", () => {
    beforeEach(() => {
        openModal.mockClear();
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        };
    });

    it("opens the ordinary quest form from the Create quest menu action", async () => {
        render(<CreateQuestDropdown />);

        const trigger = screen.getByRole("button", { name: "Create quest" });
        expect(trigger.querySelector("svg.lucide-file-terminal")).toBeInTheDocument();
        fireEvent.click(trigger);
        fireEvent.click(await screen.findByRole("menuitem", { name: "Create quest" }));

        expect(openModal).toHaveBeenCalledWith({ navigateToQuest: true });
    });

    it("opens the scheduled quest form from the dropdown", async () => {
        render(<CreateQuestDropdown />);

        fireEvent.click(screen.getByRole("button", { name: "Create quest" }));
        fireEvent.click(await screen.findByRole("menuitem", { name: "Schedule quest" }));

        expect(openModal).toHaveBeenCalledWith({ mode: "scheduled" });
    });

    it("forwards scheduledHost when scheduling from a host-scoped dropdown", async () => {
        render(<CreateQuestDropdown scheduledHost={{ id: "host-9", name: "db-01" }} />);

        fireEvent.click(screen.getByRole("button", { name: "Create quest" }));
        fireEvent.click(await screen.findByRole("menuitem", { name: "Schedule quest" }));

        expect(openModal).toHaveBeenCalledWith({
            mode: "scheduled",
            scheduledHost: { id: "host-9", name: "db-01" },
        });
    });
});
