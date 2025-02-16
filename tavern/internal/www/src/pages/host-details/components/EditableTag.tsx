import { PencilSquareIcon } from "@heroicons/react/20/solid";
import { useContext, useState } from "react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { AuthorizationContext } from "../../../context/AuthorizationContext";
import { TomeTag } from "../../../utils/consts";

export default function EditableTag({ tag }: { tag?: TomeTag }) {
    const [displayEditTag, setDisplayEditTag] = useState(false);
    const { data } = useContext(AuthorizationContext);
    const canEdit = data?.me?.isAdmin || false;

    return (
        <div className="text-gray-600 text-sm ml-6 min-h-[38px] flex flex-col justify-center">
            <div className="flex flex-row gap-2">
                {tag?.name}
                {canEdit &&
                    <Button
                        buttonVariant="ghost"
                        className="p-0"
                        leftIcon={<PencilSquareIcon className="w-4" />}
                        buttonStyle={{ color: "gray", size: "md" }}
                        aria-label="Edit group tag"
                        onClick={() => setDisplayEditTag(true)}
                    />
                }
            </div>
        </div>
    );
}
