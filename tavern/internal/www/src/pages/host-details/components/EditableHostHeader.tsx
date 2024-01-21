import { ApolloError } from "@apollo/client";
import { CloseIcon } from "@chakra-ui/icons";
import { Button } from "@chakra-ui/react";
import { Link } from "react-router-dom";
import { HostType } from "../../../utils/consts";

type Props = {
    hostId?: string;
    loading: boolean;
    error: ApolloError | undefined;
    hostData: HostType | null;
}
const EditableHostHeader = (props: Props) => {
    const { hostId, loading, error, hostData } = props;
    return (
        <div className="flex flex-row justify-between w-full">
            <div className="flex flex-row gap-2 items-center">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">
                    Host details for
                </h3>
                {hostData &&
                    <Link to="/hosts">
                        <Button rightIcon={<CloseIcon />} colorScheme='purple' variant='outline' size="xs">
                            {hostData?.name}
                        </Button>
                    </Link>
                }
                {(error || (!hostData && !loading)) &&
                    <Link to="/hosts">
                        <Button rightIcon={<CloseIcon />} colorScheme='purple' variant='outline' size="xs">
                            Id: {hostId}
                        </Button>
                    </Link>
                }
            </div>
            {/* TODO support changing tags
            <Button
                leftIcon={<EditIcon />}
                colorScheme="gray"
                variant="ghost"
                verticalAlign="center"
            >

                Change host tags
            </Button> */}

        </div>
    );
}
export default EditableHostHeader;
