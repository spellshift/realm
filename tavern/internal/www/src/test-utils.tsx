import * as React from "react"
import { render, RenderOptions } from "@testing-library/react"
import { Steps, ChakraProvider, theme } from "@chakra-ui/react";

const AllProviders = ({ children }: { children?: React.ReactNode }) => (
  <ChakraProvider value={String(theme)}>{children}</ChakraProvider>
)

const customRender = (ui: React.ReactElement, options?: RenderOptions) =>
  render(ui, { wrapper: AllProviders, ...options })

export { customRender as render }
