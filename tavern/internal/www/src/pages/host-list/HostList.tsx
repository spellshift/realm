import React from "react";
import { PageWrapper } from "../../features/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import HostTable from "./components/HostTable";
import Breadcrumbs from "../../components/Breadcrumbs";
import { useHosts } from "./useHosts";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import FilterControls, { FilterPageType } from "../../components/filter-controls";

const HostList = () => {
    const { loading, error, data, updateHosts, page, setPage } = useHosts(true);

    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <Breadcrumbs pages={[{
                label: "Hosts",
                link: "/hosts"
            }]}
            />
            <div className="flex flex-row justify-between items-end px-4 py-2 border-b border-gray-200 pb-5">
                <div className="flex-1 flex flex-col gap-2">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Hosts
                    </h3>
                    <p className="max-w-2xl text-sm">
                        Hosts are in-scope systems for the current engagement. A host can have multiple beacons which can execute instructions provided by tomes.
                    </p>
                </div>
                <div className="flex flex-row justify-end">
                    {/* Sorting not added yet */}
                    {/* <Button leftIcon={<Bars3BottomLeftIcon className="w-4" />} buttonVariant="ghost" buttonStyle={{ color: 'gray', size: "md" }} onClick={() => console.log("hi")}>Sort</Button> */}
                    <FilterControls type={FilterPageType.HOST} />
                </div>
            </div>
            <div className="flex flex-col justify-center items-center gap-6">
                {error ? (
                    <EmptyState type={EmptyStateType.error} label="Error hosts..." />
                ) : !data.hasData || loading ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading hosts..." />
                ) : data?.hosts?.length === 0 ? (
                    <EmptyState type={EmptyStateType.noData} label="No hosts data found" />
                )
                    : (
                        <div className="flex flex-col gap-1 w-full">
                            <HostTable data={data?.hosts} />
                            <TablePagination totalCount={data.totalCount} pageInfo={data.pageInfo} refetchTable={updateHosts} page={page} setPage={setPage} rowLimit={TableRowLimit.HostRowLimit} />
                        </div>
                    )}
            </div>
        </PageWrapper>
    );
}
export default HostList;
