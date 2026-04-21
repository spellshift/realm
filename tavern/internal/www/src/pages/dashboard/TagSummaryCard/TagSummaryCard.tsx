import { FC, useState } from "react";
import { Tab, TabGroup, TabList } from "@headlessui/react";
import { Table } from "../../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useTagSummaryData } from "./useTagSummaryData";
import { useTagSummaryColumns } from "./useTagSummaryColumns";
import { TagKind } from "./types";

const TAB_KINDS: TagKind[] = ["group", "service"];
const TAB_LABELS = ["Group", "Service"];

export const TagSummaryCard: FC = () => {
    const [selectedIndex, setSelectedIndex] = useState(0);
    const tagKind = TAB_KINDS[selectedIndex];
    const { rows, loading, error } = useTagSummaryData(tagKind);
    const columns = useTagSummaryColumns();

    return (
        <div className="bg-white rounded-lg border border-gray-200 py-2 px-6 flex flex-col gap-4">
            <div className="flex flex-row justify-between items-center">
                <h3 className="text-lg font-semibold text-gray-900">Access by Tag</h3>

                <TabGroup selectedIndex={selectedIndex} onChange={setSelectedIndex}>
                    <TabList className="flex rounded-lg bg-gray-100 p-1">
                        {TAB_LABELS.map((label) => (
                            <Tab
                                key={label}
                                className={({ selected }) =>
                                    `px-4 py-1.5 text-sm font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 ${selected
                                        ? "bg-white text-purple-800 semi-bold shadow-sm"
                                        : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
                                    }`
                                }
                            >
                                {label}
                            </Tab>
                        ))}
                    </TabList>
                </TabGroup>
            </div>

            {loading && rows.length === 0 ? (
                <EmptyState type={EmptyStateType.loading} label="Loading tags..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Failed to load tag data" details={error.message} />
            ) : rows.length === 0 ? (
                <EmptyState type={EmptyStateType.noData} label={`No ${tagKind} tags found`} />
            ) : (
                <div className="h-72 w-full overflow-x-scroll">
                    <Table columns={columns} data={rows} initialSorting={[{ id: "tagName", desc: false }]} />
                </div>
            )}
        </div>
    );
};

export default TagSummaryCard;
