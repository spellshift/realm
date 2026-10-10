import { useLocation } from "react-router-dom";

export function useQuestModalOptions() {
    const { pathname } = useLocation();

    if (pathname.startsWith("/hosts/")) {
        return { refetchQueries: ["GetHostTaskIds", "GetHostContext"] };
    }
    return { navigateToQuest: true };
}
