import { gql, useMutation } from "@apollo/client";
import { useCallback, useContext, useEffect, useState } from "react";
import { KindOfTag, TagContextType, TagOptionType, TomeTag } from "../../../../utils/consts";
import { TagContext } from "../../../../context/TagContext";
import { GET_HOST_QUERY, GET_TAG_FILTERS } from "../../../../utils/queries";
import { useToast } from "@chakra-ui/react";

export const useEditableTag = (kind: KindOfTag) => {
    const toast = useToast();
    const { data: allTagData } = useContext(TagContext);
    const [options, setOptions ] = useState<Array<TagOptionType> | undefined>(undefined);
    const [loading, setLoading] = useState(true);
    const [displayEditTag, setDisplayEditTag] = useState(false);
    const [tagValue, setTagValue] = useState<TagOptionType | null>(null);

    const getDefaultTags = useCallback( (kind: KindOfTag, allTagData?: TagContextType)=> {
        switch (kind) {
            case 'group':
                return formatTagOptions(allTagData?.groupTags || []);
            case 'service':
                return formatTagOptions(allTagData?.serviceTags || []);
            default:
                return [];
        }
    },[]);

    useEffect(()=>{
        if(allTagData && !options){
            setOptions(getDefaultTags(kind, allTagData));
            setLoading(false);
        }
    },[allTagData, kind, options, getDefaultTags]);

    function formatTagOptions(tags: Array<TomeTag>){
        return tags.map(function (tag: TomeTag) {
            return {
                ...tag,
                value: tag?.id,
                label: tag?.name,
                kind: tag?.kind
            };
        });
    };

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

    const handleCreateOption = async (inputValue: string | null, hostId?: string, previousTag?: TomeTag) => {
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

    const handleSelectOption = async (selectedTag: TagOptionType | null, hostId?: string, previousTag?: TomeTag) => {
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
        options: options || [],
        loading,
        displayEditTag,
        handleSelectOption,
        handleCreateOption,
        setDisplayEditTag
    }

}
