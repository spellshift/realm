import React from "react";
import { ArrowLeftIcon } from "@heroicons/react/24/outline";

import { PageWrapper } from "../../components/page-wrapper";
import { Link, useParams } from "react-router-dom";
import { GET_JOB_QUERY } from "../../utils/queries";
import { useQuery } from "@apollo/client";
import { TaskList } from "./task-list";

export const Job = () => {
    let { jobId } = useParams();

    const PARAMS = {
        variables: {
            where: {id: jobId}
        }
    }
    const { loading, error, data } = useQuery(GET_JOB_QUERY, PARAMS);
    console.log(data);

    return (
        <PageWrapper>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                    <h3 className="text-2xl font-semibold leading-6 text-gray-900">Job details ({data?.jobs[0]?.name})</h3>
                <div className="mt-3 sm:mt-0 sm:ml-4">
                    <Link to="/createJob">
                        <button
                            type="button"
                            className="inline-flex items-center rounded-md bg-purple-700 px-6 py-4 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                        >
                            Create new job
                        </button>
                    </Link>
                </div>
            </div>
            {loading ? "loading..." : <TaskList tasks={data?.jobs[0]?.tasks} />}
        </PageWrapper>
    );
};