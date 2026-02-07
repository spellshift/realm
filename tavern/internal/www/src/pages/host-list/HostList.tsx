import React from "react";
import { TableRowLimit } from "../../utils/enums";
import Breadcrumbs from "../../components/Breadcrumbs";
import { useHosts } from "./useHosts";
import HostTable from "./HostTable";
import { TablePagination, TableWrapper } from "../../components/tavern-base-ui/table";

const HostList: React.FC = () => {
    const { loading, initialLoading, error, data, updateHosts, page, setPage } = useHosts(true);

    return (
        <>
            <Breadcrumbs
                pages={[{
                    label: "Hosts",
                    link: "/hosts"
                }]}
            />
            <TableWrapper
                title="Hosts"
                totalItems={data?.hosts?.totalCount}
                loading={initialLoading}
                error={error}
                table={<HostTable data={data?.hosts?.edges || []} />}
                pagination={
                    <TablePagination
                        totalCount={data?.hosts?.totalCount || 0}
                        pageInfo={data?.hosts?.pageInfo || { hasNextPage: false, hasPreviousPage: false, startCursor: null, endCursor: null }}
                        refetchTable={updateHosts}
                        page={page}
                        setPage={setPage}
                        rowLimit={TableRowLimit.HostRowLimit}
                        loading={loading}
                    />
                }
            />
        </>
    );
};

export default HostList;
