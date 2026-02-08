import { useState } from "react";
import { useFormik } from "formik";
import * as yup from "yup";
import { format, add } from "date-fns";
import { useCreateLink } from "../useAssets";
import { generateRandomString } from "../../../utils/random";

interface UseCreateLinkLogicProps {
    assetId: string;
    onSuccess?: () => void;
    setOpen: (open: boolean) => void;
}

export const useCreateLinkLogic = ({ assetId, onSuccess, setOpen }: UseCreateLinkLogicProps) => {
    const { createLink, loading } = useCreateLink();
    const [createdLink, setCreatedLink] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const formik = useFormik({
        initialValues: {
            downloadLimit: 1,
            hasDownloadLimit: false,
            expiryMode: 0, // 0: 10m, 1: 1h, 2: Custom
            expiresAt: format(add(new Date(), { days: 1 }), "yyyy-MM-dd'T'HH:mm"),
            path: generateRandomString(12),
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

    const handleCopy = () => {
        if (createdLink) {
            navigator.clipboard.writeText(createdLink);
        }
    };

    const handleClose = () => {
        setCreatedLink(null);
        setOpen(false);
    };

    const handleCopyAndClose = () => {
        handleCopy();
        handleClose();
    }

    return {
        formik,
        createdLink,
        error,
        loading,
        handleClose,
        handleCopyAndClose
    };
};
