import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import HostHeader from "./HostHeader";

const openModal = vi.fn();
const setSearchParams = vi.fn();

vi.mock("react-router-dom", () => ({ useSearchParams: () => [new URLSearchParams(), setSearchParams] }));
vi.mock("../../../context/HostContext", () => ({
    useHost: () => ({ data: { id: "host-1", name: "Test host" } }),
}));
vi.mock("../../../context/CreateQuestModalContext", () => ({
    useCreateQuestModal: () => ({ openModal }),
}));
vi.mock("../../../components/Breadcrumbs", () => ({ default: () => <div>Breadcrumbs</div> }));
vi.mock("./HostSubscriptionButton", () => ({ default: () => <button>Subscribe</button> }));

describe("HostHeader", () => {
    beforeEach(() => {
        openModal.mockClear();
        setSearchParams.mockClear();
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        };
    });

    it("opens the immediate quest form for this host and returns to its tasks tab on completion", async () => {
        render(<HostHeader />);

        fireEvent.click(screen.getByRole("button", { name: "Create quest" }));
        fireEvent.click(await screen.findByRole("menuitem", { name: "Create quest" }));

        expect(openModal).toHaveBeenCalledWith({
            initialFormData: {
                initialFilters: {
                    beaconFields: [{
                        id: "host-1",
                        name: "Test host",
                        value: "host-1",
                        label: "Test host",
                        kind: "host",
                    }],
                },
            },
            onComplete: expect.any(Function),
        });
        expect(openModal.mock.calls[0][0].navigateToQuest).toBeUndefined();

        openModal.mock.calls[0][0].onComplete("quest-1");
        expect(setSearchParams).toHaveBeenCalledWith({ tab: "tasks" });
    });
});
