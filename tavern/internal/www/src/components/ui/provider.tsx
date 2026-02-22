"use client"

import { Steps, ChakraProvider, defaultSystem } from "@chakra-ui/react";
import { ThemeProvider, useTheme } from "next-themes"
import type { ThemeProviderProps } from "next-themes"

export function Provider(props: ThemeProviderProps) {
  return (
    <ChakraProvider value={defaultSystem}>
      <ThemeProvider defaultTheme='light' attribute="class" disableTransitionOnChange {...props} />
    </ChakraProvider>
  );
}
