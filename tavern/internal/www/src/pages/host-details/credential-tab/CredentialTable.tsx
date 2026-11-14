import { useCallback, useMemo } from "react";
import { formatDistance } from "date-fns";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Tooltip from "../../../components/tavern-base-ui/Tooltip";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { GET_HOST_CREDENTIALS } from "../../../utils/queries";
import { CredentialNode, HostCredentialsQueryTopLevel } from "../../../utils/interfacesQuery";
import Credential from "./Credential";

export interface CredentialTableProps {
    hostId?: number;
    data: CredentialNode[];
    showLastModified?: boolean;
}

const getRowKey = (cred: CredentialNode, index: number): string =>
    cred.id ? String(cred.id) : `${cred.principal}-${cred.kind}-${index}`;

const CredentialTable = ({ hostId, data, showLastModified = false }: CredentialTableProps) => {
    const currentDate = useMemo(() => new Date(), []);

    const items = useMemo(
        () => data.map((cred, index) => getRowKey(cred, index)),
        [data]
    );

    const dataById = useMemo(() => {
        const map = new Map<string, CredentialNode>();
        data.forEach((cred, index) => {
            map.set(getRowKey(cred, index), cred);
        });
        return map;
    }, [data]);

    const getVariables = useCallback(
        (id: string) => ({
            where: hostId !== undefined ? { id: hostId } : undefined,
            credentialWhere: { id },
        }),
        [hostId]
    );

    const extractData = useCallback(
        (response: HostCredentialsQueryTopLevel, itemId: string): CredentialNode | null => {
            const localCred = dataById.get(itemId);
            const edges = response?.hosts?.edges?.[0]?.node?.credentials?.edges;
            const matchedServerCred = edges?.find((e) => String(e.node.id) === String(itemId))?.node;
            const cred = localCred
                ? { ...matchedServerCred, ...localCred }
                : matchedServerCred;
            if (!cred) {
                return null;
            }
            return {
                ...cred,
                kind: cred.kind.replace(/^KIND_/, ""),
            };
        },
        [dataById]
    );

    const columns: VirtualizedTableColumn<CredentialNode>[] = useMemo(() => {
        const cols: VirtualizedTableColumn<CredentialNode>[] = [
            {
                key: "createdAt",
                label: "Created",
                width: "minmax(120px,1fr)",
                render: (cred: CredentialNode) => formatDistance(new Date(cred.createdAt), currentDate),
            },
        ];

        if (showLastModified) {
            cols.push({
                key: "lastModifiedAt",
                label: "Last Modified",
                width: "minmax(120px,1fr)",
                render: (cred: CredentialNode) => formatDistance(new Date(cred.lastModifiedAt), currentDate),
            });
        }

        cols.push(
            {
                key: "principal",
                label: "Principal",
                width: "minmax(120px,1fr)",
                render: (cred: CredentialNode) => (
                    <Tooltip label={cred.principal} isDisabled={!cred.principal}>
                        <div className="truncate text-sm text-gray-600">
                            {cred.principal || "-"}
                        </div>
                    </Tooltip>
                ),
            },
            {
                key: "kind",
                label: "Credential Kind",
                width: "minmax(120px,1fr)",
                render: (cred: CredentialNode) => (
                    <div className="flex flex-row flex-wrap gap-1">
                        <Badge badgeStyle={{ color: "gray" }}>{cred.kind}</Badge>
                    </div>
                ),
            },
            {
                key: "secret",
                label: "Secret",
                width: "minmax(200px,2fr)",
                render: (cred: CredentialNode) => (
                    <div className="flex justify-between items-center gap-2 min-w-0">
                        <Credential secret={cred.secret} />
                    </div>
                ),
            }
        );

        return cols;
    }, [currentDate, showLastModified]);

    return (
        <VirtualizedTable<CredentialNode, HostCredentialsQueryTopLevel>
            items={items}
            columns={columns}
            query={GET_HOST_CREDENTIALS}
            getVariables={getVariables}
            extractData={extractData}
            height="60vh"
        />
    );
};
export default CredentialTable;
