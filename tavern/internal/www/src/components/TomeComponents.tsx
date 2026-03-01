import { ReactNode } from "react";
import { TomeNode } from "../utils/interfacesQuery";
import { FieldInputParams } from "../utils/interfacesUI";

type Props = {
    tome: TomeNode;
    params: Array<FieldInputParams>;
    showParamValues?: boolean;
}

type ParamDisplayProps = {
    params: Array<FieldInputParams>;
};

const ParamValuesDisplay = ({ params }: ParamDisplayProps) => (
    <>
        {params.map((paramDef: FieldInputParams) => {
            if (!paramDef.value) return null;
            return (
                <div className="flex flex-row gap-1 text-sm" key={paramDef.name}>
                    <div className="capitalize">{paramDef.name}:</div>
                    <div className="break-all">{paramDef.value}</div>
                </div>
            );
        })}
    </>
);

const ParamLabelsDisplay = ({ params }: ParamDisplayProps) => (
    <div className="flex flex-row flex-wrap gap-1 text-sm">
        Parameters:
        {params.map((element: FieldInputParams, index: number) => (
            <div key={`${index}_${element.name}`}>
                {element.label}{index < (params.length - 1) && ","}
            </div>
        ))}
    </div>
);

const TomeDetails = ({
    tome,
    params,
    showParamValues = true,
}: Props) => {
    const hasParams = params && params.length > 0;
    const hasTactic = tome.tactic && tome.tactic !== "UNSPECIFIED";
    return (
        <div className="flex-1 text-left flex flex-col w-full gap-1">
            <div className="break-all text-base">
                {tome.name}
            </div>
            {tome.description && (
                <div className="line-clamp-2">
                    {tome.description}
                </div>
            )}
            {showParamValues && hasParams && <ParamValuesDisplay params={params} />}
            {!showParamValues && hasParams && <ParamLabelsDisplay params={params} />}
            {hasTactic && (
                <div className="gap-2">
                    Tactic: <span className="lowercase">{tome.tactic}</span>
                </div>
            )}
        </div>
    )
}
export default TomeDetails;