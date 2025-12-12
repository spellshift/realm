import * as yup from 'yup';

export const createQuestSchema = () => {
    const schema = {
        tomeId: yup.string().required(),
        params: yup.object().shape({
            command: yup.string().required()
        }),
        // beacons: yup.object().test('hasBeaconSelected', 'At least one beacon required', (beacons: any) => {
        //     for (let sVal in beacons) {
        //         if (beacons[sVal]) {
        //             return true;
        //         }
        //     }
        //     return false;
        // }),
        // get beacons() {
        //     return this._beacons;
        // },
        // set beacons(value) {
        //     this._beacons = value;
        // },

    }

    return yup.object().shape(schema);
  };