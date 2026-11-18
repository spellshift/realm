import { gql, useMutation, useQuery } from "@apollo/client";
import { Switch } from "@chakra-ui/react";
import AlertError from "../../components/tavern-base-ui/AlertError";

export const GET_REDIRECTORS = gql`
    query AdminRedirectors {
        redirectors { id domains enabled online }
    }
`;

export const SET_REDIRECTOR_ENABLED = gql`
    mutation SetRedirectorEnabled($redirectorID: ID!, $enabled: Boolean!) {
        setRedirectorEnabled(redirectorID: $redirectorID, enabled: $enabled)
    }
`;

type Redirector = {
    id: string;
    domains: string[];
    enabled: boolean;
    online: boolean;
};

export const RedirectorsTab = () => {
    const { data, loading, error, refetch } = useQuery<{ redirectors: Redirector[] }>(GET_REDIRECTORS, {
        pollInterval: 30000,
    });
    const [setEnabled, { loading: saving, error: mutationError }] = useMutation(SET_REDIRECTOR_ENABLED);
    const redirectors = data?.redirectors ?? [];

    const toggle = async (redirector: Redirector) => {
        try {
            await setEnabled({ variables: { redirectorID: redirector.id, enabled: !redirector.enabled } });
            await refetch();
        } catch {
            // The mutation error is rendered below; keep the switch at server state.
        }
    };

    return (
        <div className="flex flex-col gap-3">
            <h3 className="text-xl font-semibold leading-6 text-gray-900">Redirectors</h3>
            {error && <AlertError label="Unable to load redirectors" details={error.message} />}
            {mutationError && <AlertError label="Unable to update redirector" details={mutationError.message} />}
            {loading && redirectors.length === 0 && <p role="status">Loading redirectors…</p>}
            {!loading && !error && redirectors.length === 0 && <p className="text-gray-600">No managed redirectors available.</p>}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
                {redirectors.map((redirector) => (
                    <div key={redirector.id} className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm flex flex-col gap-3">
                        <div className="flex items-center justify-between gap-3">
                            <span className={`text-sm font-medium ${redirector.online ? "text-green-700" : "text-gray-500"}`}>
                                {redirector.online ? "Online" : "Offline"}
                            </span>
                            <label className="flex items-center gap-2 text-sm text-gray-700">
                                <span>Enabled</span>
                                <Switch
                                    aria-label={`Enable redirector for ${redirector.domains.join(", ")}`}
                                    isChecked={redirector.enabled}
                                    isDisabled={saving}
                                    onChange={() => void toggle(redirector)}
                                />
                            </label>
                        </div>
                        <ul className="flex flex-col gap-1 break-all">
                            {redirector.domains.map((domain) => <li key={domain} className="text-gray-900">{domain}</li>)}
                        </ul>
                    </div>
                ))}
            </div>
        </div>
    );
};
