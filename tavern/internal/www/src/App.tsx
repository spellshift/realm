import * as React from "react"
import {
  ChakraProvider,
  theme,
} from "@chakra-ui/react";
import './style.css';

import { QuestList } from "./pages/quest-list";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { CreateQuest } from "./pages/create-quest";
import 'react-virtualized/styles.css';
import { TagContextProvider } from "./context/TagContext";
import { AuthorizationContextProvider } from "./context/AuthorizationContext";
import Tasks from "./pages/tasks/Tasks";
import HostList from "./pages/host-list/HostList";
import HostDetails from "./pages/host-details/HostDetails";
import { Dashboard } from "./pages/dashboard";


const router = createBrowserRouter([
  {
    path: "/",
    element: <Dashboard />,
  },
  {
    path: "/dashboard",
    element: <Dashboard />,
  },
  {
    path: "/hosts",
    element: <HostList />,
  },
  {
    path: "/hosts/:hostId",
    element: <HostDetails />,
  },
  {
    path: "/quests",
    element: <QuestList />,
  },
  {
    path: "/tasks/:questId",
    element: <Tasks />,
  },
  {
    path: "/createQuest",
    element: <CreateQuest />,
  },
  {
    path: "/tasks",
    element: <Tasks />,
  },
]);

export const App = () => {

  return (
    <ChakraProvider theme={theme}>
      <AuthorizationContextProvider>
        <TagContextProvider>
          <RouterProvider router={router} />
        </TagContextProvider>
      </AuthorizationContextProvider>
    </ChakraProvider>
  )
}
