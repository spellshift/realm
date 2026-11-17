import { FC, useState } from "react";
import { gql, useMutation } from "@apollo/client";
import { Heading } from "@chakra-ui/react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
import { FormSteps } from "../../../components/form-steps";

const CREATE_BUILDER_ENROLLMENT_TOKEN = gql`
    mutation CreateBuilderEnrollmentToken {
        createBuilderEnrollmentToken
    }
`;

const STEPS = [
    { name: "Generate token", description: "Step 1", href: "#", step: 0 },
    { name: "Start builder", description: "Step 2", href: "#", step: 1 },
];

type EnrollBuilderModalProps = {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
};

/**
 * Slide-out for enrolling a builder. The token lives only in the inner
 * component's state: closing the slide-out unmounts it, so the one-time
 * token is gone the next time it opens.
 */
const EnrollBuilder = ({ onClose }: { onClose: () => void }) => {
    const [token, setToken] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [createToken, { loading }] = useMutation<{ createBuilderEnrollmentToken: string }>(CREATE_BUILDER_ENROLLMENT_TOKEN);

    const currStep = token ? 1 : 0;

    const generate = async () => {
        if (loading || token) return;
        setError(null);
        try {
            const { data } = await createToken();
            if (!data?.createBuilderEnrollmentToken) throw new Error("The server did not return an enrollment token.");
            setToken(data.createBuilderEnrollmentToken);
        } catch (caught) {
            setError(caught instanceof Error ? caught.message : String(caught));
        }
    };

    const quotedToken = token?.replace(/'/g, "'\\''");

    return (
        <div className="flex flex-col gap-4 w-full min-w-0 pb-6" data-testid="enroll-builder-modal">
            <div className="flex flex-col gap-1">
                <Heading size="sm">Enroll builder</Heading>
                <p className="text-sm text-gray-500">Generate a one-time token to enroll a builder.</p>
            </div>
            {error && <AlertError label="Unable to generate enrollment token" details={error} />}
            <div className="flex flex-row gap-6">
                <div className="flex-shrink-0 mt-2 lg:block hidden">
                    <FormSteps steps={STEPS} currStep={currStep} />
                </div>
                <div className="flex-1 min-w-0">
                    {token ? (
                        <div className="flex flex-col gap-4">
                            <h2 className="text-xl font-semibold">Start builder</h2>
                            <p className="text-sm text-amber-700">Copy this token now. It will disappear when you close this panel.</p>
                            <CodeBlock code={token} showCopyButton />
                            <p className="text-sm text-gray-700">From the Realm project root, run this command on the builder host. The token is used on first enrollment and the builder saves its identity locally.</p>
                            <CodeBlock code={`REALM_BUILDER_ENROLLMENT_TOKEN='${quotedToken}' go run ./tavern builder`} language="bash" showCopyButton />
                            <div className="flex flex-row gap-2">
                                <Button buttonStyle={{ color: "purple", size: "md" }} onClick={onClose}>Done</Button>
                            </div>
                        </div>
                    ) : (
                        <div className="flex flex-col gap-4">
                            <p className="text-sm text-gray-700">Generate a token when you are ready to start the builder. The token can be used once.</p>
                            <div className="flex flex-row gap-2">
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "md" }}
                                    onClick={onClose}
                                    disabled={loading}
                                    aria-label="cancel enrollment"
                                >
                                    Cancel
                                </Button>
                                <Button buttonStyle={{ color: "purple", size: "md" }} isLoading={loading} disabled={loading} onClick={() => { void generate(); }}>Generate</Button>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export const EnrollBuilderModal: FC<EnrollBuilderModalProps> = ({ isOpen, setOpen }) => (
    <Modal isOpen={isOpen} setOpen={setOpen} size="lg">
        <EnrollBuilder onClose={() => setOpen(false)} />
    </Modal>
);

export default EnrollBuilderModal;
