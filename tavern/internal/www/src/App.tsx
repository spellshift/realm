import * as React from "react"
import {
  ChakraProvider,
  theme,
} from "@chakra-ui/react";
import './style.css';

import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { CreateQuest } from "./pages/create-quest";
import 'react-virtualized/styles.css';
import { TagContextProvider } from "./context/TagContext";
import { AuthorizationContextProvider } from "./context/AuthorizationContext";
import Tasks from "./pages/tasks/Tasks";
import HostList from "./pages/host-list/HostList";
import HostDetails from "./pages/host-details/HostDetails";
import { Dashboard } from "./pages/dashboard";
import { Tomes } from "./pages/tomes";
import Quests from "./pages/quest-list/Quests";
import Shell from "./pages/shell/Shell";
import { AdminPortal } from "./pages/admin";
import { UserPreferencesContextProvider } from "./context/UserPreferences";
import { FilterProvider } from "./context/FilterContext";


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
    element: <Quests />,
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
  {
    path: "/tomes",
    element: <Tomes />,
  },
  {
    path: "/shells/:shellId",
    element: <Shell />,
  },
  {
    path: "/admin",
    element: <AdminPortal />,
  },
]);

export const App = () => {

  return (
    <ChakraProvider theme={theme}>
      <AuthorizationContextProvider>
        <TagContextProvider>
          <UserPreferencesContextProvider>
            <FilterProvider>
              <RouterProvider router={router} />
            </FilterProvider>
          </UserPreferencesContextProvider>
        </TagContextProvider>
      </AuthorizationContextProvider>
    </ChakraProvider>
  )
}
