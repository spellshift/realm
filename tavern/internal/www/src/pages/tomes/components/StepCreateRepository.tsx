import { useFormik } from "formik";
import { FC } from "react";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import { RepositoryType } from "../../../utils/consts";
import { useCreateRepositoryLink } from "../hooks/useCreateRepositoryLink";

type StepCreateRepositoryProps = {
    setCurrStep: (step: number) => void;
    setNewRepository: (repository: RepositoryType) => void;
}

const StepCreateRepository: FC<StepCreateRepositoryProps> = ({ setCurrStep, setNewRepository }) => {
    const { submitRepositoryLink, error } = useCreateRepositoryLink(setCurrStep, setNewRepository);

    const formik = useFormik({
        initialValues: {
            url: ""
        },
        onSubmit: (values: any) => submitRepositoryLink(values),
    });

    return (
        <form className="flex flex-col gap-6">
            <div>
                <h3 className="font-bold text-lg">Link repository</h3>
                <p className="text-sm">
                    Provide a valid ssh path to a remote repository that contains tomes. <a className="external-link" target="_blank" href="https://docs.realm.pub/user-guide/golem#creating-and-testing-tomes">Learn more</a> about how to structure a tome repository.
                </p>
            </div>
            {error !== "" && (
                <AlertError label={"Error saving link"} details={error} />
            )}
            <FormTextField
                htmlFor="url"
                label="Repository ssh path"
                placeholder="ssh://github.com/repo_name"
                value={formik?.values?.url || ""}
                onChange={(event) => formik.setFieldValue('url', event?.target?.value)}
            />
            <div>
                <button
                    className="btn-primary"
                    onClick={(event) => {
                        event.preventDefault();
                        formik.handleSubmit();
                    }}
                    disabled={formik?.values?.url === ""}
                    type="submit"
                >
                    Save link
                </button>
            </div>
        </form>
    );
}
export default StepCreateRepository;
