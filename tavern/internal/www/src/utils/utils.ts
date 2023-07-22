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