import * as yup from "yup";

const isFilledParamValue = (value: unknown): boolean => {
    if (typeof value === "string") {
        return value.trim() !== "";
    }
    return value !== null && value !== undefined && value !== "";
};

export const modalQuestSchema = yup.object().shape({
    name: yup
        .string()
        .required("Quest name is required")
        .min(1, "Quest name cannot be empty")
        .max(255, "Quest name must be less than 255 characters"),

    tomeId: yup
        .string()
        .nullable()
        .required("Tome selection is required")
        .test("is-not-null", "Tome selection is required", (value) => value !== null && value !== ""),

    beacons: yup
        .array()
        .of(yup.string())
        .min(1, "At least one beacon must be selected")
        .required("Beacon selection is required"),

    params: yup
        .array()
        .of(
            yup.object().shape({
                name: yup.string().required(),
                value: yup
                    .mixed()
                    .required("Parameter value is required")
                    .test("is-filled", "Parameter value is required", isFilledParamValue),
            })
        )
        .test("all-params-filled", "All parameters must have values", (params) => {
            if (!params || params.length === 0) return true;
            return params.every((param) => isFilledParamValue(param.value));
        }),
});
