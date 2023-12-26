import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Badge, Box, Heading } from "@chakra-ui/react";
import { formatDistance } from "date-fns";

type Props = {
    data: any;
}
const AccordinanQuestView = (props: Props) => {
    const {data} = props;

    const currentDate = new Date();

    function getInitialQuestsTableData(data:any ){ 
        const formattedData = data?.map( (quest: any) => {
            const taskDetails = quest.tasks.reduce( (map:any, task: any)=> {
                const modMap = {...map};

                // Add task count
                modMap.totalTasks += 1;

                // Add if task has finished
                if(task.execFinishedAt){
                    modMap.finished += 1;
                }

                // Add if task has updated
                if(!modMap.lastUpdated || new Date(task.lastModifiedAt) > new Date(modMap.lastUpdated) ){
                    modMap.lastUpdated = task.lastModifiedAt;
                }

                // Add if task has output
                if(task.output !== ""){
                    modMap.outputCount += 1;
                }

                return modMap
            },
                {
                    finished: 0,
                    totalTasks: 0,
                    outputCount: 0,
                    lastUpdated: null

                }
            );

            return {
                id: quest.id,
                name: quest.name,
                tasks: quest.tasks,
                ...taskDetails
            }
        });
        return formattedData.sort(function(a:any,b:any){return new Date(b.lastUpdated).getTime() - new Date(a.lastUpdated).getTime()});
    };
    const format = getInitialQuestsTableData(data);

    return (
        <Accordion allowMultiple={true}>
        {format.map((item:any)=> {
            return(
                <AccordionItem key={item.id}>
                    <h2>
                    <AccordionButton>
                        <Box as="div" display="flex" flexDirection="column" flexDir="column" flex='1' textAlign='left'>
                            <div className=" grid grid-cols-5">
                                <div className="col-span-2">
                                    <Heading size="sm">
                                        {item.name}
                                    </Heading>
                                </div>
                                <div className="flex flex-row items-center gap-1">
                                    Finished: 
                                    {(item.finished < item.totalTasks) ? 
                                        <Badge ml='1' px='4' colorScheme='gray' fontSize="font-sm">
                                            {item.finished}/{item.totalTasks}
                                        </Badge>
                                        : (
                                        <Badge ml='1' px='4' colorScheme='green' fontSize="font-sm">
                                           {item.finished}/{item.totalTasks} 
                                         </Badge>
                                        )
                                    }
                                </div> 
                                <div className="flex flex-row items-center gap-1">
                                    Results:
                                    {item.outputCount > 0 ? (
                                        <Badge ml='1' px='4' colorScheme='purple' fontSize="font-sm">
                                            {item.outputCount}
                                        </Badge>
                                    ): (
                                        <Badge ml='1' px='4' colorScheme='alphaWhite' fontSize="font-sm">
                                           {item.outputCount}
                                        </Badge>
                                    )}
                                </div>
                                <div>
                                    last updated: {formatDistance(new Date(item.lastUpdated), currentDate)}
                                </div>
                            </div>
                        </Box>
                        <AccordionIcon />
                    </AccordionButton>
                    </h2>
                    <AccordionPanel pb={4}>
                        {/* <Table outputData={item} /> */}
                        {item.tasks.map((item:any)=> item.id).join(",")}
                    </AccordionPanel>
                </AccordionItem>
            );
        })}
    </Accordion>
    );
}
export default AccordinanQuestView