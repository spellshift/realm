import React from "react";
import { Fragment} from 'react'
import { Dialog, Transition } from '@headlessui/react'
import { XMarkIcon } from '@heroicons/react/24/outline'
import { CopyBlock, tomorrow} from "react-code-blocks";
import { Badge } from "@chakra-ui/react";
import TaskStatusBadge from "../TaskStatusBadge";

type Props = {
    isOpen: boolean,
    setOpen: (arg: any) => any,
    selectedTask: any
}

export const TaskOutput =(props: Props) => {
  const {isOpen, setOpen, selectedTask} = props;
  const createdTime = new Date(selectedTask?.createdAt|| "");
  const finishTime = new Date(selectedTask?.execFinishedAt|| "");
  const startTime = new Date(selectedTask?.execStartedAt|| "");

  let params = selectedTask?.quest?.parameters ? JSON.parse(selectedTask?.quest?.parameters) : {};
  let paramKeys = Object.keys(params);

  return (
    <Transition.Root show={isOpen} as={Fragment}>
      <Dialog as="div" className="relative z-10" onClose={setOpen}>
        <div className="fixed inset-0 bg-black/30" aria-hidden="true" />

        <div className="fixed inset-0 overflow-hidden">
          <div className="absolute inset-0 overflow-hidden">
            <div className="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10 ">
              <Transition.Child
                as={Fragment}
                enter="transform transition ease-in-out duration-500 sm:duration-700"
                enterFrom="translate-x-full"
                enterTo="translate-x-0"
                leave="transform transition ease-in-out duration-500 sm:duration-700"
                leaveFrom="translate-x-0"
                leaveTo="translate-x-full"
              >
                <Dialog.Panel className="pointer-events-auto w-screen max-w-xs md:max-w-md lg:max-w-4xl">
                  <div className="flex h-full flex-col overflow-y-scroll bg-white py-6 shadow-xl">
                    <div className="px-4 sm:px-6">
                      <div className="flex w-full justify-end">

                        <div className="ml-3 flex h-7 items-center">
                          <button
                            type="button"
                            className="relative rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
                            onClick={() => setOpen(false)}
                          >
                            <span className="absolute -inset-2.5" />
                            <span className="sr-only">Close panel</span>
                            <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                          </button>
                        </div>
                      </div>
                    </div>
                    <div className="relative mt-6 flex-1 px-4 sm:px-6 flex flex-col gap-4">
                      <div className="flex flex-row gap-4 items-center">
                          <h2 className="text-3xl font-semibold text-gray-900">{selectedTask?.quest?.name}</h2>
                          <TaskStatusBadge task={selectedTask} />
                      </div>
                      <div className="flex flex-col gap-2">
                        <h3 className="text-2xl">Status</h3>
                        <div className="flex flex-row gap-4 sm:gap-12 text-sm">
                          <div className="flex flex-col">
                            <span className="font-semibold">Created</span>
                            <span>{`${createdTime.toLocaleTimeString()}`}</span>
                            <span>{`on ${createdTime.toDateString()}`}</span>
                          </div>
                          {selectedTask?.execStartedAt && (
                          <div className="flex flex-col">
                            <span className="font-semibold">Started</span>
                            <span>{`${startTime.toLocaleTimeString()}`}</span>
                            <span>{`on ${startTime.toDateString()}`}</span>
                          </div>
                          )}
                          {selectedTask?.execFinishedAt  && (
                          <div className="flex flex-col">
                            <span className="font-semibold">Finished</span>
                            <span>{`${finishTime.toLocaleTimeString()}`}</span>
                            <span>{`on ${finishTime.toDateString()}`}</span>
                          </div>
                          )}
                        </div>
                      </div>
                      <div className="flex flex-col gap-2">
                        <h3 className="text-2xl text-gray-800">Beacon</h3>
                          <div className="flex flex-col gap-1">
                            <h4 className="font-semibold text-gray-900">{selectedTask?.beacon.name}</h4>
                            <div className="flex flex-row flex-wrap gap-1">
                                {selectedTask?.beacon?.host?.tags.map((tag: any)=> {
                                    return <Badge>{tag.name}</Badge>
                                })}
                                <Badge>{selectedTask?.beacon?.host?.name}</Badge>
                                <Badge>{selectedTask?.beacon?.host?.primaryIP}</Badge>
                                <Badge>{selectedTask?.beacon?.host?.platform}</Badge>
                            </div>
                          </div>
                      </div>
                      <div className="flex flex-col gap-2">
                        <h3 className="text-2xl text-gray-800">Tome</h3>
                          <div className="flex flex-col gap-2">
                            <div>
                              <h4 className="font-semibold text-gray-900">{selectedTask?.quest?.tome?.name}</h4>
                              <p className="text-sm">
                                {selectedTask?.quest?.tome?.description}
                              </p>
                            </div>
                            {paramKeys.length > 0 &&(
                              <div className="flex flex-row gap-8 flex-wrap text-sm">
                                  {paramKeys.map((value: string) => {
                                    return (
                                    <div className="flex flex-col gap-0">
                                      <div className="font-semibold">{value}</div>
                                      <div>{params[value]}</div>
                                    </div>)
                                  })}
                              </div>
                            )}
                          </div>
                      </div>
                      <div className="flex flex-col gap-2">
                        <h3 className="text-2xl text-gray-800">Output</h3>
                        <div className="bg-gray-200 rounded-md p-0.5">
                          <CopyBlock
                            text={selectedTask?.output ? selectedTask?.output : "No output available"}
                            language={""}
                            showLineNumbers={false}
                            theme={tomorrow}
                            codeBlock
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                </Dialog.Panel>
              </Transition.Child>
            </div>
          </div>
        </div>
      </Dialog>
    </Transition.Root>
  )
}
  