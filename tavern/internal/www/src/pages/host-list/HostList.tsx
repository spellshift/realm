import React from "react";
import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import Breadcrumbs from "../../components/Breadcrumbs";
import { useHosts } from "./useHosts";
import FilterControls, { FilterPageType } from "../../components/FilterControls";
import HostTable from "./HostTable";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import SortingControls from "../../components/SortingControls";

const HostList: React.FC = () => {
    const { loading, error, data, updateHosts, page, setPage } = useHosts(true);

    const renderHostContent = () => {
        if (error) {
            return <EmptyState type={EmptyStateType.error} label="Error loading hosts" />;
        }
        else if (loading || !data) {
            return <EmptyState type={EmptyStateType.loading} label="Loading hosts..." />;
        }
        else if (data?.hosts.totalCount === 0) {
            return <EmptyState type={EmptyStateType.noData} label="No hosts found" />;
        }

        return (
            <div className="flex flex-col gap-1 w-full">
                <HostTable data={data.hosts?.edges} />
                <TablePagination
                    totalCount={data.hosts.totalCount}
                    pageInfo={data.hosts.pageInfo}
                    refetchTable={updateHosts}
                    page={page}
                    setPage={setPage}
                    rowLimit={TableRowLimit.HostRowLimit}
                />
            </div>
        );
    };

    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <Breadcrumbs
                pages={[{
                    label: "Hosts",
                    link: "/hosts"
                }]}
            />
            <div className="flex flex-col md:flex-row justify-between items-end px-4 py-2 border-b border-gray-200 pb-5">
                <div className="flex-1 flex flex-col gap-2">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Hosts
                    </h3>
                    <p className="max-w-2xl text-sm">
                        Hosts are in-scope systems for the current engagement. A host can have multiple beacons which can execute instructions provided by tomes.
                    </p>
                </div>
                <div className="flex flex-row justify-end m">
                    <SortingControls type={PageNavItem.hosts} />
                    <FilterControls type={FilterPageType.HOST} />
                </div>
            </div>
            <div className="flex flex-col justify-center items-center gap-6">
                {renderHostContent()}
            </div>

        </PageWrapper>
    );
};

export default HostList;
