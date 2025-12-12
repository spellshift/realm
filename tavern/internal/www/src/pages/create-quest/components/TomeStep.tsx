import { Heading } from "@chakra-ui/react";
import { FormTextArea } from "../../../components/form-text-area";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { Tome, TomeParams } from "../../../utils/consts";
import { safelyJsonParse } from "../../../utils/utils";
import TomeRadioGroup from "./TomeRadioGroup";

type Props = {
    formik: any;
    data: Array<any>
}
const TomeStep = (props: Props) => {
    const { formik, data } = props;

    const handleSelectTome = (tome: Tome) => {
        const { params } = safelyJsonParse(tome?.paramDefs);
        formik.setFieldValue('tome', tome);
        formik.setFieldValue('params', params ? params : []);
    };

    if (data.length < 1) {
        // TODO Link to github upload a tome in empty state when that is ready
        return (
            <div className="flex flex-col gap-3">
                <Heading size="sm" >Select a tome</Heading>
                <EmptyState type={EmptyStateType.noData} label="No tome data found." />
            </div>
        )
    }

    return (
        <div className="flex flex-col gap-6">
            <TomeRadioGroup
                label="Select a tome"
                data={data}
                selected={formik?.values?.tome}
                setSelected={handleSelectTome}
            />
            {formik?.values?.params.length > 0 && formik?.values?.params.map((field: TomeParams, index: number) => {
                return (
                    <FormTextArea
                        key={field.name}
                        field={field}
                        index={index}
                        formik={formik}
                    />
                );
            })}
        </div>
    );
}
export default TomeStep;
