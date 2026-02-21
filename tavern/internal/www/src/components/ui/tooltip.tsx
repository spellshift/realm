import { Tooltip as ChakraTooltip, Portal } from "@chakra-ui/react";
import * as React from "react"

export interface TooltipProps extends React.ComponentProps<typeof ChakraTooltip.Root> {
  showArrow?: boolean
  portalled?: boolean
  portalRef?: React.RefObject<HTMLElement | null>
  content: React.ReactNode
  contentProps?: React.ComponentProps<typeof ChakraTooltip.Content>
  disabled?: boolean
  children?: React.ReactNode
}

export const Tooltip = React.forwardRef<HTMLDivElement, TooltipProps>(
  function Tooltip(props, ref) {
    const {
      showArrow,
      children,
      disabled,
      portalled = true,
      content,
      contentProps,
      portalRef,
      ...rest
    } = props

    if (disabled) return children

    return (
      <ChakraTooltip.Root {...rest}>
        <ChakraTooltip.Trigger asChild>{children}</ChakraTooltip.Trigger>
        <Portal disabled={!portalled} container={portalRef}>
          <ChakraTooltip.Positioner>
            <ChakraTooltip.Content ref={ref} {...contentProps}>
              {showArrow && (
                <ChakraTooltip.Arrow>
                  <ChakraTooltip.ArrowTip />
                </ChakraTooltip.Arrow>
              )}
              {content}
            </ChakraTooltip.Content>
          </ChakraTooltip.Positioner>
        </Portal>
      </ChakraTooltip.Root>
    )
  },
)
