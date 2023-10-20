export function getFilterBarSearchTypes(typeFilters: any){
    return typeFilters.reduce((accumulator:any, currentValue:any) => {
        if(currentValue.kind === "beacon"){
            accumulator.beacon.push(currentValue.name);
        }
        else if(currentValue.kind === "service"){
            accumulator.service.push(currentValue.name);
        }
        else if(currentValue.kind === "group"){
            accumulator.group.push(currentValue.name);
        }
        return accumulator;
    },
    {
        "beacon": [],
        "service": [],
        "group": []
    });
};