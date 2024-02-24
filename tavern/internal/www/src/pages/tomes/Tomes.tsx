import React from "react"
import { PageWrapper } from "../../components/page-wrapper"
import { PageNavItem } from "../../utils/enums"

import { Button } from "@chakra-ui/react";
import { ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import RepositoryTable from "./components/RepositoryTable";
import { useRepositoryView } from "./hooks/useRepositoryView";

export const Tomes = () => {
    const { loading, repositories, error } = useRepositoryView();
    console.log(repositories);

    return (
        <PageWrapper currNavItem={PageNavItem.tomes}>
            <div className="border-b border-gray-200 pb-5 flex flex-col sm:flex-row  sm:items-center sm:justify-between gap-4">
                <div className="flex-1 flex flex-col gap-2">
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Tomes</h3>
                    <div className="max-w-2xl text-sm">
                        <span>A tome is a prebuilt bundle, which includes execution instructions and files. Tomes are how beacon actions are defined. </span>
                        <a className=" text-gray-600 hover:text-purple-900 font-semibold  underline hover:cursor-pointer" href="https://docs.realm.pub/user-guide/golem#creating-and-testing-tomes">Learn more</a>
                        <span> about how to write, test, and import tome repositories.</span>
                    </div>

                </div>
                <div>
                    <Button size="sm" leftIcon={<ArrowUpTrayIcon className="h-4 w-4" />}>
                        Import tome repository
                    </Button>
                </div>
            </div>
            <div>

                {(repositories && repositories.length > 0) && <RepositoryTable repositories={repositories} />}
            </div>
        </PageWrapper>
    )
}
