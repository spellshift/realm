import { gql, useMutation } from "@apollo/client";
import { useState } from "react";
import { GET_HOST_QUERY, GET_TAG_FILTERS } from "../../../../utils/queries";
import { toaster } from "@/components/ui/toaster";
import { FilterBarOption, KindOfTag } from "../../../../utils/interfacesUI";

export const useEditableTag = (kind: KindOfTag) => {
    const [loading, setLoading] = useState(false);
    const [displayEditTag, setDisplayEditTag] = useState(false);
    const [tagValue, setTagValue] = useState<FilterBarOption | null>(null);

    const CREATE_TAG_MUTATION = gql`
        mutation CreateTag($input: CreateTagInput!){
            createTag(input: $input){
                id
                name,
                kind,
            }
        }
    `;

    const [createTagMutation] = useMutation(CREATE_TAG_MUTATION);

    const UPDATE_HOST_TAGS = gql`
        mutation UpdateHostTags($hostID: ID!, $input: UpdateHostInput!){
                updateHost(hostID: $hostID, input: $input){
                id
            }
        }
    `;

    const [updateHostMutation] = useMutation(UPDATE_HOST_TAGS, {
        refetchQueries: [
            GET_HOST_QUERY,
            GET_TAG_FILTERS,
        ],
      });

    const handleCreateOption = async (inputValue: string | null, hostId?: string, previousTag?: FilterBarOption) => {
        if(!inputValue){
            return
        }
        setLoading(true);

        const formatVariables = {
            "variables": {
                "input": {
                "name": inputValue,
                "kind": kind
                }
            }
        };

        try{
            const response = await createTagMutation(formatVariables);
            if(response.data){
                const tag = {
                    value: response.data.createTag.id,
                    label: response.data.createTag.name,
                    ...response.data.createTag
                }
                handleSelectOption(tag, hostId, previousTag);
            }
        }
        catch(error){
            toaster.create({
                title: "Error Creating Tag",
                description: `${error}`,
                type: "error",
                duration: 3000,
                closable: true,
            });
        }

        setLoading(false);
    };

    const handleSelectOption = async (selectedTag: FilterBarOption  | null, hostId?: string, previousTag?: FilterBarOption) => {
        setLoading(true);

        const formatVariables = {
            "variables": {
                "hostID": hostId,
                "input": {
                  ...(previousTag ? {"removeTagIDs": [previousTag?.id]} : {}),
                  ...(selectedTag ? {"addTagIDs": [selectedTag.id]} : {})
                }
            }
        };
        try{
            const response = await updateHostMutation(formatVariables);
            if(response.data){
                toaster.create({
                    title: "Success",
                    description: "Tag has been updated successfully.",
                    type: "success",
                    duration: 3000,
                    closable: true,
                });
                setTagValue(selectedTag);
                setDisplayEditTag(false);
            }
        }
        catch(error){
            toaster.create({
                title: "Error Updating Tag",
                description: `${error}`,
                type: "error",
                duration: 3000,
                closable: true,
            });
        }

        setLoading(false);
    };

    return {
        tagValue,
        loading,
        displayEditTag,
        handleSelectOption,
        handleCreateOption,
        setDisplayEditTag
    }

}
