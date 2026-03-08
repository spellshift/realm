import { gql, useMutation } from "@apollo/client";
import { useState } from "react";
import { GET_HOST_QUERY } from "../../../../utils/queries";
import { useToast } from "@chakra-ui/react";
import { FilterBarOption, KindOfTag } from "../../../../utils/interfacesUI";
import { GET_TAG_OPTIONS } from "./useTagOptions";

export const useEditableTag = (kind: KindOfTag) => {
    const toast = useToast();
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
            GET_TAG_OPTIONS
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
            toast({
                title: "Error Creating Tag",
                description: `${error}`,
                status: "error",
                duration: 3000,
                isClosable: true,
                position: "bottom-right",
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
                  ...previousTag ? {"removeTagIDs": [previousTag?.id]} : {},
                  ...selectedTag ? {"addTagIDs": [selectedTag.id]} : {}
                }
            }
        };
        try{
            const response = await updateHostMutation(formatVariables);
            if(response.data){
                toast({
                    title: "Success",
                    description: "Tag has been updated successfully.",
                    status: "success",
                    duration: 3000,
                    isClosable: true,
                    position: "bottom-right",
                });
                setTagValue(selectedTag);
                setDisplayEditTag(false);
            }
        }
        catch(error){
            toast({
                title: "Error Updating Tag",
                description: `${error}`,
                status: "error",
                duration: 3000,
                isClosable: true,
                position: "bottom-right",
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
