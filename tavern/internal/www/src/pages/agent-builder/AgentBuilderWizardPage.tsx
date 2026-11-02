import AgentBuilderWizard from "./components/AgentBuilderWizard";

/**
 * Standalone agent builder wizard, reached from the landing page's
 * "Generate agent" action.
 */
export const AgentBuilderWizardPage = () => {
    return <AgentBuilderWizard showHeader={true} />;
};

export default AgentBuilderWizardPage;
