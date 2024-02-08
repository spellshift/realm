import { Badge } from "@chakra-ui/react";
import { UserType } from "../utils/consts";
import { CheckCircleIcon, ShieldCheckIcon, XCircleIcon } from "@heroicons/react/24/outline";

type Props = {
    data: UserType
}
const HostTile = (props: Props) => {
    const { data } = props;
    return (
        <div className="flex flex-col gap-2">
            <div>{data.name}</div>
            <div className="flex flex-row flex-wrap gap-1">
                {data.isActivated && <Badge fontSize='0.8em' size="large" colorScheme="green" variant="solid">
                    <div className="flex flex-row gap-1 justify-center items-center p-1" >
                        <CheckCircleIcon className="w-5" color="white"/>
                        <div>Activated</div>
                    </div>
                </Badge>}
                {!data.isActivated && <Badge fontSize='0.8em' size="large" colorScheme="red" variant="solid">
                    <div className="flex flex-row gap-1 justify-center items-center p-1" >
                        <XCircleIcon className="w-5" color="white"/>
                        <div>Pending</div>
                    </div>
                </Badge>}
                {data.isAdmin && <Badge fontSize='0.8em' size="large" colorScheme="green" variant="solid">
                    <div className="flex flex-row gap-1 justify-center items-center p-1" >
                        <ShieldCheckIcon className="w-5" color="white"/>
                        <div>Admin</div>
                    </div>
                </Badge>}
            </div>
        </div>
    )
}
export default HostTile;
