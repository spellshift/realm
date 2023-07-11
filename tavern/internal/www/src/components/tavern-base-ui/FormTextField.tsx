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
                name={htmlFor}
                {...rest}
                size='sm'
                className="focus:border-1 focus:border-purple-700"
            />
        </div>
    )
}
export default FormTextField;