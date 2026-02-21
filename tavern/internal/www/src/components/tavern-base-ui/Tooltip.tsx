import { Steps } from "@chakra-ui/react";

import { Tooltip as ChakraTooltip } from '../ui/tooltip';

const Tooltip = ({ label, ...props }: any) => {
    return <ChakraTooltip
        content={label}
        contentProps={{
            bg: "white",
            color: "gray.600",
            borderWidth: "1px",
            borderColor: "gray.100"
        }}
        {...props}>
        {props.children}
    </ChakraTooltip>;
}
export default Tooltip;
