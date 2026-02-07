import { FC, useState, useEffect } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { useCreateLink } from "../useAssets";
import { format, add } from "date-fns";
import { Clipboard } from "lucide-react";
import { Checkbox, Tabs, TabList, TabPanels, Tab, TabPanel } from "@chakra-ui/react";
import * as yup from "yup";
import { useFormik } from "formik";

type CreateLinkModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    assetId: string;
    assetName: string;
    onSuccess?: () => void;
};

const generateRandomString = (length: number) => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    const randomValues = new Uint32Array(length);
    crypto.getRandomValues(randomValues);
    for (let i = 0; i < length; i++) {
        result += chars[randomValues[i] % chars.length];
    }
    return result;
};

const CreateLinkModal: FC<CreateLinkModalProps> = ({ isOpen, setOpen, assetId, assetName, onSuccess }) => {
    const { createLink, loading } = useCreateLink();
    const [createdLink, setCreatedLink] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const formik = useFormik({
        initialValues: {
            downloadLimit: 1,
            hasDownloadLimit: false,
            expiryMode: 0, // 0: 10m, 1: 1h, 2: Custom
            expiresAt: format(add(new Date(), { days: 1 }), "yyyy-MM-dd'T'HH:mm"),
            path: "",
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

    useEffect(() => {
        if (isOpen) {
            setCreatedLink(null);
            setError(null);
            formik.setValues({
                downloadLimit: 1,
                hasDownloadLimit: false,
                expiryMode: 0,
                expiresAt: format(add(new Date(), { days: 1 }), "yyyy-MM-dd'T'HH:mm"),
                path: generateRandomString(12),
            });
            formik.setTouched({});
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [isOpen]);

    const handleCopy = () => {
        if (createdLink) {
            navigator.clipboard.writeText(createdLink);
        }
    };

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

                {createdLink ? (
                    <div className="flex flex-col gap-4">
                        <h4 className="font-medium text-gray-900">Link Created!</h4>
                        <div className="flex flex-row items-center gap-2">
                            <input
                                type="text"
                                readOnly
                                value={createdLink}
                                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            />
                            <Button
                                onClick={handleCopy}
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "sm" }}
                                leftIcon={<Clipboard className="h-5 w-5" />}
                            >
                                Copy
                            </Button>
                        </div>
                        <Button
                            onClick={() => {
                                setCreatedLink(null);
                                setOpen(false);
                            }}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                        >
                            Done
                        </Button>
                    </div>
                ) : (
                    <form onSubmit={formik.handleSubmit} className="flex flex-col gap-4">
                        <div>
                            <div className="flex items-center gap-2 mb-2">
                                <Checkbox
                                    isChecked={formik.values.hasDownloadLimit}
                                    onChange={(e) => formik.setFieldValue("hasDownloadLimit", e.target.checked)}
                                    colorScheme="purple"
                                >
                                    <span className="text-sm font-medium text-gray-700">Limit Downloads</span>
                                </Checkbox>
                            </div>

                            {formik.values.hasDownloadLimit && (
                                <div>
                                    <input
                                        type="number"
                                        min="1"
                                        name="downloadLimit"
                                        required
                                        value={formik.values.downloadLimit}
                                        onChange={formik.handleChange}
                                        onBlur={formik.handleBlur}
                                        className={`mt-1 block w-full rounded-md shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border ${
                                            formik.touched.downloadLimit && formik.errors.downloadLimit ? "border-red-500" : "border-gray-300"
                                        }`}
                                    />
                                    {formik.touched.downloadLimit && formik.errors.downloadLimit && (
                                        <p className="mt-1 text-sm text-red-600">{formik.errors.downloadLimit}</p>
                                    )}
                                </div>
                            )}
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-700 mb-2">
                                Expires In
                            </label>
                            <Tabs
                                index={formik.values.expiryMode}
                                onChange={(index) => formik.setFieldValue("expiryMode", index)}
                                variant="soft-rounded"
                                colorScheme="purple"
                                size="sm"
                            >
                                <TabList>
                                    <Tab>10min</Tab>
                                    <Tab>1hr</Tab>
                                    <Tab>Custom</Tab>
                                </TabList>
                                <TabPanels>
                                    <TabPanel p={0} pt={2}>
                                        <p className="text-sm text-gray-500">Link will expire in 10 minutes.</p>
                                    </TabPanel>
                                    <TabPanel p={0} pt={2}>
                                        <p className="text-sm text-gray-500">Link will expire in 1 hour.</p>
                                    </TabPanel>
                                    <TabPanel p={0} pt={2}>
                                        <input
                                            type="datetime-local"
                                            name="expiresAt"
                                            required={formik.values.expiryMode === 2}
                                            value={formik.values.expiresAt}
                                            onChange={formik.handleChange}
                                            onBlur={formik.handleBlur}
                                            className={`block w-full rounded-md shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border ${
                                                formik.touched.expiresAt && formik.errors.expiresAt ? "border-red-500" : "border-gray-300"
                                            }`}
                                        />
                                        {formik.touched.expiresAt && formik.errors.expiresAt && (
                                            <p className="mt-1 text-sm text-red-600">{formik.errors.expiresAt}</p>
                                        )}
                                    </TabPanel>
                                </TabPanels>
                            </Tabs>
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-700">
                                Path
                            </label>
                            <div className="flex rounded-md shadow-sm mt-1">
                                <span className="inline-flex items-center rounded-l-md border border-r-0 border-gray-300 bg-gray-50 px-3 text-gray-500 sm:text-sm">
                                    /cdn/
                                </span>
                                <input
                                    type="text"
                                    name="path"
                                    required
                                    value={formik.values.path}
                                    onChange={formik.handleChange}
                                    onBlur={formik.handleBlur}
                                    className={`block w-full min-w-0 flex-1 rounded-none rounded-r-md focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border ${
                                        formik.touched.path && formik.errors.path ? "border-red-500" : "border-gray-300"
                                    }`}
                                />
                            </div>
                            {formik.touched.path && formik.errors.path && (
                                <p className="mt-1 text-sm text-red-600">{formik.errors.path}</p>
                            )}
                        </div>

                        <div className="flex justify-end gap-2 mt-4">
                            <Button
                                type="button"
                                onClick={() => setOpen(false)}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "md" }}
                            >
                                Cancel
                            </Button>
                            <Button
                                type="submit"
                                disabled={loading}
                                buttonVariant="solid"
                                buttonStyle={{ color: "purple", size: "md" }}
                            >
                                {loading ? "Creating..." : "Create Link"}
                            </Button>
                        </div>
                    </form>
                )}
            </div>
        </Modal>
    );
};

export default CreateLinkModal;
