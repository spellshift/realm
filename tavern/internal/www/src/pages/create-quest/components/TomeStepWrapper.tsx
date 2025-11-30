import { useQuery } from "@apollo/client";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import TomeStep from "./TomeStep";
import Button from "../../../components/tavern-base-ui/button/Button";
import { FieldInputParams } from "../../../utils/interfacesUI";
import { GET_TOMES_QUERY } from "../../../utils/queries";
import { TomeQueryTopLevel } from "../../../utils/interfacesQuery";
import { QuestFormikProps } from "../types";

type Props = {
    setCurrStep: (step: number) => void;
    formik: QuestFormikProps;
}

const TomeStepWrapper = (props: Props) => {
    const { setCurrStep, formik } = props;
    const { loading, error, data } = useQuery<TomeQueryTopLevel>(GET_TOMES_QUERY);

    const paramsWithValues = formik.values.params.filter((param: FieldInputParams) => {
        return param?.value != null && param?.value !== "";
    });

    const isContinueDisabled = paramsWithValues.length !== formik.values.params.length || formik.values.tome === null;

    const tomeNodes = data?.tomes?.edges?.map(edge => edge.node) || [];

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Select a tome</h2>
            {loading ? (
                <EmptyState type={EmptyStateType.loading} label="Loading tomes..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading tomes..." />
            ) : (
                <TomeStep formik={formik} data={tomeNodes} />
            )}
            <div className="flex flex-row gap-2">
                <Button
                    onClick={() => setCurrStep(0)}
                    buttonVariant="ghost"
                >
                    Back
                </Button>
                <Button
                    onClick={() => setCurrStep(2)}
                    disabled={isContinueDisabled}
                >
                    Continue
                </Button>
            </div>
        </div>
    );
}
export default TomeStepWrapper;
