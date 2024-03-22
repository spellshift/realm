import React, { useCallback } from "react";
import TaskStatusBadge from "../../components/TaskStatusBadge";
import BeaconTile from "../../components/BeaconTile";
import TomeAccordion from "../../components/TomeAccordion";
import { Image } from "@chakra-ui/react";
import ErrorWrapper from "./ErrorWrapper";
import { useNavigate } from "react-router-dom";
import { checkIfBeaconOffline, constructTomeParams } from "../../utils/utils";
import Modal from "../../components/tavern-base-ui/Modal";
import Button from "../../components/tavern-base-ui/button/Button";
import ExpandedDetails from "./ExpandedDetails";

type Props = {
  isOpen: boolean,
  setOpen: (arg: any) => any,
  selectedTask: any
}

export const TaskOutput = (props: Props) => {
  const { isOpen, setOpen, selectedTask } = props;

  const nav = useNavigate();
  const createdTime = new Date(selectedTask?.createdAt || "");
  const finishTime = new Date(selectedTask?.execFinishedAt || "");
  const startTime = new Date(selectedTask?.execStartedAt || "");

  const params = constructTomeParams(selectedTask?.quest?.parameters, selectedTask?.quest?.tome?.paramDefs);

  const beaconOffline = checkIfBeaconOffline(selectedTask?.beacon);

  const hanldeRerunQuest = useCallback(() => {
    const beaconId = selectedTask?.beacon?.id;
    const tome = selectedTask?.quest?.tome

    nav("/createQuest", {
      state: {
        step: 2,
        beacons: [beaconId],
        tome: tome,
        params: params,
        name: selectedTask?.quest?.name
      }
    });
  }, [selectedTask, nav, params]);

  return (
    <Modal isOpen={isOpen} setOpen={setOpen}>
      <div className="relative flex-1 flex flex-col gap-4">
        <div className="flex flex-row justify-between">
          <div className="flex flex-row gap-4 items-center">
            <h2 className="text-3xl font-semibold text-gray-900">{selectedTask?.quest?.name}</h2>
            <TaskStatusBadge task={selectedTask} />
          </div>
          {!beaconOffline &&
            <div>
              <Button
                buttonStyle={{ color: "gray", size: "md" }}
                buttonVariant="ghost"
                onClick={() => hanldeRerunQuest()}
                disabled={beaconOffline}
                title="Beacon must be online to rerun"
              >
                Re-run task
              </Button>
            </div>
          }
        </div>
        <div className="flex flex-col gap-2">
          <h3 className="text-2xl">Status</h3>
          <div className="flex flex-row gap-4 sm:gap-12 text-sm mx-4">
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
            {selectedTask?.execFinishedAt && (
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
          <div className="mx-4">
            <BeaconTile beaconData={selectedTask?.beacon} />
          </div>
        </div>
        <div className="flex flex-col gap-2">
          <h3 className="text-2xl text-gray-800">Tome</h3>
          <TomeAccordion tome={selectedTask?.quest?.tome} params={params} />
        </div>
        {selectedTask?.quest?.creator && (
          <div className="flex flex-col gap-2">
            <h3 className="text-2xl text-gray-800">Creator</h3>
            <div className="flex flex-row gap-2 items-center mx-4">
              <Image
                borderRadius='full'
                boxSize='20px'
                src={selectedTask?.quest?.creator?.photoURL}
                alt={`Profile of ${selectedTask?.quest?.creator?.name}`}
              />
              <div className="text-sm flex flex-row gap-1 items-center text-gray-600">
                {selectedTask?.quest?.creator?.name}
              </div>
            </div>
          </div>
        )}
        {selectedTask && selectedTask.error.length > 0 &&
          <ErrorWrapper error={selectedTask.error} />
        }
        {selectedTask && selectedTask?.id && <ExpandedDetails id={selectedTask.id} />}
      </div>
    </Modal>
  )
}
