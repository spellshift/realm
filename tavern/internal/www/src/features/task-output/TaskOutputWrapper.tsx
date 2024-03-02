import { useQuery } from "@apollo/client";
import React, { FC } from "react";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Modal from "../../components/tavern-base-ui/Modal";
import { GET_TASK_QUERY } from "../../utils/queries";
import { TaskOutput } from "./TaskOutput";

type TaskOutputWrapperProps = {
    isOpen: boolean;
    setOpen: (arg: any) => any;
    selectedTask?: any;
    skipFetch: boolean;
}

export const TaskOutputWrapper: FC<TaskOutputWrapperProps> = ({
    isOpen,
    setOpen,
    selectedTask,
    skipFetch
}) => {
    const { data, loading, error } = useQuery(GET_TASK_QUERY, {
        variables: {
            "where": {
                id: selectedTask.id
            }
        },
        skip: skipFetch
    });

    if (skipFetch) {
        return (
            <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={selectedTask} />
        );
    }

    if (loading) {
        <Modal isOpen={isOpen} setOpen={setOpen}>
            <EmptyState label="Loading task details" type={EmptyStateType.loading} />
        </Modal>
    }

    if (error) {
        <Modal isOpen={isOpen} setOpen={setOpen}>
            <EmptyState label="Error fetching task details" type={EmptyStateType.error} />
        </Modal>
    }

    if (data?.tasks?.edges?.length > 0) {
        const task = data?.tasks?.edges?.[0]?.node;
        <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={task} />
    }

    return (
        <Modal isOpen={isOpen} setOpen={setOpen}>
            <EmptyState label="Error fetching task details" type={EmptyStateType.error} />
        </Modal>
    );

}
