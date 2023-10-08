import React from "react";
import { Box, Code, Heading, Slide, } from "@chakra-ui/react";
import { Fragment, useState } from 'react'
import { Dialog, Transition } from '@headlessui/react'
import { XMarkIcon } from '@heroicons/react/24/outline'
import { Task } from "../../../utils/consts";
import { format } from 'date-fns'
import { CopyBlock, tomorrow} from "react-code-blocks";

type Props = {
    isOpen: boolean,
    setOpen: (arg: any) => any,
    selectedTask: Task | null
}

export const TaskOutput =(props: Props) => {
    const {isOpen, setOpen, selectedTask} = props;
    const createdTime = new Date(selectedTask?.createdAt|| "");
    const finishTime = new Date(selectedTask?.execFinishedAt|| "");
    const startTime = new Date(selectedTask?.execStartedAt|| "");
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
                <Dialog.Panel className="pointer-events-auto w-screen max-w-2xl">
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
                      <div className="flex flex-col gap-1">
                        <h4 className="font-semibold text-gray-900">Beacon name</h4>
                        <p>{selectedTask?.beacon.name}</p>
                      </div>
                      <div className="flex flex-col gap-1">
                        <h4 className="font-semibold text-gray-900">Created</h4>
                        <p>{`${createdTime.toLocaleTimeString()} on ${createdTime.toDateString()} `}</p>
                      </div>
                      {selectedTask?.execStartedAt &&
                        <div className="flex flex-col gap-1">
                          <h4 className="font-semibold text-gray-900">Started</h4>
                          <p>{`${startTime.toLocaleTimeString()} on ${startTime.toDateString()} `}</p>
                        </div>
                      }
                      {selectedTask?.execFinishedAt  && 
                        <div className="flex flex-col gap-1">
                          <h4 className="font-semibold text-gray-900">Finished</h4>
                          <p>{`${finishTime.toLocaleTimeString()} on ${finishTime.toDateString()} `}</p>
                       </div>
                      }
                      <div className="flex flex-col gap-1">
                        <h4 className="font-semibold text-gray-900">Output</h4>
                        <div className="bg-gray-200 rounded-md p-0.5">
                          <CopyBlock
                            text={selectedTask?.output ? `${selectedTask?.output} \n \t test` : "No output available"}
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
  