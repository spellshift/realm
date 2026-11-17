import { render, screen, fireEvent } from "@testing-library/react";
import ShellStatusBar from "../ShellStatusBar";
import ShellHeader from "../ShellHeader";
import ShellSidebar from "../ShellSidebar";
import { ConnectionStatus } from "../../../../lib/browser-adapter";
import React from "react";
import { expect, describe, it, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import { ChakraProvider } from "@chakra-ui/react";
import { AuthorizationContext } from "../../../../context/AuthorizationContext";
import { GET_NOTIFICATIONS } from "../../../../lib/notifications";
import "@testing-library/jest-dom";

// Mock NotificationBell to avoid requiring ApolloProvider in ShellStatusBar tests
vi.mock("../../../../components/notifications/NotificationBell", () => ({
    default: () => <div data-testid="notification-bell" />,
}));

describe("ShellStatusBar", () => {
    const defaultProps = {
        portalId: null,
        timeUntilCallback: "",
        isMissedCallback: false,
        connectionStatus: "connected" as ConnectionStatus,
        connectionMessage: "",
    };

    it("does not render connected icon when connectionStatus is connected", () => {
        const { container } = render(<ShellStatusBar {...defaultProps} connectionStatus="connected" />);
        expect(container.querySelector(".text-green-500")).not.toBeInTheDocument();
        expect(screen.queryByTestId("status-bar-reconnect-indicator")).not.toBeInTheDocument();
    });

    it("renders disconnected status icon only for disconnection", () => {
        const { container } = render(<ShellStatusBar {...defaultProps} connectionStatus="disconnected" />);
        const disconnectedIcon = container.querySelector(".text-red-500");
        expect(disconnectedIcon).toBeInTheDocument();
        expect(screen.getByTestId("status-bar-reconnect-indicator")).toBeInTheDocument();
    });

    it("renders reconnecting status icon and orders bottom-right elements as reconnect -> async -> next", () => {
        const { container } = render(
            <ShellStatusBar
                {...defaultProps}
                connectionStatus="reconnecting"
                timeUntilCallback="Next 5s"
            />
        );
        const reconnectingIcon = container.querySelector(".text-yellow-500.animate-spin");
        expect(reconnectingIcon).toBeInTheDocument();

        const reconnectEl = screen.getByTestId("status-bar-reconnect-indicator");
        const asyncEl = screen.getByTestId("status-bar-async-button");
        const nextEl = screen.getByTestId("status-bar-next-callback");
        expect(
            reconnectEl.compareDocumentPosition(asyncEl) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
        expect(
            asyncEl.compareDocumentPosition(nextEl) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
    });

    it("displays portal active message when portalId is present without showing portal ID, and opens Close Portal warning dialog when clicked", () => {
        const onClosePortal = vi.fn();
        const onToggleScratchpad = vi.fn();
        render(
            <ShellStatusBar
                {...defaultProps}
                portalId={123}
                onClosePortal={onClosePortal}
                sessionSettingsOpen={true}
                onToggleSessionSettings={vi.fn()}
                onToggleComposer={vi.fn()}
                composerOpen={true}
                onToggleScratchpad={onToggleScratchpad}
                scratchpadOpen={false}
            />
        );
        expect(screen.getByText("Portal Active")).toBeInTheDocument();
        expect(screen.queryByText("(ID: 123)")).not.toBeInTheDocument();
        const bellEl = screen.getByTestId("notification-bell");
        const settingsEl = screen.getByTestId("session-settings-toggle");
        const composerEl = screen.getByTestId("status-bar-composer-toggle");
        const scratchpadEl = screen.getByTestId("status-bar-scratchpad-toggle");
        expect(bellEl).toBeInTheDocument();
        expect(settingsEl).toBeInTheDocument();
        expect(settingsEl).toHaveClass("text-white", "bg-purple-600");
        expect(composerEl).toBeInTheDocument();
        expect(composerEl).toHaveClass("text-white", "bg-purple-600");
        expect(scratchpadEl).toBeInTheDocument();

        // Verify left-toolbar DOM order: Notification Bell -> Settings -> Composer -> Scratchpad
        expect(bellEl.compareDocumentPosition(settingsEl) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
        expect(settingsEl.compareDocumentPosition(composerEl) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
        expect(composerEl.compareDocumentPosition(scratchpadEl) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

        fireEvent.click(scratchpadEl);
        expect(onToggleScratchpad).toHaveBeenCalledTimes(1);

        fireEvent.click(screen.getByTestId("status-bar-portal-active-button"));
        const closeDialog = screen.getByTestId("close-portal-dialog");
        expect(closeDialog).toBeInTheDocument();
        expect(closeDialog).toHaveTextContent(/close all associated pivots/i);

        fireEvent.click(screen.getByRole("button", { name: "Close Portal" }));
        expect(onClosePortal).toHaveBeenCalledTimes(1);
    });

    it("displays async message when portalId is null and opens Open Portal dialog when clicked", () => {
        const onNewPortal = vi.fn();
        render(<ShellStatusBar {...defaultProps} portalId={null} onNewPortal={onNewPortal} />);
        expect(screen.getByText("async")).toBeInTheDocument();
        expect(screen.getByTestId("session-settings-toggle")).toBeInTheDocument();

        fireEvent.click(screen.getByTestId("status-bar-async-button"));
        const openDialog = screen.getByTestId("open-portal-dialog");
        expect(openDialog).toBeInTheDocument();
        expect(openDialog).toHaveTextContent(
            "Opens an interactive portal on this beacon, which works similar to a SOCKS5 proxy. This establishes a real-time stream between the agent and server for low-latency execution, interactive PTY, or pivoting operations."
        );

        fireEvent.click(screen.getByRole("button", { name: "Open Portal" }));
        expect(onNewPortal).toHaveBeenCalledTimes(1);
    });

    it("displays countdown callback timer and shows missed callbacks in callback stats popover on hover", () => {
        render(
            <ShellStatusBar
                {...defaultProps}
                timeUntilCallback="Next 5min"
                callbackStats={{
                    intervalSeconds: 5,
                    formattedInterval: "5s",
                    sampleCount: 100,
                    expectedCount: 103,
                    onTimeCount: 98,
                    lateCount: 2,
                    missedCount: 3,
                    onTimePercentage: 95,
                    avgLatencyMs: 85,
                    lastSeenAt: "2025-01-01T12:00:00Z",
                    nextSeenAt: "2025-01-01T12:00:05Z",
                }}
            />
        );
        expect(screen.getByText("Next 5min")).toBeInTheDocument();
        expect(screen.queryByText("Next callback:")).not.toBeInTheDocument();
        expect(screen.queryByTestId("status-bar-callback-popover")).not.toBeInTheDocument();

        const nextCallbackEl = screen.getByTestId("status-bar-next-callback");
        fireEvent.mouseEnter(nextCallbackEl);

        const popover = screen.getByTestId("status-bar-callback-popover");
        expect(popover).toBeInTheDocument();
        expect(screen.getByTestId("callback-stats-interval")).toHaveTextContent("5s");
        expect(screen.getByTestId("callback-stats-ontime")).toHaveTextContent("98 / 103 (95%)");
        expect(screen.getByTestId("callback-stats-missed")).toHaveTextContent("3");

        fireEvent.mouseLeave(nextCallbackEl);
        expect(screen.queryByTestId("status-bar-callback-popover")).not.toBeInTheDocument();
    });

    it("highlights missed callback countdown", () => {
        render(<ShellStatusBar {...defaultProps} timeUntilCallback="Missing 1hr ago" isMissedCallback={true} />);
        const timer = screen.getByText("Missing 1hr ago");
        expect(timer).toHaveClass("text-red-500 font-bold");
    });

    it("maintains consistent callback time highlight width across countdown states", () => {
        const { rerender } = render(
            <ShellStatusBar {...defaultProps} timeUntilCallback="Next now" />
        );
        const nextCallbackEl = screen.getByTestId("status-bar-next-callback");
        expect(nextCallbackEl).toHaveTextContent("Next now");
        expect(nextCallbackEl).toHaveClass(
            "inline-grid",
            "place-items-center",
            "min-w-[7rem]",
            "whitespace-nowrap",
            "after:content-['Missing_10min_ago']"
        );
        const classesWhenNow = nextCallbackEl.className;

        rerender(
            <ShellStatusBar {...defaultProps} timeUntilCallback="Next 5s" />
        );
        expect(nextCallbackEl).toHaveTextContent("Next 5s");
        expect(nextCallbackEl.className).toBe(classesWhenNow);
    });

    it("displays terminated status when closedAt is present", () => {
        render(<ShellStatusBar {...defaultProps} closedAt="2024-01-01T00:00:00Z" />);
        expect(screen.getByText(/Terminated/i)).toBeInTheDocument();
    });

    it("does not render a queue indicator at the bottom of the terminal even when tasks are queued or running", () => {
        render(
            <ShellStatusBar
                {...defaultProps}
                runningCount={1}
                queuedCount={2}
                runningTasks={[
                    {
                        id: "t-run",
                        createdAt: "2025-01-01T12:00:00Z",
                        claimedAt: "2025-01-01T12:00:02Z",
                        input: 'sys.shell("uname -a")',
                    },
                ]}
                queuedTasks={[
                    {
                        id: "t-q1",
                        createdAt: "2025-01-01T12:01:00Z",
                        input: 'file.list("/etc")',
                    },
                ]}
            />
        );
        expect(screen.queryByTestId("status-bar-queue-button")).not.toBeInTheDocument();
        expect(screen.queryByTestId("status-bar-queue-popover")).not.toBeInTheDocument();
        expect(screen.queryByText(/executing/i)).not.toBeInTheDocument();
        expect(screen.queryByText(/queued/i)).not.toBeInTheDocument();
    });

    it("renders sidebar toggle in top bar to the left of Home icon with LogOut when hidden by default and mirrored LogIn when expanded, and toggles icon-only ShellSidebar", () => {
        const StatefulShellSidebarWrapper = () => {
            const [sidebarOpen, setSidebarOpen] = React.useState(false);
            return (
                <BrowserRouter>
                    <AuthorizationContext.Provider
                        value={{
                            data: {
                                me: {
                                    id: "u1",
                                    name: "admin-op",
                                    photoURL: "",
                                    isActivated: true,
                                    isAdmin: true,
                                },
                            },
                            isLoading: false,
                            error: undefined,
                        }}
                    >
                        <ShellSidebar isOpen={sidebarOpen} />
                        <ShellHeader
                            shellData={null}
                            portalId={null}
                            onExport={vi.fn()}
                            onNewPortal={vi.fn()}
                            onSshConnect={vi.fn()}
                            onPtyOpen={vi.fn()}
                            onSendCtrlC={vi.fn()}
                            onSendCtrlR={vi.fn()}
                            onClosePortal={vi.fn()}
                            sidebarOpen={sidebarOpen}
                            onToggleSidebar={() => setSidebarOpen((prev) => !prev)}
                        />
                        <ShellStatusBar {...defaultProps} />
                    </AuthorizationContext.Provider>
                </BrowserRouter>
            );
        };

        render(<StatefulShellSidebarWrapper />);

        // Sidebar is hidden by default and toggle shows LogOut icon
        expect(screen.queryByTestId("shell-sidebar")).not.toBeInTheDocument();
        const toggleBtn = screen.getByTestId("status-bar-sidebar-toggle");
        expect(toggleBtn).toHaveAttribute("aria-expanded", "false");
        expect(screen.getByTestId("sidebar-toggle-logout-icon")).toBeInTheDocument();
        expect(screen.queryByTestId("sidebar-toggle-login-icon")).not.toBeInTheDocument();

        // Click to expand sidebar from the left
        fireEvent.click(toggleBtn);
        const sidebar = screen.getByTestId("shell-sidebar");
        expect(sidebar).toBeInTheDocument();
        expect(sidebar).toHaveClass("w-12");
        expect(sidebar).toHaveClass("z-20");

        // Status bar stacks above the sidebar so notification popover is not hidden behind the sidebar
        const statusBar = document.querySelector('[data-tour="shell-status-bar"]');
        expect(statusBar).toHaveClass("relative", "z-30");
        expect(screen.getByTestId("notification-bell")).toBeInTheDocument();

        expect(toggleBtn).toHaveAttribute("aria-expanded", "true");
        const loginIcon = screen.getByTestId("sidebar-toggle-login-icon");
        expect(loginIcon).toBeInTheDocument();
        expect(loginIcon).toHaveClass("-scale-x-100");
        expect(screen.queryByTestId("sidebar-toggle-logout-icon")).not.toBeInTheDocument();

        // Verify navigation items match the rest of the UI
        expect(screen.getByLabelText("Dashboard")).toHaveAttribute("href", "/dashboard");
        expect(screen.getByLabelText("Hosts")).toHaveAttribute("href", "/hosts");
        expect(screen.getByLabelText("Quests")).toHaveAttribute("href", "/quests");
        expect(screen.getByLabelText("Tomes")).toHaveAttribute("href", "/tomes");
        expect(screen.getByLabelText("Assets")).toHaveAttribute("href", "/assets");
        expect(screen.getByLabelText("Admin")).toHaveAttribute("href", "/admin");
        expect(screen.getByLabelText("Documentation")).toHaveAttribute("href", "https://docs.realm.pub/");
        expect(screen.getByLabelText("API Playground")).toHaveAttribute("href", "/playground");
        expect(screen.getByLabelText("Profile")).toHaveAttribute("href", "/profile");
        expect(screen.getByLabelText("Realm")).toHaveAttribute("href", "/");

        // Click again to collapse sidebar
        fireEvent.click(toggleBtn);
        expect(screen.queryByTestId("shell-sidebar")).not.toBeInTheDocument();
        expect(screen.getByTestId("sidebar-toggle-logout-icon")).toBeInTheDocument();
    });

    it("renders NotificationBell popover positioner above the open sidebar so notifications can be viewed", async () => {
        const { default: ActualNotificationBell } = await vi.importActual<{
            default: React.ComponentType<any>;
        }>("../../../../components/notifications/NotificationBell");

        const mocks = [
            {
                request: {
                    query: GET_NOTIFICATIONS,
                    variables: { first: 100 },
                },
                result: {
                    data: {
                        me: {
                            id: "u1",
                            notifications: {
                                totalCount: 0,
                                pageInfo: {
                                    hasNextPage: false,
                                    hasPreviousPage: false,
                                    startCursor: null,
                                    endCursor: null,
                                },
                                edges: [],
                            },
                        },
                    },
                },
            },
        ];

        const { container } = render(
            <BrowserRouter>
                <ChakraProvider>
                    <MockedProvider mocks={mocks} addTypename={false}>
                        <AuthorizationContext.Provider
                            value={{
                                data: {
                                    me: {
                                        id: "u1",
                                        name: "admin-op",
                                        photoURL: "",
                                        isActivated: true,
                                        isAdmin: true,
                                    },
                                },
                                isLoading: false,
                                error: undefined,
                            }}
                        >
                            <ShellSidebar isOpen={true} />
                            <div className="relative z-30">
                                <ActualNotificationBell
                                    placement="top-start"
                                    buttonClassName="relative inline-flex items-center justify-center p-1 rounded-md"
                                    iconClassName="w-4 h-4"
                                />
                            </div>
                        </AuthorizationContext.Provider>
                    </MockedProvider>
                </ChakraProvider>
            </BrowserRouter>
        );

        expect(screen.getByTestId("shell-sidebar")).toBeInTheDocument();
        const bellButton = screen.getByRole("button", { name: "Notifications" });
        fireEvent.click(bellButton);

        expect(screen.getByText("Notifications")).toBeInTheDocument();
        const popperEl = container.querySelector(".chakra-popover__popper") as HTMLElement;
        expect(popperEl).toBeInTheDocument();
        expect(popperEl.style.zIndex).toBe("1500");
    });
});
