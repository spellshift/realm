import { FormLabel, Heading, Switch } from "@chakra-ui/react";
import React, { useContext, useState } from "react";
import { Link } from "react-router-dom";
import { BeaconFilterBar } from "../../components/beacon-filter-bar";
import { PageWrapper } from "../../components/page-wrapper";
import { TaskOutput } from "../../components/task-output";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { TagContext } from "../../context/TagContext";
import { Task } from "../../utils/consts";
import { PageNavItem } from "../../utils/enums";
import { OutputTable } from "./output-table";
import { SearchOutput } from "./search-output";
import { useOutputResult } from "./useOutputResult";

export const OutputResults = () => {
    const {data: tagData, isLoading: tagDataIsLoading,} = useContext(TagContext);
    const {loading: formattedOutputLoading, tableData, filteredData, setSearch, setTypeFilters, setShowOnlyMyQuests} = useOutputResult();
    const [isOpen, setOpen] = useState(false);
    const [selectedTask, setSelectedTask] = useState<Task | null>(null);


    const handleClick =(e: any) => {
        const selectedTaskData = e?.original?.taskDetails as Task
        setSelectedTask(selectedTaskData);
        setOpen((state)=> !state);
    }

    return (
        <PageWrapper currNavItem={PageNavItem.results}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Quest ouputs </h3>
            </div>
            <div className="flex flex-col justify-center items-center">
                {formattedOutputLoading ?
                    <EmptyState type={EmptyStateType.loading} label="Loading quest results..." />
                : (tableData?.length > 0) ? (
                    <div className="flex flex-col gap-2 w-full">
                        <div className="px-6 py-4 flex flex-row w-full gap-4 items-center">
                            <SearchOutput setSearch={setSearch} />
                            <div className="flex-1">
                                <BeaconFilterBar setFiltersSelected={setTypeFilters}  beacons={tagData?.beacons || []} groups={tagData?.groupTags || []} services={tagData?.serviceTags || []}  />
                            </div>
                            <div className="gap-1">
                                <FormLabel htmlFor="onlyMyQuestst">
                                    <Heading size="sm" >Only my quests</Heading>
                                </FormLabel>
                                <Switch id='onlyMyQuests' onChange={()=> setShowOnlyMyQuests((status: boolean)=> !status)} />
                            </div>
                        </div>
                        {filteredData?.length > 0 ?
                            (<OutputTable outputData={filteredData} onToggle={handleClick} />) : 
                            <EmptyState label="No data matching filters" details="Try adjusting filters or search term." type={EmptyStateType.noData} />
                        }
                    </div>
                ): (
                    <EmptyState label="No data found" details="Tasks may take a few minutes before they have output ready." type={EmptyStateType.noData}>
                        <Link to="/createQuest">
                            <button
                                type="button"
                                className="inline-flex items-center rounded-md bg-purple-700 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                            >
                                Create new quest
                            </button>
                        </Link>
                    </EmptyState>
                )
                }
                <TaskOutput isOpen={isOpen} setOpen={setOpen} selectedTask={selectedTask}/>
            </div>
        </PageWrapper>
    )
}