import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import AssetsHeader from "../AssetsHeader";

describe("AssetsHeader", () => {
    it("renders Generate agent and Upload Assets buttons and triggers callbacks", async () => {
        const onGenerateAgent = vi.fn();
        const setOpen = vi.fn();
        const user = userEvent.setup();

        render(
            <MemoryRouter>
                <AssetsHeader setOpen={setOpen} onGenerateAgent={onGenerateAgent} />
            </MemoryRouter>
        );

        const generateBtn = screen.getByRole("button", { name: /generate agent/i });
        const uploadBtn = screen.getByRole("button", { name: /upload assets/i });

        expect(generateBtn).toBeInTheDocument();
        expect(uploadBtn).toBeInTheDocument();

        await user.click(generateBtn);
        expect(onGenerateAgent).toHaveBeenCalledTimes(1);

        await user.click(uploadBtn);
        expect(setOpen).toHaveBeenCalledWith(true);
    });
});
