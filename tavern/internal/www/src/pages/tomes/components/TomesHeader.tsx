import { ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import React from "react";
import Breadcrumbs from "../../../components/Breadcrumbs";
import Button from "../../../components/tavern-base-ui/button/Button";
import PageHeader from "../../../components/tavern-base-ui/PageHeader";

type TomesHeaderType = {
    setOpen: (arg: boolean) => void
}
const TomesHeader = ({ setOpen }: TomesHeaderType) => {

    return (
        <div className="flex flex-col gap-4">
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                    label: "Tomes",
                    link: "/tomes"
                }]} />
                <div>
                    <Button
                        buttonStyle={{ color: "purple", "size": "md" }}
                        leftIcon={<ArrowUpTrayIcon className="h-4 w-4" />}
                        onClick={() => setOpen(true)}
                    >
                        Import tome repository
                    </Button>
                </div>
            </div>
            <PageHeader title="Tomes">
                <>
                    <span>A tome is a prebuilt bundle, which includes execution instructions and files. Tomes are how beacon actions are defined. </span>
                    <a className="external-link" target="_blank" rel="noreferrer" href="https://docs.realm.pub/user-guide/tomes">Learn more</a>
                    <span> about how to write, test, and import tome repositories.</span>
                </>
            </PageHeader>
        </div>
    );
}
export default TomesHeader;
