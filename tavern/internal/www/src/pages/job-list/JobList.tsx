import { gql, useQuery } from "@apollo/client";
import React from "react";
import { Link } from "react-router-dom";

import { CreateJobDrawer } from "../../components/create-job-drawer/CreateJobDrawer";
import { FormSteps } from "../../components/form-steps";
import { PageWrapper } from "../../components/page-wrapper";
import { GET_JOB_QUERY } from "../../utils/queries";

export const JobList = () => {

    const { loading, error, data } = useQuery(GET_JOB_QUERY);
    

    return (
        <PageWrapper>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Jobs</h3>
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
            <div>
                {data?.jobs?.map((item: any) =>{
                    return <div key={item?.id}>{item?.name}</div>
                })}
            </div>
        </PageWrapper>
    );
}