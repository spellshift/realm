import { PencilSquareIcon } from "@heroicons/react/20/solid";
import Button from "../../../../components/tavern-base-ui/button/Button";
import { useAuthorization } from "../../../../context/AuthorizationContext";
import CreatableSelect from "react-select/creatable";
import { useEditableTag } from "./useEditableTag";
import { SingleValue } from "react-select";
import { TagNode } from "../../../../utils/interfacesQuery";
import { FilterBarOption, KindOfTag } from "../../../../utils/interfacesUI";


export default function EditableTag({ kind, tagSaved, hostId }: { kind: KindOfTag, tagSaved?: TagNode, hostId?: string }) {
    const { data } = useAuthorization();
    const canEdit = data?.me?.isAdmin || false;
    const {
        tagValue,
        options,
        loading,
        displayEditTag,
        handleSelectOption,
        handleCreateOption,
        setDisplayEditTag
    } = useEditableTag(kind);

    if (displayEditTag) {
        return (
            <div className="ml-6">
                <CreatableSelect
                    isClearable
                    isDisabled={loading}
                    isLoading={loading}
                    onChange={(newValue: SingleValue<FilterBarOption>) => handleSelectOption(newValue, hostId, tagSaved)}
                    onCreateOption={(inputValue: string) => handleCreateOption(inputValue, hostId, tagSaved)}
                    options={options}
                    value={tagValue}
                />
            </div>
        )
    }

    return (
        <div className="text-gray-600 text-sm ml-6 min-h-[38px] flex flex-col justify-center">
            <div>
                {canEdit ?
                    <Button
                        buttonVariant="ghost"
                        className="-ml-2 px-2 py-1 font-normal"
                        rightIcon={<PencilSquareIcon className="w-4" />}
                        buttonStyle={{ color: "gray", size: "md" }}
                        aria-label="Edit group tag"
                        onClick={() => setDisplayEditTag(true)}
                    >
                        {tagSaved?.name}
                    </Button>
                    : tagSaved?.name
                }
            </div>
        </div>
    );
}
