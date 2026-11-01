import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
    SweepConfirmModal,
    sweepModeForOverwrite,
} from "../SweepConfirmModal";

describe("sweepModeForOverwrite", () => {
    it("maps an unchecked box to fill-empty", () => {
        expect(sweepModeForOverwrite(false)).toBe("fill-empty");
    });

    it("maps a checked box to overwrite", () => {
        expect(sweepModeForOverwrite(true)).toBe("overwrite");
    });
});

describe("SweepConfirmModal", () => {
    it("confirms fill-empty when the checkbox is not filled in", async () => {
        const user = userEvent.setup();
        const onConfirm = vi.fn();
        render(
            <SweepConfirmModal
                isOpen
                setOpen={() => {}}
                sweeping={false}
                onConfirm={onConfirm}
            />
        );

        await user.click(screen.getByTestId("sweep-confirm"));
        expect(onConfirm).toHaveBeenCalledWith("fill-empty");
    });

    it("confirms overwrite when Overwrite Existing Tags is checked", async () => {
        const user = userEvent.setup();
        const onConfirm = vi.fn();
        render(
            <SweepConfirmModal
                isOpen
                setOpen={() => {}}
                sweeping={false}
                onConfirm={onConfirm}
            />
        );

        await user.click(screen.getByTestId("sweep-overwrite-checkbox"));
        await user.click(screen.getByTestId("sweep-confirm"));
        expect(onConfirm).toHaveBeenCalledWith("overwrite");
    });
});
