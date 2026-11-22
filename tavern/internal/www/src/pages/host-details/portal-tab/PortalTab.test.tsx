import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";
import "@testing-library/jest-dom/vitest";
import { PortalsHeader } from "./PortalTab";

it("offers a SOCKS5 download for every supported platform", async () => {
    const user = userEvent.setup();
    render(<PortalsHeader />);

    await user.click(screen.getByRole("button", { name: /Download socks5/i }));

    for (const [platform, asset] of [
        ["Linux", "linux/socks5"],
        ["macOS", "macos/socks5"],
        ["Windows", "windows/socks5.exe"],
        ["FreeBSD", "bsd/socks5"],
    ]) {
        expect(screen.getByRole("menuitem", { name: platform })).toHaveAttribute(
            "href",
            `/assets/download/${asset}`
        );
    }
});
