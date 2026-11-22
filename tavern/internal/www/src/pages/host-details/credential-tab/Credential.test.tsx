import { ChakraProvider } from "@chakra-ui/react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Credential from "./Credential";

const writeText = vi.fn().mockResolvedValue(undefined);

describe("Credential", () => {
    beforeEach(() => {
        writeText.mockClear();
    });

    it("copies the full secret while the display is masked", async () => {
        const user = userEvent.setup();
        Object.defineProperty(navigator, "clipboard", {
            configurable: true,
            value: { writeText },
        });
        const secret = "a-secret-value-longer-than-the-visible-cell";
        const { container } = render(
            <ChakraProvider>
                <div className="flex"><Credential secret={secret} /></div>
            </ChakraProvider>
        );

        expect(container).not.toHaveTextContent(secret);
        await user.click(screen.getByRole("button", { name: "Copy secret" }));
        expect(writeText).toHaveBeenCalledWith(secret);
        expect(screen.getByRole("button", { name: "Show secret" })).toBeInTheDocument();
    });
});
