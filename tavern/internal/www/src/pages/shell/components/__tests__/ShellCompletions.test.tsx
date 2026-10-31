import { render, screen, fireEvent, act } from "@testing-library/react";
import React from "react";
import { expect, describe, it, vi, beforeAll } from "vitest";
import ShellCompletions from "../ShellCompletions";
import ShellTerminal from "../ShellTerminal";
import { registerUserSymbolsFromCode } from "../../utils/eldritchLsp";
import "@testing-library/jest-dom";

// Polyfill ResizeObserver for jsdom
beforeAll(() => {
    if (typeof globalThis.ResizeObserver === "undefined") {
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        } as unknown as typeof ResizeObserver;
    }
});

describe("ShellCompletions", () => {
    it("returns null when show is false", () => {
        render(
            <ShellCompletions
                completions={["agent", "assets", "file"]}
                show={false}
                pos={{ x: 10, y: 20 }}
                index={0}
            />
        );
        expect(screen.queryByTestId("shell-completions")).not.toBeInTheDocument();
    });

    it("positions below the cursor line when there is sufficient room below", () => {
        const { container } = render(
            <div style={{ height: "600px", width: "800px" }}>
                <ShellCompletions
                    completions={["agent", "assets", "file"]}
                    show={true}
                    pos={{ x: 20, y: 50 }}
                    index={0}
                />
            </div>
        );
        const parent = container.firstChild as HTMLElement;
        Object.defineProperty(parent, "clientHeight", { value: 600, configurable: true });
        Object.defineProperty(parent, "clientWidth", { value: 800, configurable: true });

        const completionsEl = screen.getByTestId("shell-completions");
        expect(completionsEl).toBeInTheDocument();
        expect(completionsEl.style.top).toBe("50px");
        expect(completionsEl.style.left).toBe("20px");
    });

    it("shifts up so that it does not collide with the Eldritch composer or bottom drawers", () => {
        const { container } = render(
            <div style={{ height: "300px", width: "600px" }}>
                <ShellCompletions
                    completions={["agent", "assets", "file"]}
                    show={true}
                    pos={{ x: 20, y: 260 }}
                    index={0}
                    fontSize={13.5}
                />
            </div>
        );
        const parent = container.firstChild as HTMLElement;
        Object.defineProperty(parent, "clientHeight", { value: 300, configurable: true });
        Object.defineProperty(parent, "clientWidth", { value: 600, configurable: true });

        // Trigger resize inside act to force layout recalculation with mocked parent dimensions
        act(() => {
            window.dispatchEvent(new Event("resize"));
        });

        const completionsEl = screen.getByTestId("shell-completions");
        expect(completionsEl).toBeInTheDocument();

        // pos.y is 260 in a 300px container. Without shift, 260 + height > 300.
        // It must be shifted up above the prompt line.
        const topPx = parseInt(completionsEl.style.top, 10);
        expect(topPx).toBeLessThan(260);
        expect(topPx).toBeGreaterThanOrEqual(4);
    });

    it("shifts left when near the right boundary to avoid colliding with side drawers", () => {
        const { container } = render(
            <div style={{ height: "600px", width: "400px" }}>
                <ShellCompletions
                    completions={["agent", "assets", "file"]}
                    show={true}
                    pos={{ x: 380, y: 50 }}
                    index={0}
                />
            </div>
        );
        const parent = container.firstChild as HTMLElement;
        Object.defineProperty(parent, "clientHeight", { value: 600, configurable: true });
        Object.defineProperty(parent, "clientWidth", { value: 400, configurable: true });

        act(() => {
            window.dispatchEvent(new Event("resize"));
        });

        const completionsEl = screen.getByTestId("shell-completions");
        const leftPx = parseInt(completionsEl.style.left, 10);
        expect(leftPx).toBeLessThan(380);
        expect(leftPx).toBeGreaterThanOrEqual(4);
    });

    it("calls onCompletionSelect when a completion item is clicked", () => {
        const onSelect = vi.fn();
        render(
            <ShellCompletions
                completions={["agent.config", "agent.transport"]}
                show={true}
                pos={{ x: 20, y: 50 }}
                index={0}
                onCompletionSelect={onSelect}
            />
        );
        const item = screen.getByText("agent.config");
        fireEvent.mouseDown(item);
        expect(onSelect).toHaveBeenCalledWith("agent.config");
    });

    it("displays doc info icon and shows tooltip on hover", async () => {
        render(
            <ShellCompletions
                completions={["agent.config", "custom_cmd"]}
                show={true}
                pos={{ x: 20, y: 50 }}
                index={0}
            />
        );
        // agent.config has documentation in eldritch-docs.json, custom_cmd does not
        const agentItem = screen.getByText("agent.config").closest("li")!;
        expect(agentItem.querySelector("svg")).toBeInTheDocument();

        fireEvent.mouseEnter(agentItem);
        // DocTooltip becomes visible with the signature and description
        expect(await screen.findByText(/Returns the current configuration of the agent/i)).toBeInTheDocument();
        expect(screen.getAllByText(/agent\.config/i).length).toBeGreaterThan(1);

        fireEvent.mouseLeave(agentItem);
    });

    it("displays doc info icon and shows tooltip on hover for user-defined functions and variables", async () => {
        registerUserSymbolsFromCode('def custom_tool(target, timeout=30):\n    """Execute custom target operation."""\n    return True\nTARGET_IP = "192.168.1.100"');
        render(
            <ShellCompletions
                completions={["custom_tool", "TARGET_IP"]}
                show={true}
                pos={{ x: 20, y: 50 }}
                index={0}
            />
        );
        const toolItem = screen.getByText("custom_tool").closest("li")!;
        expect(toolItem.querySelector("svg")).toBeInTheDocument();

        fireEvent.mouseEnter(toolItem);
        expect(await screen.findByText(/Execute custom target operation/i)).toBeInTheDocument();
        fireEvent.mouseLeave(toolItem);

        const targetItem = screen.getByText("TARGET_IP").closest("li")!;
        expect(targetItem.querySelector("svg")).toBeInTheDocument();
        fireEvent.mouseEnter(targetItem);
        expect(await screen.findByText(/User-defined variable/i)).toBeInTheDocument();
        fireEvent.mouseLeave(targetItem);
    });

    it("renders inside ShellTerminal and shifts up when prompt is near the bottom", () => {
        const termRef = React.createRef<HTMLDivElement>();
        const { container } = render(
            <div style={{ height: "300px", width: "800px" }}>
                <ShellTerminal
                    termRef={termRef}
                    fontSize={14}
                    isConnecting={false}
                    completions={["agent.config", "agent.transport", "sys.info"]}
                    showCompletions={true}
                    completionPos={{ x: 25, y: 265 }}
                    completionIndex={0}
                    onMouseMove={vi.fn()}
                    tooltipState={{ visible: false, x: 0, y: 0, signature: "", description: "" }}
                    handleCompletionSelect={vi.fn()}
                />
            </div>
        );
        const terminalContainer = container.querySelector('[data-tour="shell-terminal"]') as HTMLElement;
        expect(terminalContainer).toBeInTheDocument();
        Object.defineProperty(terminalContainer, "clientHeight", { value: 300, configurable: true });
        Object.defineProperty(terminalContainer, "clientWidth", { value: 800, configurable: true });

        act(() => {
            window.dispatchEvent(new Event("resize"));
        });

        const completionsEl = screen.getByTestId("shell-completions");
        expect(completionsEl).toBeInTheDocument();
        const topPx = parseInt(completionsEl.style.top, 10);
        expect(topPx).toBeLessThan(265);
        expect(topPx).toBeGreaterThanOrEqual(4);
    });
});
