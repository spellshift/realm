import React, { useState } from "react";
import { ArrowLeftIcon } from "@heroicons/react/24/outline";

import { PageWrapper } from "../../components/page-wrapper";
import { Link, useParams } from "react-router-dom";
import { GET_QUEST_QUERY } from "../../utils/queries";
import { useQuery } from "@apollo/client";
import { TaskTable } from "./task-table";
import { TaskOutput } from "../../components/task-output";
import { Task } from "../../utils/consts";
import { PageNavItem } from "../../utils/enums";

export const QuestDetails = () => {
    let { questId } = useParams();
    const [isOpen, setOpen] = useState(false);
    const [selectedTask, setSelectedTask] = useState<Task | null>(null);

    const PARAMS = {
        variables: {
            where: {id: questId}
        }
    }
    const { loading, error, data } = useQuery(GET_QUEST_QUERY, PARAMS);

    const handleClick =(e: any) => {
        const selectedTaskData = e?.original as Task
        setSelectedTask(selectedTaskData);
        setOpen((state)=> !state);
    }

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                    <h3 className="text-2xl font-semibold leading-6 text-gray-900">Task details for {data?.quests[0]?.name}</h3>
                <div className="mt-3 sm:mt-0 sm:ml-4">
                    <Link to="/">
                        <button
                            type="button"
                            className="inline-flex items-center gap-2 rounded-md bg-white px-6 py-4 text-sm font-semibold shadow-sm ring-gray-300 hover:bg-gray-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gray-300"
                        >
                            <ArrowLeftIcon className="-ml-0.5 h-5 w-5" aria-hidden="true" />
                            Back
                        </button>
                    </Link>
                </div>
            </div>
            {loading ? "loading..." : <TaskTable tasks={data?.quests[0]?.tasks} onToggle={handleClick} />}
            <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={selectedTask}/>
        </PageWrapper>
    );
};