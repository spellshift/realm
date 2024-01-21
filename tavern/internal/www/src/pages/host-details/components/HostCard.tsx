import { EditIcon } from "@chakra-ui/icons";
import { Button } from "@chakra-ui/react";
import HostTile from "../../../components/HostTile";
import { HostType } from "../../../utils/consts";


const HostCard = (
    { host }:
        { host: HostType }
) => {
    return (
        <div className="flex flex-row p-4 bg-white rounded-lg shadow-lg mt-2 items-center">
            <div className="flex flex-row flex-1 gap-4">
                <HostTile data={host} />
            </div>
            {/*
            TODO: Add way to edit tags
            <Button
                leftIcon={<EditIcon />}
                size="sm"
                colorScheme="purple"
            >
                Change tags
            </Button>
            */}
        </div>
    );
};
export default HostCard;
