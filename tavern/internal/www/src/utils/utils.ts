export const safelyJsonParse = (value: string) => {
    let error = false;
    let params = [];
    if(value !== ""){
        try{
            params = JSON.parse(value);
        }
        catch{
            error = true;
        }
    }
    return {error, params};
};

export function getFilterNameByTypes(typeFilters: Array<any>){
    return typeFilters.reduce((accumulator:any, currentValue:any) => {
        if(currentValue.kind === "beacon"){
            accumulator.beacon.push(currentValue.name);
        }
        else if(currentValue.kind === "platform"){
            accumulator.platform.push(currentValue.name);
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
        "group": [],
        "platform": []
    });
};