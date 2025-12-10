import { FC } from "react";
import { Image } from "@chakra-ui/react";

import PlaceholderUser from "../assets/PlaceholderUser.png";
import { UserNode } from "../utils/interfacesQuery";

const UserImageAndName: FC<{ userData: UserNode | null | undefined }> = ({ userData }) => {
    const creatorImage = (userData?.photoURL && userData?.photoURL !== "") ? userData.photoURL : PlaceholderUser;

    return (
        <div className="flex flex-row gap-4 items-center" key={userData?.id}>
            <Image
                borderRadius='full'
                boxSize='20px'
                src={creatorImage}
                alt={`Profile of ${userData?.name}`}
            />
            <div className="text-gray-600 text-sm">
                {userData?.name ? userData.name : 'Not found'}
            </div>
        </div>
    );
};
export default UserImageAndName;
