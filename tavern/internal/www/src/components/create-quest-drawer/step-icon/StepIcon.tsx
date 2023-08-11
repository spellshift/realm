import { ListIcon } from "@chakra-ui/react";
import React from "react";
import { AiFillCheckCircle, AiOutlineCheckCircle } from "react-icons/ai";

type StepIconParams = {
    step: number;
    currStep: number;
}
export const StepIcon = (props: StepIconParams) => {
    const {step, currStep} = props;
    
    if(currStep > step){
        return <ListIcon as={AiFillCheckCircle} color="purple.500" />;
    }
    else{
        return <ListIcon as={AiOutlineCheckCircle} color="gray.500" />;
    }
}