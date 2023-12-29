import React from 'react';

import Select from 'react-select';

const MultiSelectService = () => {
    const options = [
        { value: 'chocolate', label: 'Chocolate' },
        { value: 'strawberry', label: 'Strawberry' },
        { value: 'vanilla', label: 'Vanilla' }
      ];

      return (
        <Select
            isMulti
            name="colors"
            options={options}
            className="basic-multi-select"
            classNamePrefix="select"
        />
      );
}
export default MultiSelectService;
