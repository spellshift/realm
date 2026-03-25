import { FC } from "react";
import { BookOpenIcon } from "@heroicons/react/24/outline";
import { TomeNode } from "../../../utils/interfacesQuery";
import { TomeTactic } from "../../../utils/enums";

interface TomeFieldsProps {
    tome: TomeNode;
}

const TomeFields: FC<TomeFieldsProps> = ({ tome }) => {
    const hasTactic = tome.tactic && tome.tactic !== "UNSPECIFIED";
    const tacticLabel = hasTactic
        ? TomeTactic[tome.tactic as keyof typeof TomeTactic]
        : null;

    return (
        <div className="text-sm flex flex-row gap-2 items-center">
            <BookOpenIcon fill="currentColor" className="w-4 h-4" />
            <span>{tome.name}</span>
            {tacticLabel && (
                <>
                    <span>|</span>
                    <span>{tacticLabel}</span>
                </>
            )}
        </div>
    );
};

export default TomeFields;
