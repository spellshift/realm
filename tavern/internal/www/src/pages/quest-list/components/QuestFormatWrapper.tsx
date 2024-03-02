import { FC } from "react";
import TablePagination from "../../../components/tavern-base-ui/TablePagination";
import { PaginationPageInfo, QuestProps } from "../../../utils/consts";
import { TableRowLimit } from "../../../utils/enums";
import { useFormatQuests } from "../hooks/useFormatQuests";
import { QuestTable } from "./QuestTable";

type QuestFormatWrapperProps = {
    totalCount: number;
    pageInfo: PaginationPageInfo;
    data: Array<QuestProps>;
    page: number;
    setPage: (num: number) => void
    updateQuestList: (afterCursor?: string | undefined, beforeCursor?: string | undefined) => void
};

const QuestFormatWrapper: FC<QuestFormatWrapperProps> = ({
    data,
    totalCount,
    pageInfo,
    page,
    setPage,
    updateQuestList
}) => {
    const { loading: formatLoading, data: formattedData } = useFormatQuests(data);
    console.log(formattedData);
    return (
        <div className="py-4 bg-white rounded-lg shadow-lg mt-2 flex flex-col gap-1 w-full">
            <QuestTable quests={formattedData} />
            <TablePagination totalCount={totalCount} pageInfo={pageInfo} refetchTable={updateQuestList} page={page} setPage={setPage} rowLimit={TableRowLimit.QuestRowLimit} />
        </div>
    );
}
export default QuestFormatWrapper;
