import React from "react";
import { PageWrapper } from "../../features/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import HostFilter from "./components/HostFilter";
import HostTable from "./components/HostTable";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { useHosts } from "../../hooks/useHosts";
import TablePagination from "../../components/tavern-base-ui/TablePagination";

const HostList = () => {
    const { loading, error, data, updateHosts, page, setPage, setFiltersSelected, filtersSelected } = useHosts(true);

    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <Breadcrumbs pages={[{
                label: "Hosts",
                link: "/hosts"
            }]} />
            <PageHeader title="Hosts" description="Hosts are in-scope systems for the current engagement. A host can have multiple beacons which can execute instructions provided by tomes." />
            <HostFilter setFiltersSelected={setFiltersSelected} typeFilters={filtersSelected} />
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
