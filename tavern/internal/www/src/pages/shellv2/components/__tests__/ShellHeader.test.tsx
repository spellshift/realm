import { render, screen } from "@testing-library/react";
import ShellHeader from "../ShellHeader";
import React from "react";
import { expect, describe, it, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import "@testing-library/jest-dom";

// Mock Tooltip because it often causes issues in tests
vi.mock("@chakra-ui/react", async () => {
    const actual = await vi.importActual("@chakra-ui/react");
    return {
        ...actual,
        Tooltip: ({ children, label }: any) => <div title={typeof label === 'string' ? label : "tooltip"}>{children}</div>,
    };
});

describe("ShellHeader", () => {
    const mockShellData = {
        node: {
            beacon: {
                name: "test-beacon",
                principal: "root",
                agentIdentifier: "agent-1",
                interval: 60,
                transport: "HTTP",
                host: {
                    id: "host-1",
                    name: "test-host",
                    primaryIP: "192.168.1.1",
                    externalIP: "8.8.8.8",
                    platform: "linux",
                    tags: { edges: [] }
                }
            }
        }
    };

    it("renders beacon name and host name", () => {
        render(
            <BrowserRouter>
                <ShellHeader shellData={mockShellData} />
            </BrowserRouter>
        );
        expect(screen.getByText("test-beacon")).toBeInTheDocument();
        expect(screen.getByText("test-host")).toBeInTheDocument();
    });

    it("renders principal badge when present", () => {
        render(
            <BrowserRouter>
                <ShellHeader shellData={mockShellData} />
            </BrowserRouter>
        );
        expect(screen.getByText("root")).toBeInTheDocument();
    });

    it("does not render principal badge when absent", () => {
        const dataWithoutPrincipal = {
            node: {
                ...mockShellData.node,
                beacon: {
                    ...mockShellData.node.beacon,
                    principal: null
                }
            }
        };
        render(
            <BrowserRouter>
                <ShellHeader shellData={dataWithoutPrincipal} />
            </BrowserRouter>
        );
        expect(screen.queryByText("root")).not.toBeInTheDocument();
    });
});
