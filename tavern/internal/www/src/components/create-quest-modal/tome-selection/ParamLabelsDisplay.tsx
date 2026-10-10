import { FieldInputParams } from "../../../utils/interfacesUI";

interface ParamLabelsDisplayProps {
    params: FieldInputParams[];
}

export const ParamLabelsDisplay = ({ params }: ParamLabelsDisplayProps) => {
    if (!params || params.length === 0) return <span className="text-gray-400">-</span>;
    return (
        <div className="flex flex-row flex-wrap gap-1 text-sm">
            {params.map((element: FieldInputParams, index: number) => (
                <span key={`${index}_${element.name}`}>
                    {element.label || element.name}
                    {index < params.length - 1 && ","}
                </span>
            ))}
        </div>
    );
};
