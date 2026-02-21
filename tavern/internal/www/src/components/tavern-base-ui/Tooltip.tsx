import { Steps, TooltipProps as ChakraTooltipProps } from "@chakra-ui/react";

import { Tooltip as ChakraTooltip } from '@/components/ui/tooltip';

const Tooltip = ({ label, ...props }: ChakraTooltipProps) => {
    return <ChakraTooltip
        label={label}
        bg="white"
        color="gray.600"
        borderWidth="1px"
        borderColor="gray.100"
        {...props}>
        {props.children}
    </ChakraTooltip>;
}
export default Tooltip;
