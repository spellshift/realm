import { FC } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { Clipboard } from "lucide-react";
import { Checkbox, Tabs, TabList, TabPanels, Tab, TabPanel } from "@chakra-ui/react";
import { useCreateLinkLogic } from "../hooks/useCreateLinkLogic";

type CreateLinkModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    assetId: string;
    assetName: string;
    onSuccess?: () => void;
};

const CreateLinkModal: FC<CreateLinkModalProps> = ({ isOpen, setOpen, assetId, assetName, onSuccess }) => {
    const { formik, createdLink, error, loading, handleClose, handleCopyAndClose } = useCreateLinkLogic({ assetId, onSuccess, setOpen });

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
                        <div className="flex flex-col gap-2">
                            <p className="text-sm text-gray-700 break-all select-all">
                                {createdLink}
                            </p>
                        </div>
                        <div className="flex justify-end gap-2 mt-2">
                            <Button
                                onClick={handleClose}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "md" }}
                            >
                                Close
                            </Button>
                            <Button
                                onClick={handleCopyAndClose}
                                buttonVariant="solid"
                                buttonStyle={{ color: "purple", size: "md" }}
                                leftIcon={<Clipboard className="h-5 w-5" />}
                            >
                                Copy & Close
                            </Button>
                        </div>
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
                            <label htmlFor="path" className="block text-sm font-medium text-gray-700">
                                Path
                            </label>
                            <div className="flex rounded-md shadow-sm mt-1">
                                <span className="inline-flex items-center rounded-l-md border border-r-0 border-gray-300 bg-gray-50 px-3 text-gray-500 sm:text-sm">
                                    /cdn/
                                </span>
                                <input
                                    type="text"
                                    id="path"
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
