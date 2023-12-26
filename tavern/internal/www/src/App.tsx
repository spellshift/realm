import * as React from "react"
import {
  ChakraProvider,
  theme,
} from "@chakra-ui/react";
import './style.css';
import { useQuery, gql } from '@apollo/client';

import { QuestList } from "./pages/quest-list";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { CreateQuest } from "./pages/create-quest";
import 'react-virtualized/styles.css';
import { TagContextProvider } from "./context/TagContext";
import { QuestDetails } from "./pages/quest-details";
import { AuthorizationContextProvider } from "./context/AuthorizationContext";
import Tasks from "./pages/tasks/Tasks";


const router = createBrowserRouter([
  {
    path: "/",
    element: <Tasks/>,
  },
  {
    path: "/quests",
    element: <QuestList />,
  },
  {
    path: "/quests/:questId",
    element: <QuestDetails />,
  },
  {
    path: "/createQuest",
    element: <CreateQuest />,
  },
  {
    path: "/results",
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
)}
