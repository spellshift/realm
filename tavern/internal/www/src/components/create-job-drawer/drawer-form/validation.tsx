import * as yup from 'yup';

export const createJobSchema = () => {
    const schema = {
        tomeId: yup.string().required(),
        params: yup.object().shape({
            command: yup.string().required()
        }),
        sessions: yup.object().test('hasSessionSelected', 'At least one session required', (sessions : any)=> {
            for(let sVal in sessions){
                if(sessions[sVal]){
                    return true;
                }
            }
            return false;
        }),

    }
  
    return yup.object().shape(schema);
  };