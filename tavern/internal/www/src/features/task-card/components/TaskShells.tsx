import { Shell } from "../../../utils/consts";
import { CommandLineIcon, } from "@heroicons/react/24/outline";
import { Image } from "@chakra-ui/react";

import PlaceholderUser from "../../../assets/PlaceholderUser.png";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useNavigate } from "react-router-dom";

const TaskShells = ({ shells }: { shells: Array<Shell> }) => {
    const nav = useNavigate();

    if (!shells || shells?.length < 1) {
        return null;
    }

    return (
        <div className="flex flex-col gap-4 max-h-80 overflow-y-scroll overflow-x-scroll">
            {shells.map((shell) => {
                const closeAtTime = new Date(shell.closedAt || "");

                return (
                    <div className="flex flex-row gap-4">
                        <CommandLineIcon className="h-5 w-5 mt-1" />
                        <div className="flex flex-col gap-1">
                            <div className="flex flex-row gap-4 items-center">
                                <div className="text-gray-600">
                                    Shell Id: {shell?.id}
                                </div>
                            </div>
                            {shell.closedAt ? <div className="text-sm text-gray-500">{`Shell closed at: $${closeAtTime.toLocaleTimeString()} on ${closeAtTime.toDateString()}`}</div>
                                : (
                                    <>
                                        <div className="flex flex-row gap-4 text-sm text-gray-500">
                                            <div className="flex flex-row gap-1 items-center ">
                                                Active users:
                                            </div>
                                            {shell?.activeUsers?.map((user) => {
                                                const userImage = (user?.photoURL && user?.photoURL !== "") ? user?.photoURL : PlaceholderUser;
                                                return (
                                                    <div className="flex flex-row gap-1 items-center flex-wrap">
                                                        <Image
                                                            borderRadius='full'
                                                            boxSize='12px'
                                                            src={userImage}
                                                            alt={`Profile of ${user?.name}`}
                                                        />
                                                        <div className="flex flex-row gap-1 items-center">
                                                            {user?.name}
                                                        </div>
                                                    </div>
                                                )
                                            })}
                                            {shell?.activeUsers.length === 0 && <div>None found</div>}
                                        </div>
                                        <div className="-mt-1">
                                            <Button
                                                buttonStyle={{ color: "purple", size: "xs", vPadding: "none", xPadding: "none" }}
                                                buttonVariant='ghost'
                                                className='hover:underline hover:bg-'
                                                onClick={() => {
                                                    nav(`/shells/${shell.id}`)
                                                }}>
                                                Join shell instance
                                            </Button>
                                        </div>
                                    </>
                                )}
                        </div>
                    </div>
                )
            })}
        </div>
    );
};
export default TaskShells;
