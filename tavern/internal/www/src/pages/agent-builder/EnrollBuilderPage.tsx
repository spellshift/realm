import { useState } from "react";
import { gql, useMutation } from "@apollo/client";
import { useNavigate } from "react-router-dom";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import Button from "../../components/tavern-base-ui/button/Button";
import AlertError from "../../components/tavern-base-ui/AlertError";
import CodeBlock from "../../components/tavern-base-ui/CodeBlock";

const CREATE_BUILDER_ENROLLMENT_TOKEN = gql`
    mutation CreateBuilderEnrollmentToken {
        createBuilderEnrollmentToken
    }
`;

export const EnrollBuilderPage = () => {
    const navigate = useNavigate();
    const [token, setToken] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [createToken, { loading }] = useMutation<{ createBuilderEnrollmentToken: string }>(CREATE_BUILDER_ENROLLMENT_TOKEN);

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
        <div className="max-w-4xl mx-auto w-full flex flex-col gap-6">
            <Breadcrumbs pages={[
                { label: "Admin", link: "/admin?tab=builders" },
                { label: "Enroll builder", link: "/admin/builders/enroll" },
            ]} />
            <PageHeader title="Enroll builder" description="Generate a one-time token to enroll a builder." />
            {error && <AlertError label="Unable to generate enrollment token" details={error} />}
            {token ? (
                <div className="flex flex-col gap-4">
                    <h2 className="text-xl font-semibold">Start builder</h2>
                    <p className="text-sm text-amber-700">Copy this token now. It will disappear when you leave or refresh this page.</p>
                    <CodeBlock code={token} showCopyButton />
                    <p className="text-sm text-gray-700">From the Realm project root, run this command on the builder host. The token is used on first enrollment and the builder saves its identity locally.</p>
                    <CodeBlock code={`REALM_BUILDER_ENROLLMENT_TOKEN='${quotedToken}' go run ./tavern builder`} language="bash" showCopyButton />
                    <Button buttonStyle={{ color: "purple", size: "md" }} onClick={() => navigate("/admin?tab=builders")}>Done</Button>
                </div>
            ) : (
                <div className="flex flex-col gap-4">
                    <p className="text-sm text-gray-700">Generate a token when you are ready to start the builder. The token can be used once.</p>
                    <Button buttonStyle={{ color: "purple", size: "md" }} isLoading={loading} disabled={loading} onClick={() => { void generate(); }}>Generate</Button>
                </div>
            )}
        </div>
    );
};

export default EnrollBuilderPage;
