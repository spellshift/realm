import React from "react";
import { PageWrapper } from "../../features/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import HostFilter from "./components/HostFilter";
import HostTable from "./components/HostTable";
import { useHostsFilter } from "./hooks/useHostsFilter";
import { useHostTable } from "./hooks/useHostsTable";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";

const HostList = () => {
    const { loading, hosts, error } = useHostTable();
    const { loading: filterLoading, filteredHosts, setTypeFilters, typeFilters } = useHostsFilter(hosts);

    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <Breadcrumbs pages={[{
                label: "Hosts",
                link: "/hosts"
            }]} />
            <PageHeader title="Hosts" description="Hosts are in-scope systems for the current engagement. A host can have multiple beacons which can execute instructions provided by tomes." />
            <HostFilter setFiltersSelected={setTypeFilters} typeFilters={typeFilters} />
            <div className="flex flex-col justify-center items-center gap-6">
                {(loading || filterLoading) ? (
                    <EmptyState type={EmptyStateType.loading} label="Loading hosts..." />
                ) : error ? (
                    <EmptyState type={EmptyStateType.error} label="Error hosts..." />
                ) : (filteredHosts.length > 0) ? (
                    <div className="flex flex-col gap-1 w-full">
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
