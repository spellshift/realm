import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import HostTasks from "./components/HostTasks";
import HostContent from "./components/HostContent";

const HostDetails = () => {
    return (
        <PageWrapper currNavItem={PageNavItem.hosts}>
            <HostContent />
            <HostTasks />
        </PageWrapper>
    );
}
export default HostDetails;
