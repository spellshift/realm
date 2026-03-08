import { FC, useState } from "react";
import Modal from "../../../../components/tavern-base-ui/Modal";
import AlertError from "../../../../components/tavern-base-ui/AlertError";
import { CREATE_LINK } from "../../queries";
import { format, add } from "date-fns";
import * as yup from "yup";
import { useFormik } from "formik";
import LinkCreated from "./LinkCreated";
import CreateLinkForm from "./CreateLinkForm";
import { generateRandomLinkPath } from "../../utils";
import { useMutation } from "@apollo/client";

type CreateLinkModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    assetId: string;
    assetName: string;
    onSuccess?: () => void;
};

const CreateLinkModal: FC<CreateLinkModalProps> = ({ isOpen, setOpen, assetId, assetName, onSuccess }) => {
    const [createLink] = useMutation(CREATE_LINK);
    const [createdLink, setCreatedLink] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const formik = useFormik({
        initialValues: {
            downloadLimit: 1,
            hasDownloadLimit: false,
            expiryMode: 0, // 0: 10m, 1: 1h, 2: Custom
            expiresAt: format(add(new Date(), { days: 1 }), "yyyy-MM-dd'T'HH:mm"),
            path: generateRandomLinkPath(),
        },
        validationSchema: yup.object({
            path: yup.string().required("Path is required"),
            downloadLimit: yup.number().when("hasDownloadLimit", {
                is: true,
                then: (schema) => schema.min(1, "Download limit must be at least 1").required("Download limit is required"),
            }),
            expiresAt: yup.string().when("expiryMode", {
                is: 2,
                then: (schema) => schema.required("Expiry date is required")
                    .test("is-future", "Expiry date must be in the future", (value) => {
                        if (!value) return false;
                        return new Date(value) > new Date();
                    }),
            }),
        }),
        onSubmit: async (values) => {
            setError(null);
            let finalExpiresAt = new Date();
            if (values.expiryMode === 0) {
                finalExpiresAt = add(new Date(), { minutes: 10 });
            } else if (values.expiryMode === 1) {
                finalExpiresAt = add(new Date(), { hours: 1 });
            } else {
                finalExpiresAt = new Date(values.expiresAt);
            }

            try {
                const { data } = await createLink({
                    variables: {
                        input: {
                            assetID: assetId,
                            downloadLimit: values.hasDownloadLimit ? Number(values.downloadLimit) : null,
                            expiresAt: finalExpiresAt.toISOString(),
                            path: values.path,
                        },
                    },
                });

                if (data?.createLink?.path) {
                    const link = `${window.location.origin}/cdn/${data.createLink.path}`;
                    setCreatedLink(link);
                    if (onSuccess) onSuccess();
                } else {
                    throw new Error("Failed to create link: no path returned");
                }
            } catch (err: any) {
                console.error(err);
                setError(err.message || "An unknown error occurred");
            }
        },
    });

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="md">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Create link for {assetName}
                    </h3>
                    <p className="mt-1 text-sm text-gray-500 font-normal">
                        Create a temporary download link for this asset.
                    </p>
                </div>

                {error && <AlertError label={error} details={""} />}

                {createdLink
                    ? <LinkCreated createdLink={createdLink} close={() => setOpen(false)} />
                    : <CreateLinkForm formik={formik} setOpen={setOpen} />}
            </div>
        </Modal>
    );
};

export default CreateLinkModal;
