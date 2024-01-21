import React from "react";
import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import HostFilter from "./components/HostFilter";
import HostTable from "./components/HostTable";
import { useHostsFilter } from "./hooks/useHostsFilter";
import { useHostTable } from "./hooks/useHostsTable";

const HostList = () => {
    const { loading, hosts, error } = useHostTable();
    const { loading: filterLoading, filteredHosts, setTypeFilters } = useHostsFilter(hosts);

    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Hosts</h3>
            </div>
            <HostFilter setFiltersSelected={setTypeFilters} />
            <div className="flex flex-col justify-center items-center gap-6">
                {(loading || filterLoading) ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading hosts..." />
                ) : error ? (
                    <EmptyState type={EmptyStateType.error} label="Error hosts..." />
                ) : (filteredHosts.length > 0) ? (
                    <div className="py-4 bg-white rounded-lg shadow-lg mt-2 flex flex-col gap-1 w-full">
                        <HostTable data={filteredHosts} />
                    </div>
                ) : (hosts.length > 0) ? (
                    <EmptyState type={EmptyStateType.noMatches} label="No hosts matching search filters" />
                ) : (
                    <EmptyState type={EmptyStateType.noData} label="No hosts data found" />
                )}
            </div>
        </PageWrapper>
    );
}
export default HostList;
