import { Textarea } from "@chakra-ui/react";
import { FieldInputParams } from "../../utils/interfacesUI";

type Props = {
    index: number;
    formik: any;
    field: FieldInputParams;
}
export const FormTextArea = (props: Props) => {
    const { index, formik, field } = props;

    const handleChange = (event: any) => {
        const allFieldParams = formik?.values?.params;
        allFieldParams[index].value = event.target.value;
        formik.setFieldValue('params', allFieldParams);
    }

    return (
        <div key={field?.name}>
            <label htmlFor="command" className="block text-base font-semibold text-gray-900">
                {field?.label}
            </label>
            <div className="mt-2">
                <Textarea
                    rows={4}
                    name={field?.name}
                    id={field?.name}
                    className="block w-full p-2 placeholder-gray-500 rounded-md border-0 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:py-1.5 sm:text-sm sm:leading-6"
                    placeholder={field?.placeholder || "Enter tome param"}
                    value={formik?.values?.params[index]?.value || ""}
                    onChange={handleChange}
                />
            </div>
        </div>
    );
}
