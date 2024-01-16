import { Link } from "react-router-dom";

import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import FreeTextSearch from "../../components/tavern-base-ui/FreeTextSearch";
import { PageNavItem } from "../../utils/enums";
import { QuestTable } from "./components/QuestTable";
import { useQuests } from "./hooks/useQuest";

export const QuestList = () => {
    const {
        hasData,
        data,
        loading,
        error,
        setSearch
    } = useQuests();

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Quest history</h3>
            </div>
            <div className="flex flex-col justify-center items-center">
                {loading ?
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
                            <EmptyState label="No quests found" type={EmptyStateType.noData} details="Get started by creating a new quest." >
                                <Link to="/createQuest">
                                    <button
                                        type="button"
                                        className="inline-flex items-center rounded-md bg-purple-700 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                                    >
                                        Create new quest
                                    </button>
                                </Link>
                            </EmptyState>
                }
            </div>
        </PageWrapper>
    );
}
