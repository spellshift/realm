import { Button } from "@chakra-ui/react";
import { useNavigate } from "react-router-dom";
import EmptyStateNoQuests from "../../components/empty-states/EmptyStateNoQuests";
import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import FreeTextSearch from "../../components/tavern-base-ui/FreeTextSearch";
import { PageNavItem } from "../../utils/enums";
import { QuestTable } from "./components/QuestTable";

export const QuestList = () => {
    const navigate = useNavigate();
    // const {
    //     hasData,
    //     data,
    //     loading,
    //     error,
    //     setSearch
    // } = useQuests();

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <div className="border-b border-gray-200 pb-5 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex-1 flex flex-col gap-2">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Quests</h3>
                    <div className="max-w-2xl text-sm">
                        Quests enable multi-beacon managment by taking a list of beacons and executing a tome with customized parameters against them. A quest is made up of tasks assocaited with a single beacon.
                    </div>
                </div>
                <div>
                    <Button size={"sm"}
                        onClick={() => navigate("/createQuest")}
                    >
                        Create new quest
                    </Button>
                </div>
            </div>
            <div className="flex flex-col justify-center items-center">
                {/* {loading ?
                    <EmptyState type={EmptyStateType.loading} label="Loading quests..." />
                    : error ?
                        <EmptyState type={EmptyStateType.error} label="Error loading quests" />
                        : hasData ?
                            <>
                                <div className="p-4 bg-white rounded-lg shadow-lg mt-2 flex flex-col gap-1 w-full">
                                    <FreeTextSearch setSearch={setSearch} placeholder="Search by quest or tome name" />
                                </div>
                                {data.length > 0 ? (
                                    <div className="py-4 bg-white rounded-lg shadow-lg mt-2 flex flex-col gap-1 w-full">
                                        <QuestTable quests={data} />
                                    </div>
                                ) : (
                                    <EmptyState label="No quests matching search term" type={EmptyStateType.noMatches} />
                                )}
                            </>
                            :
                            <EmptyStateNoQuests />
                } */}
            </div>
        </PageWrapper>
    );
}
