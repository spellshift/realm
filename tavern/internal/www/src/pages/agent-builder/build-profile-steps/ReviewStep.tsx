import { TriangleAlert } from "lucide-react";
import { Heading } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { TomeTable } from "../../../components/create-quest-modal/tome-selection/TomeTable";
import { NO_INSTALL_WARNING, TomeRef, TransportConfig } from "../types";
import { TransportList } from "./TransportList";

type ReviewStepProps = {
    name: string;
    description: string;
    buildImage: string;
    transports: TransportConfig[];
    tomes: TomeRef[];
    hasPersistenceTome: boolean;
    isEditing: boolean;
    saving: boolean;
    canSave: boolean;
    onBack: () => void;
    onSave: () => void;
};

export const ReviewStep = ({
    name,
    description,
    buildImage,
    transports,
    tomes,
    hasPersistenceTome,
    isEditing,
    saving,
    canSave,
    onBack,
    onSave,
}: ReviewStepProps) => {
    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Review &amp; save</h2>

            <div className="flex flex-col gap-4">
                <div className="flex flex-col gap-1">
                    <Heading size="sm">Name</Heading>
                    <span className="text-sm text-gray-900">{name}</span>
                </div>
                {description && (
                    <div className="flex flex-col gap-1">
                        <Heading size="sm">Description</Heading>
                        <span className="text-sm text-gray-900">{description}</span>
                    </div>
                )}
                <div className="flex flex-col gap-1">
                    <Heading size="sm">Build image</Heading>
                    <span className="break-all text-sm text-gray-900">{buildImage}</span>
                </div>
            </div>

            <div className="flex flex-col gap-3">
                <Heading size="sm">Transports ({transports.length})</Heading>
                <TransportList transports={transports} showDetails variant="summary" />
            </div>

            <div className="flex flex-col gap-3">
                <Heading size="sm">Bundled tomes ({tomes.length})</Heading>
                {tomes.length === 0
                    ? <span className="text-sm text-gray-400">None</span>
                    : <TomeTable tomeIds={tomes.map((t) => t.tomeId)} />}
            </div>
            {!hasPersistenceTome && (
                <div className="flex flex-row gap-2">
                    <TriangleAlert className="h-5 w-5 text-amber-400 flex-shrink-0" aria-hidden="true" />
                    <p className="text-sm text-amber-700">{NO_INSTALL_WARNING}</p>
                </div>
            )}

            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onBack} disabled={saving} aria-label="back button">Back</Button>
                <Button onClick={onSave} isLoading={saving} disabled={!canSave} aria-label="submit profile">
                    {isEditing ? "Save changes" : "Save profile"}
                </Button>
            </div>
        </div>
    );
};
