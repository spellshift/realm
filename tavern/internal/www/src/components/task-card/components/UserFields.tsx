import { FC } from "react";
import { Image } from "@chakra-ui/react";
import { CircleUser } from "lucide-react";
import { UserNode } from "../../../utils/interfacesQuery";

interface UserFieldsProps {
    user: UserNode | null | undefined;
}

const UserFields: FC<UserFieldsProps> = ({ user }) => {
    const hasPhoto = user?.photoURL && user.photoURL !== "";

    return (
        <div className="flex flex-row gap-1 items-center">
            {hasPhoto ? (
                <Image
                    borderRadius="full"
                    boxSize="16px"
                    src={user.photoURL}
                    alt={`Profile of ${user?.name}`}
                />
            ) : (
                <CircleUser className="w-4" />
            )}
            <div className="text-xs">
                {user?.name ? user.name : "Unknown"}
            </div>
        </div>
    );
};

export default UserFields;
