import * as yup from 'yup';

export const createQuestSchema = yup.object().shape({
    name: yup
        .string()
        .required('Quest name is required')
        .min(1, 'Quest name cannot be empty')
        .max(255, 'Quest name must be less than 255 characters'),

    tome: yup
        .object()
        .nullable()
        .required('Tome selection is required')
        .test('is-not-null', 'Tome selection is required', (value) => value !== null),

    beacons: yup
        .array()
        .of(yup.string())
        .min(1, 'At least one beacon must be selected')
        .required('Beacon selection is required'),

    params: yup
        .array()
        .of(
            yup.object().shape({
                name: yup.string().required(),
                value: yup.mixed().required('Parameter value is required')
            })
        )
        .test('all-params-filled', 'All parameters must have values', (params) => {
            if (!params || params.length === 0) return true;
            return params.every(param => param.value !== null && param.value !== '');
        })
});
