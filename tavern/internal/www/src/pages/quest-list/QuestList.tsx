import { useQuery } from "@apollo/client";
import { Link } from "react-router-dom";

import { PageWrapper } from "../../components/page-wrapper";
import { GET_QUEST_QUERY } from "../../utils/queries";
import { QuestTable } from "./quest-table";

export const QuestList = () => {
    const { loading, data } = useQuery(GET_QUEST_QUERY);

    return (
        <PageWrapper>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">Quests</h3>
                <div className="mt-3 sm:mt-0 sm:ml-4">
                    <Link to="/createQuest">
                        <button
                            type="button"
                            className="inline-flex items-center rounded-md bg-purple-700 px-6 py-4 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                        >
                            Create new quest
                        </button>
                    </Link>
                </div>
            </div>
            <div className="flex flex-col justify-center items-center">
                {loading && 
                    <div className="py-2 px-4 ">
                        <svg className="animate-spin h-5 w-5 mr-3 ..." viewBox="0 0 24 24"/>
                            loading...
                    </div>
                }
                {data?.quests?.length > 0 &&
                    <QuestTable quests={data?.quests || []} />
                }
            </div>
        </PageWrapper>
    );
}