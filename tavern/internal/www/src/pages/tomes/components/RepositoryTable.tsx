import { Badge, Button, Image } from "@chakra-ui/react";
import { ArrowPathIcon, ChevronDownIcon, ChevronRightIcon } from "@heroicons/react/24/outline";
import { ColumnDef, Row } from "@tanstack/react-table";
import { formatDistance } from "date-fns";
import Table from "../../../components/tavern-base-ui/Table";
import TomeAccordion from "../../../components/TomeAccordion";
import { RepositoryRow, Tome } from "../../../utils/consts";
import { constructTomeParams } from "../../../utils/utils";

const RepositoryTable = ({ repositories }: {
    repositories: Array<RepositoryRow>
}) => {
    const currentDate = new Date();

    const renderSubComponent = ({ row }: { row: Row<RepositoryRow> }) => {
        return (
            // <pre style={{ fontSize: '10px' }}>
            //     <code>{JSON.stringify(row.original, null, 2)}</code>
            // </pre>
            <div className="px-8">
                {row?.original?.node?.tomes.map((tome: Tome) => {
                    const params = constructTomeParams("[]", tome.paramDefs);
                    return (
                        <TomeAccordion tome={tome} params={params} />
                    )
                })}
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
                const link = url.replace("ssh://", "https://");

                return row.getCanExpand() ? (
                    <div className="flex flex-row gap-2 items-center" >
                        {row.getIsExpanded() ? <div><ChevronDownIcon className="w-4 h-4" /></div> : <div><ChevronRightIcon className="w-4 h-4" /></div>}
                        <div className="flex flex-row gap-2 flex-wrap">
                            <a href={link} className="text-gray-600 hover:text-purple-900 font-semibold  underline hover:cursor-pointer">{url}</a>
                            {row?.original?.node?.repoType === "FIRST_PARTY" &&
                                (
                                    <div><Badge colorScheme="purple" px='2'>First Party</Badge></div>
                                )}
                        </div>
                    </div>
                ) : (
                    <div className="flex flex-row gap-2 flex-wrap">
                        <a href={link} className="text-gray-600 hover:text-purple-900 font-semibold  underline hover:cursor-pointer">{url}</a>
                        {row?.original?.node?.repoType === "FIRST_PARTY" &&
                            (
                                <div><Badge colorScheme="purple" px='2'>First Party</Badge></div>
                            )}
                    </div>
                )
            },
        },
        {
            id: "owner",
            header: 'imported by',
            accessorFn: row => row?.node?.owner,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 80,
            cell: (cellData: any) => {
                const creatorData = cellData.getValue();

                if (creatorData) {
                    <div className="flex flex-row gap-2 items-center">
                        <Image
                            borderRadius='full'
                            boxSize='20px'
                            src={creatorData?.photoURL}
                            alt={`Profile of ${creatorData?.name}`}
                        />
                        <div className="text-sm flex flex-row gap-1 items-center text-gray-500">
                            {creatorData?.name}
                        </div>
                    </div>
                }
                else {
                    return <div>-</div>
                }
            }
        },
        {
            id: "lastModifiedAt",
            header: 'Last modified',
            accessorFn: row => row?.node?.lastModifiedAt ? formatDistance(new Date(row.lastModifiedAt), currentDate) : "-",
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 80,
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
            header: 'Refetch',
            accessorFn: row => row,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 40,
            cell: ({ row }) => {
                if (row?.original?.node?.repoType === "FIRST_PARTY") {
                    return <div></div>
                }
                return <Button variant="ghost" leftIcon={<ArrowPathIcon className="w-4 h-4" />} />
            }
        },
    ];

    return (
        <Table
            data={repositories}
            columns={columns}
            getRowCanExpand={() => true} onRowClick={(row) => {
                let toggle = row.getToggleExpandedHandler();
                toggle();
            }}
            renderSubComponent={renderSubComponent}
        />
    );
};
export default RepositoryTable;
