import { FC } from "react";
import { QuestProps } from "../../../utils/consts";

type QuestFormatWrapperProps = {
    data: Array<QuestProps>;
};

const QuestFormatWrapper: FC<QuestFormatWrapperProps> = ({ data }) => {
    return (
        <div>
            Here in wrapper
        </div>
    );
}
export default QuestFormatWrapper;
