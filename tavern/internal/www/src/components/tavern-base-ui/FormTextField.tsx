import { FormLabel, Heading, Input, InputProps } from "@chakra-ui/react";

type Props = {
    htmlFor: string;
    label: string;
    required?: boolean;
} & InputProps;
const FormTextField = (props: Props) => {
    const {htmlFor, label, required = false, ...rest} = props;
    return (
        <div>
            <FormLabel htmlFor={htmlFor}>
                <Heading size="sm">{label}{required && <span className="text-red-500 ml-1" aria-hidden="true">*</span>}</Heading>
            </FormLabel>
            <Input
                id={htmlFor}
                colorScheme="purple"
                name={htmlFor}
                required={required}
                {...rest}
                size='sm'
            />
        </div>
    )
}
export default FormTextField;
