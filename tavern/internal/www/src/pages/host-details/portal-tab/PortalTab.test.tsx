import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterAll, beforeAll, expect, it, vi } from "vitest";
import "@testing-library/jest-dom/vitest";
import { PortalsHeader } from "./PortalTab";

beforeAll(() => {
    vi.stubGlobal("ResizeObserver", class {
        observe() {}
        unobserve() {}
        disconnect() {}
    });
});

afterAll(() => {
    vi.unstubAllGlobals();
});

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

it("supports keyboard navigation through download options", async () => {
    const user = userEvent.setup();
    render(<PortalsHeader />);

    await user.tab();
    await user.keyboard("{Enter}");

    expect(screen.getByRole("menuitem", { name: "Linux" })).toHaveAttribute("data-focus");
    await user.keyboard("{ArrowDown}");
    expect(screen.getByRole("menuitem", { name: "macOS" })).toHaveAttribute("data-focus");

    await user.keyboard("{Escape}");
    expect(screen.getByRole("button", { name: /Download socks5/i })).toHaveAttribute("aria-expanded", "false");
});
