import React from "react";
import { render, screen, fireEvent, within } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import HostsTreeDrawer, {
    HostsTreeView,
    buildHostsTreeSections,
    filterHostsTreeSections,
} from "../HostsTreeDrawer";
import {
    KnowledgeBeaconItem,
    KnowledgeHostItem,
    KnowledgeShellItem,
    ShellPivotItem,
    ShellTaskItem,
} from "../../../pages/shell/hooks/useShellData";
import { SupportedTransports } from "../../../utils/enums";

vi.mock("@chakra-ui/react", async () => {
    const actual = await vi.importActual("@chakra-ui/react");
    return {
        ...actual,
        Tooltip: ({ children, label }: any) => (
            <div title={typeof label === "string" ? label : "tooltip"}>{children}</div>
        ),
    };
});

describe("HostsTreeDrawer & HostsTreeView", () => {
    const sampleCurrentHostBeacons: KnowledgeBeaconItem[] = [
        {
            id: "b-1",
            name: "beacon-web",
            principal: "root",
            transport: SupportedTransports.GRPC,
            interval: 5,
            isOnline: true,
            hostId: "h-1",
            hostName: "web-01",
        },
    ];

    const sampleHosts: KnowledgeHostItem[] = [
        {
            id: "h-1",
            hostId: "h-1",
            name: "web-01",
            hostname: "web-01.corp",
            primaryIP: "10.0.0.10",
            ipAddress: "10.0.0.10",
            subnet: "10.0.0.0/24",
            openPorts: [22, 80],
            tags: [],
            beaconCount: 1,
            beacons: sampleCurrentHostBeacons,
            primaryBeaconId: "b-1",
            hasOnlineBeacon: true,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: true,
            sharedTags: [],
            sameSubnet: true,
        },
        {
            id: "h-2",
            hostId: "h-2",
            name: "db-01",
            hostname: "db-01.corp",
            primaryIP: "10.0.0.20",
            ipAddress: "10.0.0.20",
            subnet: "10.0.0.0/24",
            openPorts: [22, 5432],
            tags: [],
            beaconCount: 2,
            beacons: [
                {
                    id: "b-2",
                    name: "beacon-db",
                    principal: "postgres",
                    transport: SupportedTransports.GRPC,
                    interval: 10,
                    isOnline: true,
                    hostId: "h-2",
                    hostName: "db-01",
                },
                {
                    id: "b-3",
                    name: "beacon-dead",
                    principal: "root",
                    transport: SupportedTransports.HTTP1,
                    interval: 60,
                    isOnline: false,
                    hostId: "h-2",
                    hostName: "db-01",
                },
            ],
            primaryBeaconId: "b-2",
            hasOnlineBeacon: true,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
        },
        {
            id: "nh-3",
            networkHostId: "nh-3",
            name: "fileserver",
            hostname: "fileserver.corp",
            primaryIP: "10.0.0.30",
            ipAddress: "10.0.0.30",
            subnet: "10.0.0.0/24",
            openPorts: [22, 445],
            tags: [],
            beaconCount: 0,
            beacons: [],
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
            isReachable: true,
        },
        {
            id: "nh-4",
            networkHostId: "nh-4",
            name: "unreachable-ext",
            hostname: "unreachable-ext.remote",
            primaryIP: "192.168.50.9",
            ipAddress: "192.168.50.9",
            subnet: "192.168.50.0/24",
            openPorts: [22],
            tags: [],
            beaconCount: 0,
            beacons: [],
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: false,
            isReachable: false,
        },
        {
            id: "h-5",
            hostId: "h-5",
            name: "app-01",
            hostname: "app-01.corp",
            primaryIP: "10.0.0.50",
            ipAddress: "10.0.0.50",
            subnet: "10.0.0.0/24",
            openPorts: [22, 8080],
            tags: [],
            beaconCount: 1,
            beacons: [
                {
                    id: "b-5",
                    name: "beacon-app",
                    principal: "root",
                    transport: SupportedTransports.GRPC,
                    interval: 5,
                    isOnline: true,
                    hostId: "h-5",
                    hostName: "app-01",
                },
            ],
            primaryBeaconId: "b-5",
            hasOnlineBeacon: true,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
            isReachable: true,
        },
    ];

    const sampleShells: KnowledgeShellItem[] = [
        {
            id: "10",
            createdAt: "2026-01-01T12:00:00Z",
            beaconId: "b-1",
            beaconName: "beacon-web",
            beaconPrincipal: "root",
            beaconOnline: true,
            isLive: true,
            ownerId: "u-me",
            ownerName: "kcarretto",
            hostId: "h-1",
            hostName: "web-01",
            isCurrentHost: true,
            activeUsers: [
                { id: "u1", name: "alice" },
                { id: "u2", name: "bob" },
            ],
            tasks: [
                {
                    id: "t-10",
                    input: "file.list('/var/www/html')",
                    output: "index.php wp-config.php",
                },
            ],
        },
        {
            id: "20",
            createdAt: "2026-01-01T12:02:00Z",
            beaconId: "b-2",
            beaconName: "beacon-db",
            beaconPrincipal: "postgres",
            beaconOnline: true,
            isLive: true,
            ownerId: "u-3",
            ownerName: "carol",
            hostId: "h-2",
            hostName: "db-01",
            isCurrentHost: false,
            activeUsers: [{ id: "u3", name: "carol" }],
            tasks: [
                {
                    id: "t-20",
                    input: "process.list()",
                    output: "postgres-worker-15",
                },
            ],
        },
        {
            id: "21",
            createdAt: "2026-01-01T11:00:00Z",
            beaconId: "b-3",
            beaconName: "beacon-dead",
            beaconPrincipal: "root",
            beaconOnline: false,
            isLive: false,
            ownerId: "u-me",
            ownerName: "kcarretto",
            hostId: "h-2",
            hostName: "db-01",
            isCurrentHost: false,
            tasks: [
                {
                    id: "t-21",
                    input: "sys.shell('pg_dumpall')",
                    output: "CREATE ROLE audit_admin",
                },
            ],
        },
        {
            id: "50",
            createdAt: "2026-01-01T12:05:00Z",
            beaconId: "b-5",
            beaconName: "beacon-app",
            beaconPrincipal: "root",
            beaconOnline: true,
            isLive: true,
            ownerId: "u-me",
            ownerName: "kcarretto",
            hostId: "h-5",
            hostName: "app-01",
            isCurrentHost: false,
            activeUsers: [{ id: "u-me", name: "kcarretto" }],
        },
    ];

    const samplePivots: ShellPivotItem[] = [
        {
            id: "piv-pty-1",
            kind: "PTY",
            destination: "pty",
            closedAt: null,
            data: "root@web-01:~# systemctl status nginx\r\n",
            shellId: "10",
            hostId: "h-1",
        },
        {
            id: "piv-closed-pty",
            kind: "PTY",
            destination: "pty",
            closedAt: "2026-01-01T11:30:00Z",
            data: "\x1b[32muid=0(root)\x1b[0m gid=0(root)\r\n",
            shellId: "10",
            hostId: "h-1",
        },
        {
            id: "piv-ssh-2",
            kind: "SSH",
            destination: "postgres@10.0.0.20:22",
            closedAt: null,
            data: "postgres@db-01:~$ psql -l\r\n",
            shellId: "20",
            hostId: "h-2",
        },
        {
            id: "piv-closed-ssh",
            kind: "SSH",
            destination: "postgres@10.0.0.20:22",
            closedAt: "2026-01-01T11:45:00Z",
            data: "Linux db-01 6.1.0-amd64\r\n",
            shellId: "20",
            hostId: "h-2",
        },
        {
            id: "piv-closed-failed",
            kind: "SSH",
            destination: "root@10.0.0.99:22",
            closedAt: "2026-01-01T12:01:00Z",
            data: "[-] connection refused",
            shellId: "10",
        },
    ];

    it("renders flattened specific-host view by default with breadcrumb, Filters button, reachable NetworkHosts, and hides Other Shells unless filtered", () => {
        const onSelectPivot = vi.fn();
        const onClosePivot = vi.fn();
        const onOpenSshWithTarget = vi.fn();

        const outgoingSshPivot: ShellPivotItem = {
            id: "piv-out-ssh",
            kind: "SSH",
            destination: "deploy@10.0.0.40:22",
            closedAt: null,
            data: "deploy@jump:~$ id\r\n",
            shellId: "10",
            hostId: "h-1",
        };
        const pivotsWithOutgoing = [...samplePivots, outgoingSshPivot];

        const sections = buildHostsTreeSections({
            hosts: sampleHosts,
            shells: sampleShells,
            pivots: pivotsWithOutgoing,
            currentHostBeacons: sampleCurrentHostBeacons,
            currentHostId: "h-1",
            currentHostName: "web-01",
            currentHostIp: "10.0.0.10",
            currentShellId: "10",
        });

        expect(sections.currentHostNode?.name).toBe("web-01");
        expect(sections.currentHostNode?.beacons.map((b) => b.id)).toEqual(["b-1"]);
        expect(sections.currentHostNode?.closedPivots.map((p) => p.id)).toEqual(["piv-closed-pty"]);
        expect(sections.activePivotHosts.map((n) => n.name)).toEqual(["10.0.0.40", "db-01"]);
        expect(sections.hiddenOtherHosts.map((n) => n.name)).toEqual(["app-01"]);
        expect(sections.availablePivotHosts.map((n) => n.name)).toEqual(["fileserver"]);
        expect(sections.visibleHostCount).toBe(4);
        expect(sections.activePivotCount).toBe(3);

        const { rerender } = render(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={pivotsWithOutgoing}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                currentUserId="u-me"
                currentUserName="kcarretto"
                onSelectPivot={onSelectPivot}
                onClosePivot={onClosePivot}
                onOpenSshWithTarget={onOpenSshWithTarget}
            />
        );

        expect(screen.getByTestId("shell-hosts-drawer")).toBeInTheDocument();

        // Breadcrumb displays Access (purple clickable text) / web-01 and preserves top-right button order (Add Host -> Add Credential -> Close)
        const accessBreadcrumbBtn = screen.getByTestId("hosts-tree-breadcrumb-hosts");
        expect(accessBreadcrumbBtn).toHaveTextContent("Access");
        expect(screen.getByText("Access")).toHaveClass("text-purple-400");
        expect(screen.getByTestId("hosts-tree-breadcrumb-active-host")).toHaveTextContent("web-01");
        const addButton = screen.getByTestId("hosts-tree-add-button");
        const addCredentialButton = screen.getByTestId("hosts-tree-add-credential-button");
        const closeButton = screen.getByLabelText("Close access drawer");
        expect(
            addButton.compareDocumentPosition(addCredentialButton) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
        expect(
            addCredentialButton.compareDocumentPosition(closeButton) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
        expect(screen.getByTestId("hosts-tree-add-plus-icon")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-add-credential-plus-icon")).toBeInTheDocument();

        // Replaces free-text search input with Filters (0) button
        expect(screen.queryByTestId("hosts-tree-search-input")).not.toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-filters-button")).toHaveTextContent("Filters (0)");

        // No green status labels or dots
        expect(screen.queryByText("Online")).not.toBeInTheDocument();
        expect(screen.queryByText("Connected")).not.toBeInTheDocument();

        // Specific Host View is flattened (no tree branch connector) and omits current host shell row + "Other Shells" by default
        const currentSection = screen.getByTestId("hosts-tree-current-section");
        expect(currentSection.querySelector(".border-l")).toBeNull();
        expect(screen.getByTestId("hosts-tree-current-indicator")).toHaveClass("bg-purple-400");
        expect(screen.getByTestId("hosts-tree-current-monitor-icon")).toHaveClass("text-gray-400");
        expect(screen.getByTestId("hosts-tree-host-ip-curr-h-1")).toHaveTextContent("10.0.0.10");
        expect(screen.queryByText("10.0.0.0/24")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-shell-10")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-other-shells-divider")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-active-pivots-section")).not.toBeInTheDocument();
        expect(screen.queryByText("db-01")).not.toBeInTheDocument();
        expect(screen.queryByText("app-01")).not.toBeInTheDocument();
        expect(screen.queryByText("unreachable-ext")).not.toBeInTheDocument();

        // Displays current-host PTY and outgoing SSH pivot with right-aligned X close buttons; clicking X closes without selecting
        const ptyRow = screen.getByTestId("hosts-tree-pivot-piv-pty-1");
        expect(ptyRow).toHaveTextContent("PTY");
        expect(ptyRow).not.toHaveTextContent(/PTY\s*PTY/);
        expect(ptyRow).not.toHaveTextContent("#piv-pty-1");
        const closePtyBtn = screen.getByTestId("hosts-tree-close-pivot-piv-pty-1");
        const closeSshBtn = screen.getByTestId("hosts-tree-close-pivot-piv-out-ssh");
        expect(closePtyBtn).toBeEnabled();
        expect(closeSshBtn).toBeEnabled();
        fireEvent.click(closePtyBtn);
        expect(onClosePivot).toHaveBeenCalledWith(expect.objectContaining({ id: "piv-pty-1" }));
        fireEvent.click(closeSshBtn);
        expect(onClosePivot).toHaveBeenCalledWith(expect.objectContaining({ id: "piv-out-ssh" }));
        expect(onSelectPivot).not.toHaveBeenCalled();

        // Disabled state while closing
        rerender(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={pivotsWithOutgoing}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                currentUserId="u-me"
                currentUserName="kcarretto"
                onSelectPivot={onSelectPivot}
                onClosePivot={onClosePivot}
                closingPivotIds={["piv-out-ssh"]}
                onOpenSshWithTarget={onOpenSshWithTarget}
            />
        );
        expect(screen.getByTestId("hosts-tree-close-pivot-piv-out-ssh")).toBeDisabled();

        const availableHostBtn = screen.getByTestId("hosts-tree-available-host-host-nh-3");
        expect(currentSection).toContainElement(availableHostBtn);
        expect(availableHostBtn).toHaveTextContent("fileserver");
        expect(availableHostBtn).toHaveTextContent("10.0.0.30:22");
        fireEvent.click(availableHostBtn);
        expect(onOpenSshWithTarget).toHaveBeenCalledWith("10.0.0.30:22");

        // Closed toggle on current host works for transcripts and does NOT render X close button
        const currentToggle = screen.getByTestId("hosts-tree-toggle-closed-curr-h-1");
        fireEvent.click(currentToggle);
        const closedPtyRow = screen.getByTestId("hosts-tree-closed-pivot-piv-closed-pty");
        expect(closedPtyRow).toHaveTextContent("Transcript");
        expect(
            screen.queryByTestId("hosts-tree-close-pivot-piv-closed-pty")
        ).not.toBeInTheDocument();
        fireEvent.click(closedPtyRow);
        expect(onSelectPivot).toHaveBeenCalledWith(
            expect.objectContaining({ id: "piv-closed-pty" }),
            "10"
        );
        fireEvent.click(currentToggle);
        expect(screen.queryByTestId("hosts-tree-closed-pivot-piv-closed-pty")).not.toBeInTheDocument();
    });

    it("supports zoomed-out Access overview with consolidated Host -> Shell -> Pivot/PTY tree (with shell principals, no beacons or host IPs) and priority navigation", () => {
        const onSelectShell = vi.fn();
        const onCreateShellForHost = vi.fn();
        const onSelectPivot = vi.fn();
        const onClosePivot = vi.fn();
        const onOpenSshWithTarget = vi.fn();

        const multiBeaconHost: KnowledgeHostItem = {
            id: "h-multi",
            hostId: "h-multi",
            name: "bastion-01",
            hostname: "bastion-01.corp",
            primaryIP: "10.0.0.60",
            ipAddress: "10.0.0.60",
            subnet: "10.0.0.0/24",
            openPorts: [22],
            tags: [],
            beaconCount: 3,
            beacons: [
                {
                    id: "b-low",
                    name: "beacon-www",
                    principal: "www-data",
                    transport: SupportedTransports.HTTP1,
                    interval: 1,
                    isOnline: true,
                    hostId: "h-multi",
                    hostName: "bastion-01",
                },
                {
                    id: "b-root-http",
                    name: "beacon-root-http",
                    principal: "root",
                    transport: SupportedTransports.HTTP1,
                    interval: 5,
                    isOnline: true,
                    hostId: "h-multi",
                    hostName: "bastion-01",
                },
                {
                    id: "b-root-grpc",
                    name: "beacon-root-grpc",
                    principal: "root",
                    transport: SupportedTransports.GRPC,
                    interval: 10,
                    isOnline: true,
                    hostId: "h-multi",
                    hostName: "bastion-01",
                },
            ],
            hasOnlineBeacon: true,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
            isReachable: true,
        };

        const collabHost: KnowledgeHostItem = {
            id: "h-collab",
            hostId: "h-collab",
            name: "collab-01",
            hostname: "collab-01.corp",
            primaryIP: "10.0.0.70",
            ipAddress: "10.0.0.70",
            subnet: "10.0.0.0/24",
            openPorts: [22],
            tags: [],
            beaconCount: 1,
            beacons: [
                {
                    id: "b-collab",
                    name: "beacon-collab",
                    principal: "www-data",
                    transport: SupportedTransports.GRPC,
                    interval: 5,
                    isOnline: true,
                    hostId: "h-collab",
                    hostName: "collab-01",
                },
            ],
            hasOnlineBeacon: true,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
            isReachable: true,
        };

        const ownUserHost: KnowledgeHostItem = {
            id: "h-own",
            hostId: "h-own",
            name: "alpha-own-01",
            hostname: "alpha-own-01.corp",
            primaryIP: "10.0.0.80",
            ipAddress: "10.0.0.80",
            subnet: "10.0.0.0/24",
            openPorts: [22],
            tags: [],
            beaconCount: 1,
            beacons: [
                {
                    id: "b-own",
                    name: "beacon-own",
                    principal: "CORP\\Administrator",
                    transport: SupportedTransports.GRPC,
                    interval: 5,
                    isOnline: true,
                    hostId: "h-own",
                    hostName: "alpha-own-01",
                },
            ],
            hasOnlineBeacon: true,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
            isReachable: true,
        };

        const winSystemHost: KnowledgeHostItem = {
            id: "h-win",
            hostId: "h-win",
            name: "win-dc-01",
            hostname: "win-dc-01.corp",
            primaryIP: "10.0.0.90",
            ipAddress: "10.0.0.90",
            subnet: "10.0.0.0/24",
            openPorts: [445],
            tags: [],
            beaconCount: 1,
            beacons: [
                {
                    id: "b-win",
                    name: "beacon-win",
                    principal: "NT AUTHORITY\\SYSTEM",
                    transport: SupportedTransports.GRPC,
                    interval: 5,
                    isOnline: true,
                    hostId: "h-win",
                    hostName: "win-dc-01",
                },
            ],
            hasOnlineBeacon: true,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
            isReachable: true,
        };

        const overviewExtraShells: KnowledgeShellItem[] = [
            {
                id: "60",
                createdAt: "2026-01-01T12:06:00Z",
                beaconId: "b-collab",
                beaconName: "beacon-collab",
                beaconPrincipal: "www-data",
                beaconOnline: true,
                isLive: true,
                ownerId: "u-diana",
                ownerName: "diana",
                hostId: "h-collab",
                hostName: "collab-01",
                isCurrentHost: false,
                activeUsers: [{ id: "u-diana", name: "diana" }],
            },
            {
                id: "70",
                createdAt: "2026-01-01T12:07:00Z",
                beaconId: "b-own",
                beaconName: "beacon-own",
                beaconPrincipal: "CORP\\Administrator",
                beaconOnline: true,
                isLive: true,
                ownerId: "u-me",
                ownerName: "kcarretto",
                hostId: "h-own",
                hostName: "alpha-own-01",
                isCurrentHost: false,
                activeUsers: [{ id: "u-me", name: "kcarretto" }],
            },
            {
                id: "71",
                createdAt: "2026-01-01T12:08:00Z",
                beaconId: "b-win",
                beaconName: "beacon-win",
                beaconPrincipal: "NT AUTHORITY\\SYSTEM",
                beaconOnline: true,
                isLive: true,
                ownerId: "u-me",
                ownerName: "kcarretto",
                hostId: "h-win",
                hostName: "win-dc-01",
                isCurrentHost: false,
                activeUsers: [],
            },
        ];

        const customProtocolPivot: ShellPivotItem = {
            id: "piv-socks-5",
            kind: "SOCKS",
            destination: "svc@10.0.0.50:1080",
            closedAt: null,
            data: "SOCKS5 listening on :1080\r\n",
            shellId: "50",
            hostId: "h-5",
        };

        const { container } = render(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={[...sampleHosts, multiBeaconHost, collabHost, ownUserHost, winSystemHost]}
                shells={[...sampleShells, ...overviewExtraShells]}
                pivots={[...samplePivots, customProtocolPivot]}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                currentUserId="u-me"
                currentUserName="kcarretto"
                onSelectShell={onSelectShell}
                onCreateShellForHost={onCreateShellForHost}
                onSelectPivot={onSelectPivot}
                onClosePivot={onClosePivot}
                onOpenSshWithTarget={onOpenSshWithTarget}
            />
        );

        // Click "Access" breadcrumb to enter zoomed-out overview
        fireEvent.click(screen.getByTestId("hosts-tree-breadcrumb-hosts"));
        expect(screen.queryByTestId("hosts-tree-breadcrumb-active-host")).not.toBeInTheDocument();

        // Header omits host/pivot summary counts; overview omits "Other Hosts" label text, horizontal divider, subnets, host IPs, "Shell" word, and beacon rows
        expect(screen.queryByText(/hosts\s*·\s*\d+\s*pivots/i)).not.toBeInTheDocument();
        expect(screen.queryByText("Other Hosts")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-other-hosts-divider")).not.toBeInTheDocument();
        expect(screen.queryByText("10.0.0.0/24")).not.toBeInTheDocument();
        expect(container.querySelectorAll('[data-testid^="hosts-tree-beacon-"]')).toHaveLength(0);
        expect(container.querySelectorAll('[data-testid^="hosts-tree-host-ip-"]')).toHaveLength(0);
        expect(container.querySelectorAll('[data-testid^="hosts-tree-close-pivot-"]')).toHaveLength(0);

        // Hosts (purple | bar + regular Monitor + Zap for root, no host IP) -> Shells (with User + principal) -> indented Pivots & PTYs
        const currentSection = screen.getByTestId("hosts-tree-current-section");
        expect(screen.queryByTestId("hosts-tree-current-chevron-icon")).not.toBeInTheDocument();
        const currentIndicator = screen.getByTestId("hosts-tree-current-indicator");
        const currentMonitor = screen.getByTestId("hosts-tree-current-monitor-icon");
        expect(currentIndicator).toHaveClass("bg-purple-400");
        expect(currentMonitor).toHaveClass("text-gray-400");
        expect(
            currentIndicator.compareDocumentPosition(currentMonitor) &
                Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
        expect(screen.getByTestId("hosts-tree-privileged-icon-curr-h-1")).toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-host-ip-curr-h-1")).not.toBeInTheDocument();

        const currentShellRow = screen.getByTestId("hosts-tree-shell-10");
        expect(currentSection).toContainElement(currentShellRow);
        expect(currentShellRow).not.toHaveTextContent(/\bShell\b/);
        expect(currentShellRow).toHaveTextContent("beacon-web");
        expect(currentShellRow).toHaveTextContent("active");
        expect(screen.getByTestId("hosts-tree-shell-principal-10")).toHaveTextContent("root");
        expect(within(currentShellRow).getByAltText("alice")).toBeInTheDocument();
        expect(
            within(screen.getByTestId("hosts-tree-shell-60")).getByAltText("diana")
        ).toBeInTheDocument();
        const currentShellPivots = screen.getByTestId("hosts-tree-shell-pivots-10");
        expect(currentShellPivots).toHaveClass("ml-3.5", "pl-2.5", "border-l");
        expect(currentShellPivots).toContainElement(screen.getByTestId("hosts-tree-pivot-piv-pty-1"));

        const overviewSection = screen.getByTestId("hosts-tree-overview-section");
        expect(overviewSection).toContainElement(screen.getByTestId("hosts-tree-host-host-h-2"));
        expect(screen.queryByTestId("hosts-tree-host-ip-host-h-2")).not.toBeInTheDocument();
        const dbShellRow = screen.getByTestId("hosts-tree-shell-20");
        expect(overviewSection).toContainElement(dbShellRow);
        expect(dbShellRow).not.toHaveTextContent(/\bShell\b/);
        expect(screen.getByTestId("hosts-tree-shell-principal-20")).toHaveTextContent("postgres");
        // Offline beacon's closed shell ("21") is hidden in unfiltered Access view
        expect(screen.queryByTestId("hosts-tree-closed-shell-21")).not.toBeInTheDocument();

        // Zap overlay renders ONLY on hosts with active root / SYSTEM / Administrator shells
        expect(screen.getByTestId("hosts-tree-privileged-icon-host-h-5")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-privileged-icon-host-h-own")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-privileged-icon-host-h-win")).toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-privileged-icon-host-h-2")).not.toBeInTheDocument();
        expect(
            screen.queryByTestId("hosts-tree-privileged-icon-host-h-collab")
        ).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-privileged-icon-host-nh-3")).not.toBeInTheDocument();

        // Pivot rows are indented under their parent shell, prioritize user@ip, omit "Pivot" text, and center protocol badge
        const dbShellPivots = screen.getByTestId("hosts-tree-shell-pivots-20");
        const sshPivotRow = screen.getByTestId("hosts-tree-pivot-piv-ssh-2");
        expect(dbShellPivots).toContainElement(sshPivotRow);
        expect(sshPivotRow).not.toHaveTextContent("#piv-ssh-2");
        expect(sshPivotRow).not.toHaveTextContent(/pivot/i);
        expect(sshPivotRow).toHaveTextContent("postgres@10.0.0.20:22");
        const sshProtocolBadge = screen.getByTestId("hosts-tree-pivot-protocol-piv-ssh-2");
        expect(sshProtocolBadge).toHaveTextContent("SSH");
        expect(sshProtocolBadge).toHaveClass("inline-flex", "items-center", "justify-center", "leading-none");
        const sshTargetSpan = screen.getByText("postgres@10.0.0.20:22");
        expect(
            sshTargetSpan.compareDocumentPosition(sshProtocolBadge) &
                Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();

        const appShellPivots = screen.getByTestId("hosts-tree-shell-pivots-50");
        const socksPivotRow = screen.getByTestId("hosts-tree-pivot-piv-socks-5");
        expect(appShellPivots).toContainElement(socksPivotRow);
        expect(socksPivotRow).not.toHaveTextContent(/pivot/i);
        expect(socksPivotRow).toHaveTextContent("svc@10.0.0.50:1080");
        expect(screen.getByTestId("hosts-tree-pivot-protocol-piv-socks-5")).toHaveTextContent("SOCKS");

        expect(overviewSection).toContainElement(screen.getByTestId("hosts-tree-host-host-h-5"));
        expect(overviewSection).toContainElement(screen.getByTestId("hosts-tree-shell-50"));
        expect(screen.getByTestId("hosts-tree-shell-principal-50")).toHaveTextContent("root");
        expect(overviewSection).toContainElement(screen.getByTestId("hosts-tree-host-host-nh-3"));

        // Closed toggle in overview is per-host and hidden on non-current hosts when unfiltered
        expect(screen.queryByTestId("hosts-tree-toggle-closed-host-h-2")).not.toBeInTheDocument();
        const overviewCurrentToggle = screen.getByTestId("hosts-tree-toggle-closed-curr-h-1");
        expect(currentSection).toContainElement(overviewCurrentToggle);
        fireEvent.click(overviewCurrentToggle);
        expect(screen.getByTestId("hosts-tree-closed-pivot-piv-closed-pty")).toBeInTheDocument();
        fireEvent.click(overviewCurrentToggle);

        // Priority ordering: active pivots (app-01, db-01) > shells with other active users (collab-01) > online/own shells (alpha-own-01, bastion-01, win-dc-01) > available hosts (fileserver)
        const renderedHostIds = Array.from(
            overviewSection.querySelectorAll('[data-testid^="hosts-tree-host-"]:not([data-testid^="hosts-tree-host-ip-"])')
        ).map((el) => el.getAttribute("data-testid"));
        expect(renderedHostIds).toEqual([
            "hosts-tree-host-host-h-5",
            "hosts-tree-host-host-h-2",
            "hosts-tree-host-host-h-collab",
            "hosts-tree-host-host-h-own",
            "hosts-tree-host-host-h-multi",
            "hosts-tree-host-host-h-win",
            "hosts-tree-host-host-nh-3",
        ]);

        // Searching by host IP narrows to matching shells/pivots without rendering any beacon rows
        fireEvent.click(screen.getByTestId("hosts-tree-filters-button"));
        fireEvent.change(screen.getByTestId("hosts-tree-filter-ip"), { target: { value: "10.0.0.20" } });
        expect(screen.getByTestId("hosts-tree-shell-20")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-pivot-piv-ssh-2")).toBeInTheDocument();
        expect(container.querySelectorAll('[data-testid^="hosts-tree-beacon-"]')).toHaveLength(0);
        fireEvent.click(screen.getByTestId("hosts-tree-filters-clear-all"));
        fireEvent.keyDown(screen.getByTestId("hosts-tree-filters-popover"), { key: "Escape" });

        // 1. Clicking bastion-01 (no shells yet) runs selectBestBeaconId behind the scenes -> root + gRPC beacon ("b-root-grpc")
        fireEvent.click(screen.getByTestId("hosts-tree-host-host-h-multi").querySelector("button")!);
        expect(onCreateShellForHost).toHaveBeenCalledWith({
            hostId: "h-multi",
            hostName: "bastion-01",
            beaconId: "b-root-grpc",
        });

        // 2. Clicking a host or live shell row navigates directly to that shell (app-01 -> shell 50; shell-20 -> 20)
        fireEvent.click(screen.getByTestId("hosts-tree-host-host-h-5").querySelector("button")!);
        expect(onSelectShell).toHaveBeenCalledWith("50");

        fireEvent.click(screen.getByTestId("hosts-tree-shell-20"));
        expect(onSelectShell).toHaveBeenCalledWith("20");

        // 3. Clicking a reachable NetworkHost opens SSH modal
        fireEvent.click(screen.getByTestId("hosts-tree-host-host-nh-3").querySelector("button")!);
        expect(onOpenSshWithTarget).toHaveBeenCalledWith("10.0.0.30:22");

        // 4. Clicking a pivot or PTY calls onSelectPivot(pivot, pivot.shellId)
        fireEvent.click(screen.getByTestId("hosts-tree-pivot-piv-ssh-2"));
        expect(onSelectPivot).toHaveBeenCalledWith(
            expect.objectContaining({ id: "piv-ssh-2" }),
            "20"
        );

        // 5. Clicking current host or current shell switches back to Specific Host View
        fireEvent.click(screen.getByTestId("hosts-tree-shell-10"));
        expect(onSelectShell).toHaveBeenCalledWith("10");
        expect(screen.getByTestId("hosts-tree-breadcrumb-active-host")).toHaveTextContent("web-01");
        expect(screen.queryByTestId("hosts-tree-other-hosts-divider")).not.toBeInTheDocument();

        fireEvent.click(screen.getByTestId("hosts-tree-breadcrumb-hosts"));
        fireEvent.click(screen.getByTestId("hosts-tree-host-curr-h-1").querySelector("button")!);
        expect(screen.getByTestId("hosts-tree-breadcrumb-active-host")).toHaveTextContent("web-01");
        expect(screen.queryByTestId("hosts-tree-other-hosts-divider")).not.toBeInTheDocument();
    });

    it("supports light mode styling for breadcrumb, drawer, Add Credential button, and Filters popover", () => {
        const onAddCredential = vi.fn();
        render(
            <HostsTreeView
                theme="light"
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={samplePivots}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                onAddHost={vi.fn()}
                onAddCredential={onAddCredential}
            />
        );

        const view = screen.getByTestId("hosts-tree-view");
        expect(view).toHaveAttribute("data-theme", "light");
        expect(view).toHaveClass("bg-white", "text-gray-900");
        expect(screen.getByText("Access")).toHaveClass("text-purple-700");
        expect(screen.getByTestId("hosts-tree-current-indicator")).toHaveClass("bg-purple-600");
        expect(screen.getByTestId("hosts-tree-current-monitor-icon")).toHaveClass("text-gray-500");
        expect(screen.getByTestId("hosts-tree-add-plus-icon")).toHaveClass("bg-white");
        expect(screen.getByTestId("hosts-tree-add-credential-plus-icon")).toHaveClass("bg-white");
        fireEvent.click(screen.getByTestId("hosts-tree-add-credential-button"));
        expect(onAddCredential).toHaveBeenCalledTimes(1);

        const filtersBtn = screen.getByTestId("hosts-tree-filters-button");
        expect(filtersBtn).toHaveClass("!text-gray-700", "border-gray-300");
        fireEvent.click(filtersBtn);
        expect(screen.getByTestId("hosts-tree-filters-popover")).toHaveClass("bg-white");
        expect(screen.getByTestId("hosts-tree-filter-name")).toHaveClass("bg-gray-50", "pl-8");
        expect(screen.getByTestId("hosts-tree-filter-icon-name")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-filter-icon-ip")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-filter-icon-input")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-filter-icon-output")).toBeInTheDocument();
    });

    it("filters across Name, IP, Input, and Output with AND semantics and reveals Other Shells only when matched", () => {
        const extraTasks: ShellTaskItem[] = [
            {
                id: "ext-1",
                shellId: "20",
                createdAt: "2026-01-01T12:03:00Z",
                input: "user.info()",
                output: "uid=109(postgres)",
            },
        ];

        const baseSections = buildHostsTreeSections({
            hosts: sampleHosts,
            shells: sampleShells,
            pivots: samplePivots,
            currentHostBeacons: sampleCurrentHostBeacons,
            currentHostId: "h-1",
            currentHostName: "web-01",
            currentHostIp: "10.0.0.10",
            currentShellId: "10",
        });

        // Pure helper AND combination check: Name ("db-01") + Output ("uid=109")
        const combinedMatch = filterHostsTreeSections(
            baseSections,
            { name: "db-01", output: "uid=109" },
            extraTasks
        );
        expect(combinedMatch.currentHostNode).toBeNull();
        expect(combinedMatch.activePivotHosts).toHaveLength(1);
        expect(combinedMatch.activePivotHosts[0]?.activeShells.map((s) => s.id)).toEqual(["20"]);
        expect(combinedMatch.activePivotHosts[0]?.activePivots).toEqual([]);

        render(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={samplePivots}
                currentHostBeacons={sampleCurrentHostBeacons}
                shellTasks={extraTasks}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                currentUserId="u-me"
                currentUserName="kcarretto"
            />
        );

        // Open Filters popover
        fireEvent.click(screen.getByTestId("hosts-tree-filters-button"));
        const nameInput = screen.getByTestId("hosts-tree-filter-name");
        const ipInput = screen.getByTestId("hosts-tree-filter-ip");
        const inputField = screen.getByTestId("hosts-tree-filter-input");
        const outputField = screen.getByTestId("hosts-tree-filter-output");

        // 1. Filter by Name ("db-01") AND Input ("psql -l"): reveals "Other Shells" with db-01 -> piv-ssh-2
        fireEvent.change(nameInput, { target: { value: "db-01" } });
        fireEvent.change(inputField, { target: { value: "psql -l" } });
        expect(screen.getByTestId("hosts-tree-filters-button")).toHaveTextContent("Filters (2)");
        expect(screen.getByTestId("hosts-tree-other-shells-divider")).toHaveTextContent("Other Shells");
        expect(screen.getByTestId("hosts-tree-pivot-piv-ssh-2")).toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-current-section")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-toggle-closed-host-h-2")).not.toBeInTheDocument();

        // Adding a non-matching Output breaks AND predicate -> empty search state
        fireEvent.change(outputField, { target: { value: "nonexistent-output" } });
        expect(screen.getByTestId("hosts-tree-empty-search")).toBeInTheDocument();

        // 2. Clear all, then filter by IP ("10.0.0.20") AND Output ("Linux db-01") -> reveals closed SSH transcript on db-01
        fireEvent.click(screen.getByTestId("hosts-tree-filters-clear-all"));
        expect(screen.getByTestId("hosts-tree-filters-button")).toHaveTextContent("Filters (0)");
        expect(screen.queryByTestId("hosts-tree-other-shells-divider")).not.toBeInTheDocument();

        fireEvent.change(ipInput, { target: { value: "10.0.0.20" } });
        fireEvent.change(outputField, { target: { value: "Linux db-01" } });
        expect(screen.getByTestId("hosts-tree-toggle-closed-host-h-2")).toBeInTheDocument();
        expect(screen.getByTestId("hosts-tree-closed-pivot-piv-closed-ssh")).toHaveTextContent("Transcript");
        expect(screen.queryByTestId("hosts-tree-pivot-piv-ssh-2")).not.toBeInTheDocument();

        // 3. Filter closed PTY on current host by ANSI-stripped Output ("uid=0(root)")
        fireEvent.click(screen.getByTestId("hosts-tree-filters-clear-all"));
        fireEvent.change(outputField, { target: { value: "uid=0(root)" } });
        expect(screen.getByTestId("hosts-tree-closed-pivot-piv-closed-pty")).toHaveTextContent("Transcript");
        expect(screen.queryByTestId("hosts-tree-pivot-piv-pty-1")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-active-pivots-section")).not.toBeInTheDocument();

        // 4. Filter by Input ("file.list") -> reveals current host shell 10 task match
        fireEvent.click(screen.getByTestId("hosts-tree-filters-clear-all"));
        fireEvent.change(inputField, { target: { value: "file.list" } });
        expect(screen.getByTestId("hosts-tree-shell-10")).toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-pivot-piv-pty-1")).not.toBeInTheDocument();

        // 5. Closed shells on non-current hosts remain hidden even when searching ("audit_admin")
        fireEvent.click(screen.getByTestId("hosts-tree-filters-clear-all"));
        fireEvent.change(outputField, { target: { value: "audit_admin" } });
        expect(screen.queryByTestId("hosts-tree-toggle-closed-host-h-2")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-closed-shell-21")).not.toBeInTheDocument();

        // 6. Unpivoted host app-01 appears in "Other Shells" when Name matches ("beacon-app"), then disappears on Clear
        fireEvent.click(screen.getByTestId("hosts-tree-filters-clear-all"));
        fireEvent.change(nameInput, { target: { value: "beacon-app" } });
        expect(screen.getByTestId("hosts-tree-host-host-h-5")).toBeInTheDocument();
        fireEvent.click(screen.getByTestId("hosts-tree-clear-search"));
        expect(screen.queryByTestId("hosts-tree-host-host-h-5")).not.toBeInTheDocument();
        expect(screen.queryByTestId("hosts-tree-other-shells-divider")).not.toBeInTheDocument();
    });

    it("does not display NetworkHosts under the current host when an active pivot exists to that host", () => {
        const secondaryDbNetHost: KnowledgeHostItem = {
            id: "nh-db-nic2",
            hostId: "h-2",
            networkHostId: "nh-db-nic2",
            name: "db-01-nic2",
            hostname: "db-01-nic2.corp",
            primaryIP: "10.0.10.20",
            ipAddress: "10.0.10.20",
            subnet: "10.0.10.0/24",
            openPorts: [22],
            tags: [],
            beaconCount: 0,
            credentials: [],
            credentialCount: 0,
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: false,
            isReachable: true,
        };

        const activeIpPivotToFileserver: ShellPivotItem = {
            id: "piv-fileserver-ip",
            kind: "SSH",
            destination: "ops@10.0.0.30:22",
            closedAt: null,
            data: "ops@fileserver:~$ id\r\n",
            shellId: "10",
            hostId: "h-1",
        };

        const activeFqdnPivotToFileserver: ShellPivotItem = {
            id: "piv-fileserver-fqdn",
            kind: "SSH",
            destination: "ops@fileserver.corp:22",
            activeTarget: "ops@fileserver:22",
            closedAt: null,
            data: "ops@fileserver:~$ ls\r\n",
            shellId: "10",
            hostId: "h-1",
        };

        const sectionsWithIpPivot = buildHostsTreeSections({
            hosts: [...sampleHosts, secondaryDbNetHost],
            shells: sampleShells,
            pivots: [...samplePivots, activeIpPivotToFileserver],
            currentHostBeacons: sampleCurrentHostBeacons,
            currentHostId: "h-1",
            currentHostName: "web-01",
            currentHostIp: "10.0.0.10",
            currentShellId: "10",
        });

        expect(sectionsWithIpPivot.availablePivotHosts).toEqual([]);
        expect(sectionsWithIpPivot.activePivotHosts.map((n) => n.name)).toEqual([
            "db-01",
            "fileserver",
        ]);

        const { rerender } = render(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={[...sampleHosts, secondaryDbNetHost]}
                shells={sampleShells}
                pivots={[...samplePivots, activeIpPivotToFileserver]}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
            />
        );

        const currentSection = screen.getByTestId("hosts-tree-current-section");
        expect(
            screen.queryByTestId("hosts-tree-available-host-host-nh-3")
        ).not.toBeInTheDocument();
        expect(
            screen.queryByTestId("hosts-tree-available-host-host-nh-db-nic2")
        ).not.toBeInTheDocument();
        // Outgoing SSH pivot created from web-01 is surfaced directly in Specific Host View without duplicating the NetworkHost
        expect(currentSection).toContainElement(screen.getByTestId("hosts-tree-pivot-piv-fileserver-ip"));

        // Also works when active pivot matches via FQDN/activeTarget instead of IP
        rerender(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={[...sampleHosts, secondaryDbNetHost]}
                shells={sampleShells}
                pivots={[...samplePivots, activeFqdnPivotToFileserver]}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
            />
        );

        expect(
            screen.queryByTestId("hosts-tree-available-host-host-nh-3")
        ).not.toBeInTheDocument();
        expect(
            screen.getByTestId("hosts-tree-pivot-piv-fileserver-fqdn")
        ).toBeInTheDocument();
    });

    it("shows a terminal Open PTY button on the current host when no active PTY exists, disabled with tooltip when no portal is open", () => {
        const onOpenPty = vi.fn();
        const pivotsWithoutActivePty = samplePivots.filter((p) => p.id !== "piv-pty-1");

        // 1. Active PTY present -> button is NOT shown
        const { rerender } = render(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={samplePivots}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                portalOpen={true}
                onOpenPty={onOpenPty}
            />
        );
        expect(screen.queryByTestId("hosts-tree-open-pty-button")).not.toBeInTheDocument();

        // 2. No active PTY & portal open -> enabled terminal button opens PTY without toggling closed section
        rerender(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={pivotsWithoutActivePty}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                portalOpen={true}
                onOpenPty={onOpenPty}
            />
        );
        const currentSection = screen.getByTestId("hosts-tree-current-section");
        const enabledBtn = screen.getByTestId("hosts-tree-open-pty-button");
        expect(currentSection).toContainElement(enabledBtn);
        expect(enabledBtn).toBeEnabled();
        expect(enabledBtn.querySelector("svg")).toBeInTheDocument();
        expect(enabledBtn.closest("[title]")?.getAttribute("title")).toBe("Open PTY");

        fireEvent.click(enabledBtn);
        expect(onOpenPty).toHaveBeenCalledTimes(1);
        expect(
            screen.queryByTestId("hosts-tree-closed-pivot-piv-closed-pty")
        ).not.toBeInTheDocument();

        // 3. No active PTY & no open portal (async mode) -> disabled greyed-out button with instructional tooltip
        onOpenPty.mockClear();
        rerender(
            <HostsTreeDrawer
                isOpen={true}
                onClose={vi.fn()}
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={pivotsWithoutActivePty}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                portalOpen={false}
                onOpenPty={onOpenPty}
            />
        );
        const disabledBtn = screen.getByTestId("hosts-tree-open-pty-button");
        expect(disabledBtn).toBeDisabled();
        expect(disabledBtn).toHaveClass("opacity-50", "cursor-not-allowed");
        expect(disabledBtn.closest("[title]")?.getAttribute("title")).toBe(
            "PTY unavailable in async mode, please create a portal first."
        );

        fireEvent.click(disabledBtn);
        expect(onOpenPty).not.toHaveBeenCalled();

        // 4. Light mode also renders disabled styling properly
        rerender(
            <HostsTreeView
                theme="light"
                hosts={sampleHosts}
                shells={sampleShells}
                pivots={pivotsWithoutActivePty}
                currentHostBeacons={sampleCurrentHostBeacons}
                currentHostId="h-1"
                currentHostName="web-01"
                currentHostIp="10.0.0.10"
                currentShellId="10"
                portalOpen={false}
                onOpenPty={onOpenPty}
            />
        );
        const lightDisabledBtn = screen.getByTestId("hosts-tree-open-pty-button");
        expect(lightDisabledBtn).toBeDisabled();
        expect(lightDisabledBtn).toHaveClass("text-gray-400", "opacity-50", "cursor-not-allowed");
    });
});
