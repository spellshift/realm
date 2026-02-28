import {
  ChakraProvider,
  theme,
} from "@chakra-ui/react";
import './style.css';

import { createBrowserRouter, RouterProvider } from "react-router-dom";
import 'react-virtualized/styles.css';
import { TagContextProvider } from "./context/TagContext";
import { AuthorizationContextProvider } from "./context/AuthorizationContext";
import HostDetails from "./pages/host-details/HostDetails";
import { Dashboard } from "./pages/dashboard";
import Shell from "./pages/shell/Shell";
import ShellV2 from "./pages/shellv2";
import { UserPreferencesContextProvider } from "./context/UserPreferences";
import { AdminPortal } from "./pages/admin/AdminPortal";
import { CreateQuest } from "./pages/create-quest/CreateQuest";
import Assets from "./pages/assets/Assets";
import { PageWrapper } from "./components/page-wrapper";
import Quests from "./pages/quests";
import Hosts from "./pages/hosts/Hosts";
import { Tasks } from "./pages/tasks";
import { Tomes } from "./pages/tomes/Tomes";


const router = createBrowserRouter([
  {
    path: "/",
    element: <PageWrapper />,
    children: [
      {
        index: true,
        element: <Dashboard />,
      },
      {
        path: "dashboard",
        element: <Dashboard />,
      },
      {
        path: "hosts",
        element: <Hosts />,
      },
      {
        path: "hosts/:hostId",
        element: <HostDetails />,
      },
      {
        path: "quests",
        element: <Quests />,
      },
      {
        path: "tasks/:questId",
        element: <Tasks />,
      },
      {
        path: "createQuest",
        element: <TagContextProvider><CreateQuest /></TagContextProvider>,
      },
      {
        path: "tasks",
        element: <Tasks />,
      },
      {
        path: "tomes",
        element: <Tomes />,
      },
      {
        path: "assets",
        element: <Assets />,
      },
      {
        path: "shells/:shellId",
        element: <Shell />,
      },
      {
        path: "admin",
        element: <AdminPortal />,
      },

    ]
  },
  {
    path: "shellv2/:shellId",
    element: <ShellV2 />,
  },
]);

export const App = () => {

  return (
    <ChakraProvider theme={theme}>
      <AuthorizationContextProvider>
            <UserPreferencesContextProvider>
              <RouterProvider router={router} />
            </UserPreferencesContextProvider>
      </AuthorizationContextProvider>
    </ChakraProvider>
  )
}
