import { useQuery } from "@apollo/client";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { PageWrapper } from "../../components/page-wrapper";
import TaskTable from "../../components/TaskTable";
import { TaskOutput } from "../../components/task-output";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import TablePagination from "../../components/tavern-base-ui/TablePagination";
import { HostType } from "../../utils/consts";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { GET_HOST_QUERY } from "../../utils/queries";
import { TASK_PAGE_TYPE, useTasks } from "../tasks/useTasks";
import EditableHostHeader from "./components/EditableHostHeader";
import HostStatistics from "./components/HostStatistics";
import FilterBar from "../tasks/FilterBar";
import FreeTextSearch from "../tasks/FreeTextSearch";
import HostTasks from "./components/HostTasks";

const HostDetails = () => {
    const { hostId } = useParams();
    const { loading, data, error } = useQuery(GET_HOST_QUERY, {
        variables: {
            "where": {
                "id": hostId
            }
        }
    });

    const host = data?.hosts && data?.hosts.length > 0 ? data.hosts[0] : null as HostType | null;


    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <EditableHostHeader hostId={hostId} loading={loading} error={error} hostData={host} />
            </div>
            {host && (
                <div className="flex flex-col gap-6 py-6">
                    <HostStatistics host={host} />
                </div>
            )}

            <HostTasks />
        </PageWrapper>
    );
}
export default HostDetails;
