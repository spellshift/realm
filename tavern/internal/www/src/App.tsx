import * as React from "react"
import {
  ChakraProvider,
  Box,
  Text,
  Link,
  VStack,
  Code,
  Grid,
  theme,
  GridItem,
  Card,
  CardHeader,
  Heading,
  CardBody,
  CardFooter,
  Drawer,
} from "@chakra-ui/react";
import { ColorModeSwitcher } from "./ColorModeSwitcher";
import { Logo } from "./Logo";
import './style.css';

import { CreateJobDrawer } from "./components/create-job-drawer/CreateJobDrawer";

export const App = () => (
  <ChakraProvider theme={theme}>
    <CreateJobDrawer />
    {/* <Box p={8}>
    <Grid templateColumns='repeat(4, 1fr)' gap={8}>
      <GridItem colSpan={3}>
        <Grid templateColumns='repeat(4, 1fr)' gap={8}>
          <GridItem>
            <Card>
              <CardHeader>
                <Heading size="md">Shell execution</Heading>
                <Text size="sm">Remote execution tome</Text>
              </CardHeader>
              <CardBody>
              <Text size="sm">
                Execute a shell script using the default interpreter. /bin/bash for macos & linux, and cmd.exe for windows.
              </Text> 
              </CardBody>
              <CardFooter>

              </CardFooter>
            </Card>
          </GridItem>
        </Grid>
      </GridItem>
      <GridItem colSpan={1}>
        Cart
      </GridItem>
    </Grid>
    </Box> */}
  </ChakraProvider>
)
