import React from 'react';
import { CheckIcon } from '@heroicons/react/20/solid'
import { StepStatus } from '../../utils/enums';

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(' ')
}

export interface FormStep {
  name: string;
  description: string;
  href: string;
  step: number;
}

type Props = {
  currStep: number;
  steps: Array<FormStep>;
}
export const FormSteps = (props: Props) => {
  const { currStep, steps } = props;

  return (
    <nav aria-label="Progress">
      <ol role="list" className="overflow-hidden">
        {steps.map((step, stepIdx) => {
          const status = currStep > step.step ? StepStatus.Complete : currStep === step.step ? StepStatus.Current : StepStatus.Upcoming;

          return (
            <li key={step.name} className={classNames(stepIdx !== steps.length - 1 ? 'pb-10' : '', 'relative')}>
              {status === StepStatus.Complete ? (
                <>
                  {stepIdx !== steps.length - 1 ? (
                    <div className="absolute top-4 left-4 -ml-px mt-0.5 h-full w-0.5 bg-purple-700" aria-hidden="true" />
                  ) : null}
                  <div className="group relative flex items-start">
                    <span className="flex h-9 items-center">
                      <span className="relative z-10 flex h-8 w-8 items-center justify-center rounded-full bg-purple-700 group-hover:bg-purple-800">
                        <CheckIcon className="h-5 w-5 text-white" aria-hidden="true" />
                      </span>
                    </span>
                    <span className="ml-4 flex min-w-0 flex-col">
                      <span className="text-sm font-medium">{step.name}</span>
                      <span className="text-sm text-gray-500">{step.description}</span>
                    </span>
                  </div>
                </>
              ) : status === StepStatus.Current ? (
                <>
                  {stepIdx !== steps.length - 1 ? (
                    <div className="absolute top-4 left-4 -ml-px mt-0.5 h-full w-0.5 bg-gray-300" aria-hidden="true" />
                  ) : null}
                  <div className="group relative flex items-start" aria-current="step">
                    <span className="flex h-9 items-center" aria-hidden="true">
                      <span className="relative z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 border-purple-700 bg-white">
                        <span className="h-2.5 w-2.5 rounded-full bg-purple-700" />
                      </span>
                    </span>
                    <span className="ml-4 flex min-w-0 flex-col">
                      <span className="text-sm font-medium text-purple-700">{step.name}</span>
                      <span className="text-sm text-gray-500">{step.description}</span>
                    </span>
                  </div>
                </>
              ) : (
                <>
                  {stepIdx !== steps.length - 1 ? (
                    <div className="absolute top-4 left-4 -ml-px mt-0.5 h-full w-0.5 bg-gray-300" aria-hidden="true" />
                  ) : null}
                  <div className="group relative flex items-start">
                    <span className="flex h-9 items-center" aria-hidden="true">
                      <span className="relative z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 border-gray-300 bg-white">
                        <span className="h-2.5 w-2.5 rounded-full bg-transparent" />
                      </span>
                    </span>
                    <span className="ml-4 flex min-w-0 flex-col">
                      <span className="text-sm font-medium text-gray-500">{step.name}</span>
                      <span className="text-sm text-gray-500">{step.description}</span>
                    </span>
                  </div>
                </>
              )}
            </li>
          )
        })}
      </ol>
    </nav>
  )
}
