import { FC } from "react";
import { UserType } from "../utils/consts";
import { Image } from "@chakra-ui/react";

import PlaceholderUser from "../assets/PlaceholderUser.png";

const UserImageAndName: FC<{ userData: UserType | null | undefined }> = ({ userData }) => {
    const creatorImage = (userData?.photoURL && userData?.photoURL !== "") ? userData.photoURL : PlaceholderUser;

    if (!userData) {
        return <div className="text-sm text-gray-500">Not available</div>;
    }

    return (
        <div className="flex flex-row gap-4 items-center" key={userData?.id}>
            <Image
                borderRadius='full'
                boxSize='20px'
                src={creatorImage}
                alt={`Profile of ${userData?.name}`}
            />
            <div className="text-sm text-gray-500">
                {userData?.name}
            </div>
        </div>
    );
};
export default UserImageAndName;
