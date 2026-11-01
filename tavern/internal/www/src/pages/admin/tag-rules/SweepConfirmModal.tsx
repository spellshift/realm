import { useState } from "react";
import { Checkbox } from "@chakra-ui/react";
import Modal from "../../../components/tavern-base-ui/Modal";
import { TagRuleSweepMode } from "./queries";

interface SweepConfirmModalProps {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
    sweeping: boolean;
    onConfirm: (mode: TagRuleSweepMode) => void;
}

export function sweepModeForOverwrite(overwrite: boolean): TagRuleSweepMode {
    return overwrite ? "overwrite" : "fill-empty";
}

export function SweepConfirmModal({
    isOpen,
    setOpen,
    sweeping,
    onConfirm,
}: SweepConfirmModalProps) {
    const [overwrite, setOverwrite] = useState(false);

    const handleConfirm = () => {
        onConfirm(sweepModeForOverwrite(overwrite));
    };

    return (
        <Modal isOpen={isOpen} setOpen={setOpen} size="sm">
            <div className="flex flex-col gap-4" data-testid="sweep-confirm-modal">
                <h3 className="text-lg font-semibold text-gray-900">
                    Run backfill sweep
                </h3>
                <p className="text-sm text-gray-600">
                    Evaluate all tag rules against all hosts at once. Slots
                    with no matching rule are always left untouched.
                </p>
                <Checkbox
                    colorScheme="purple"
                    isChecked={overwrite}
                    onChange={(event) => setOverwrite(event.target.checked)}
                    data-testid="sweep-overwrite-checkbox"
                >
                    Overwrite Existing Tags
                </Checkbox>
                <p className="text-xs text-gray-500">
                    {overwrite
                        ? "Overwrite replaces the current tag in each winning slot."
                        : "Fill-empty only fills kind slots that have no tag yet."}
                </p>
                <div className="flex flex-row gap-2">
                    <button
                        type="button"
                        onClick={handleConfirm}
                        disabled={sweeping}
                        data-testid="sweep-confirm"
                        className="rounded-md bg-purple-700 px-3 py-1 text-sm text-white disabled:opacity-50"
                    >
                        {sweeping ? "Sweeping…" : "Run sweep"}
                    </button>
                    <button
                        type="button"
                        onClick={() => setOpen(false)}
                        data-testid="sweep-cancel"
                        className="rounded-md border border-gray-300 px-3 py-1 text-sm text-gray-700"
                    >
                        Cancel
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default SweepConfirmModal;
