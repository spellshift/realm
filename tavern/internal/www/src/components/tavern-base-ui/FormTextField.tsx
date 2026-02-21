import { Steps, Heading, Input, InputProps, Field } from "@chakra-ui/react";

type Props = {
    htmlFor: string;
    label: string;
} & InputProps;
const FormTextField = (props: Props) => {
    const {htmlFor, label, ...rest} = props;
    return (
        <div>
            <Field.Label htmlFor={htmlFor}>
                <Heading size="sm" >{label}</Heading>
            </Field.Label>
            <Input
                colorPalette="purple"
                name={htmlFor}
                {...rest}
                size='sm'
            />
        </div>
    );
}
export default FormTextField;