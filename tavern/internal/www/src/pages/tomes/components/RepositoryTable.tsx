import { Tooltip } from "@chakra-ui/react";
import { ArrowPathIcon, ChevronDownIcon, ChevronRightIcon, ClipboardDocumentIcon } from "@heroicons/react/24/outline";
import { ColumnDef, Row } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Table from "../../../components/tavern-base-ui/Table";
import TomeAccordion from "../../../components/TomeAccordion";
import { TomeNode } from "../../../utils/interfacesQuery";
import { constructTomeParams } from "../../../utils/utils";
import { useFetchRepositoryTome } from "../hooks/useFetchRepostioryTomes";
import Button from "../../../components/tavern-base-ui/button/Button";
import UserImageAndName from "../../../components/UserImageAndName";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { RepositoryRow } from "../../../utils/interfacesUI";


const RepositoryTable = ({ repositories }: {
    repositories: Array<RepositoryRow>
}) => {
    const currentDate = new Date();
    const {
        importRepositoryTomes,
        loading,
    } = useFetchRepositoryTome(undefined, true);

    const renderSubComponent = ({ row }: { row: Row<RepositoryRow> }) => {
        return (
            <div className="px-8">
                {row?.original?.node?.tomes.map((tome: TomeNode) => {
                    const params = constructTomeParams("[]", tome.paramDefs);
                    return (
                        <div key={tome.id}>
                            <TomeAccordion tome={tome} params={params} />
                        </div>
                    )
                })}

                {row?.original?.node?.tomes.length < 1 &&
                    <EmptyState type={EmptyStateType.noData} label="No tomes found" details="Try refetching the repository" />
                }
            </div>
        )
    }

    const columns: ColumnDef<any>[] = [
        {
            id: 'expander',
            header: 'Repository',
            accessorFn: row => row.node.url,
            footer: props => props.column.id,
            enableSorting: false,
            cell: ({ row }) => {
                const url = row?.original?.node?.url;
                const hasLink = url.includes("http");

                return (
                    <div className="flex flex-row gap-2 items-center" >
                        {row.getIsExpanded() ? <div><ChevronDownIcon className="w-4 h-4" /></div> : <div><ChevronRightIcon className="w-4 h-4" /></div>}
                        <div className="flex flex-row gap-2 flex-wrap">
                            {hasLink ? <a href={url} target="_blank" rel="noreferrer" className="external-link">{url}</a> : <div className="break-all">{url}</div>}
                            {row?.original?.node?.repoType === "FIRST_PARTY" &&
                                (
                                    <div><Badge badgeStyle={{ color: "purple" }}>First Party</Badge></div>
                                )}
                        </div>
                    </div>
                );
            },
        },
        {
            id: "owner",
            header: 'Uploader',
            accessorFn: row => row?.node?.owner,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 60,
            cell: (cellData: any) => {
                const creatorData = cellData.getValue();
                return <UserImageAndName userData={creatorData} />
            }
        },
        {
            id: "lastModifiedAt",
            header: 'Updated',
            accessorFn: row => row?.node?.lastModifiedAt ? formatDistance(new Date(row?.node?.lastModifiedAt), currentDate) : "-",
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 60,
        },
        {
            id: "tomes",
            header: 'Tomes',
            accessorFn: row => row?.node?.tomes ? row?.node?.tomes.length : "-",
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 40,
        },
        {
            id: "id",
            header: 'Actions',
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 40,
            cell: ({ row }) => {
                if (row?.original?.node?.repoType === "FIRST_PARTY") {
                    return <div></div>
                }
                return (
                    <div className="flex flex-row">
                        <Tooltip label="Refetch tomes">
                            <Button
                                id="ignoreRowClick"
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<ArrowPathIcon className="w-4 h-4" id="ignoreRowClick"
                                />}
                                aria-label="Refetch tomes"
                                disabled={loading ? true : false}
                                onClick={() => importRepositoryTomes(row?.original?.node?.id)}
                            />
                        </Tooltip>
                        <Tooltip label="Copy public key">
                            <Button
                                id="ignoreRowClick"
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                aria-label="Copy public key"
                                leftIcon={<ClipboardDocumentIcon id="ignoreRowClick"
                                    className="w-4 h-4" />}
                                onClick={() => navigator.clipboard.writeText(row?.original?.node?.publicKey)}
                            />
                        </Tooltip>
                    </div>
                );
            }
        },
    ];

    return (
        <Table
            data={repositories}
            columns={columns}
            getRowCanExpand={() => true}
            onRowClick={(row, event) => {
                const clickId = event?.target?.id;
                if (clickId !== "ignoreRowClick") {
                    let toggle = row.getToggleExpandedHandler();
                    toggle();
                }
            }}
            renderSubComponent={renderSubComponent}
        />
    );
};
export default RepositoryTable;
