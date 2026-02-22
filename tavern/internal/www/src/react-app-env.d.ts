/// <reference types="react-scripts" />
import 'react';

declare module 'react' {
    interface Attributes {
        children?: ReactNode | undefined;
        [key: string]: any;
    }
    interface IntrinsicAttributes {
        children?: ReactNode | undefined;
        [key: string]: any;
    }
}
