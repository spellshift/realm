import { useCallback } from "react";
import {
    Textarea,
    FormControl,
    FormLabel,
    FormErrorMessage,
} from "@chakra-ui/react";
import { FieldInputParams } from "../../../utils/interfacesUI";
import { ModalQuestFormikProps } from "../types";

export const ConfigureParams = ({ formik }: {
    formik: ModalQuestFormikProps;
}) => {
    const params = formik.values.params;

    const handleParamChange = useCallback(
        (index: number, value: string) => {
            const allFieldParams = [...formik.values.params];
            allFieldParams[index] = { ...allFieldParams[index], value };
            formik.setFieldValue("params", allFieldParams);
        },
        [formik]
    );

    const handleParamBlur = useCallback(
        (index: number) => {
            formik.setFieldTouched(`params.${index}.value`, true, true);
        },
        [formik]
    );

    const getParamError = (index: number): string | undefined => {
        const errors = formik.errors.params;
        if (!errors) return undefined;

        if (typeof errors === "string") {
            return errors;
        }

        const paramError = errors[index];
        if (!paramError) return undefined;

        if (typeof paramError === "string") {
            return paramError;
        }

        return paramError.value as string | undefined;
    };

    const isParamTouched = (index: number): boolean => {
        const touched = formik.touched.params;
        if (!touched) return false;

        const paramTouched = touched[index];
        if (!paramTouched) return false;

        if (typeof paramTouched === "boolean") {
            return paramTouched;
        }

        return Boolean(paramTouched.value);
    };

    if (params.length === 0) {
        return null;
    }

    return (
        <div className="flex flex-col gap-4">
            <h3 className="text-lg font-semibold text-gray-900">
                Configure parameters
            </h3>
            {params.map((field: FieldInputParams, index: number) => {
                const error = getParamError(index);
                const touched = isParamTouched(index);
                const showError = touched && error;

                return (
                    <FormControl key={field.name} isInvalid={Boolean(showError)}>
                        <FormLabel htmlFor={field.name}>
                            {field.label || field.name}
                            <span className="text-red-500 ml-1">*</span>
                        </FormLabel>
                        <Textarea
                            id={field.name}
                            name={`params.${index}.value`}
                            rows={3}
                            placeholder={
                                field.placeholder || `Enter ${field.label || field.name}`
                            }
                            value={field.value || ""}
                            onChange={(e) => handleParamChange(index, e.target.value)}
                            onBlur={() => handleParamBlur(index)}
                            className="block w-full p-2 placeholder-gray-500 rounded-md border-0 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-purple-600 sm:py-1.5 sm:text-sm sm:leading-6"
                            bg="white"
                        />
                        {showError && <FormErrorMessage>{error}</FormErrorMessage>}
                    </FormControl>
                );
            })}
        </div>
    );
};

export default ConfigureParams;
