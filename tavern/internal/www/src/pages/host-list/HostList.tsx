import React from "react";
import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import Breadcrumbs from "../../components/Breadcrumbs";
import { useHosts } from "./useHosts";
import { FilterControls, FilterPageType } from "../../context/FilterContext/index";
import HostTable from "./HostTable";
import { TablePagination, TableWrapper } from "../../components/tavern-base-ui/table";
import { SortingControls } from "../../context/SortContext/index";

const HostList: React.FC = () => {
    const { loading, error, data, updateHosts, page, setPage } = useHosts(true);

    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <Breadcrumbs
                pages={[{
                    label: "Hosts",
                    link: "/hosts"
                }]}
            />
            <TableWrapper
                title="Hosts"
                totalItems={data?.hosts?.totalCount || null}
                loading={loading}
                error={error}
                filterControls={<FilterControls type={FilterPageType.HOST} />}
                sortingControls={<SortingControls type={PageNavItem.hosts} />}
                table={<HostTable data={data?.hosts?.edges || []} />}
                pagination={
                    <TablePagination
                        totalCount={data?.hosts?.totalCount || 0}
                        pageInfo={data?.hosts?.pageInfo || { hasNextPage: false, hasPreviousPage: false, startCursor: null, endCursor: null }}
                        refetchTable={updateHosts}
                        page={page}
                        setPage={setPage}
                        rowLimit={TableRowLimit.HostRowLimit}
                    />
                }
            />
        </PageWrapper>
    );
};

export default HostList;
