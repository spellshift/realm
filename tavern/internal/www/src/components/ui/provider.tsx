"use client"

import { Steps, ChakraProvider, defaultSystem } from "@chakra-ui/react";
import {
  ColorModeProvider,
  type ColorModeProviderProps,
} from "./color-mode"

export function Provider(props: ColorModeProviderProps) {
  return (
    <ChakraProvider value={String(defaultSystem)}>
      <ColorModeProvider {...props} />
    </ChakraProvider>
  );
}
