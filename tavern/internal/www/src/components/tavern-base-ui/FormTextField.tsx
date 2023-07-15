import { FormLabel, Heading, Input, InputProps } from "@chakra-ui/react";

type Props = {
    htmlFor: string;
    label: string;
} & InputProps;
const FormTextField = (props: Props) => {
    const {htmlFor, label, ...rest} = props;
    return (
        <div>
            <FormLabel htmlFor={htmlFor}>
                <Heading size="sm" >{label}</Heading>
            </FormLabel>
            <Input
                colorScheme="purple"
                name={htmlFor}
                {...rest}
                size='sm'
            />
        </div>
    )
}
export default FormTextField;