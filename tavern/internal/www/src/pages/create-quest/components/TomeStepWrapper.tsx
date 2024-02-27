import { gql, useQuery } from "@apollo/client";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { TomeParams } from "../../../utils/consts";
import TomeStep from "./TomeStep";

const GET_TOMES = gql`
    query get_tomes{
        tomes {
            id
            name
            paramDefs
            description
            eldritch
            tactic
            supportModel
        }
    }
`;

type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
const TomeStepWrapper = (
    props: Props
) => {
    const { setCurrStep, formik } = props;
    const { loading, error, data } = useQuery(GET_TOMES);

    const hasAllParamsSet = formik?.values?.params.filter((param: TomeParams) => {
        return param?.value && param?.value !== "";
    });

    const isContinueDisabled = hasAllParamsSet.length !== formik?.values?.params.length || formik?.values?.tome === null;

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Select a tome</h2>
            {loading ? (
                <EmptyState type={EmptyStateType.loading} label="Loading tomes..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error loading tomes..." />
            ) : (
                <TomeStep formik={formik} data={data?.tomes || []} />
            )}
            <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={() => setCurrStep(0)}
                >
                    Back
                </button>
                <button
                    className="btn-primary"
                    onClick={() => setCurrStep(2)}
                    disabled={isContinueDisabled}
                >
                    Continue
                </button>
            </div>
        </div>
    );
}
export default TomeStepWrapper;
