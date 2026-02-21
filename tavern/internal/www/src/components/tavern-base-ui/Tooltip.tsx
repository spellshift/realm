import React from 'react';
import { Tooltip as ChakraTooltip, TooltipProps as ChakraTooltipProps } from '@/components/ui/tooltip';

interface Props extends Omit<ChakraTooltipProps, "content"> {
    content?: React.ReactNode;
    label?: React.ReactNode;
}

const Tooltip = ({ label, content, ...props }: Props) => {
    return <ChakraTooltip
        content={label || content || ""}
        contentProps={{
            bg: "white",
            color: "gray.600",
            borderWidth: "1px",
            borderColor: "gray.100",
            ...props.contentProps
        }}
        {...props}>
        {props.children}
    </ChakraTooltip>;
}
export default Tooltip;
