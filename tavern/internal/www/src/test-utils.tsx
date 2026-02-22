import * as React from "react"
import { render, RenderOptions } from "@testing-library/react"
import { Provider } from "./components/ui/provider";

const AllProviders = ({ children }: { children?: React.ReactNode }) => (
  <Provider>{children}</Provider>
)

const customRender = (ui: React.ReactElement, options?: RenderOptions) =>
  render(ui, { wrapper: AllProviders, ...options })

export { customRender as render }
