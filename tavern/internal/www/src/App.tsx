import * as React from "react"
import {
  ChakraProvider,
  theme,
} from "@chakra-ui/react";
import './style.css';
import { useQuery, gql } from '@apollo/client';

import { QuestList } from "./pages/quest-list";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Home } from "./pages/home";
import { CreateQuest } from "./pages/create-quest";
import 'react-virtualized/styles.css';
import { TagContextProvider } from "./context/TagContext";
import { QuestDetails } from "./pages/quest-details";
import { OutputResults } from "./pages/output-results";


const router = createBrowserRouter([
  {
    path: "/",
    element: <QuestList />,
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
    path: "/output-results",
    element: <OutputResults />,
  },
]);

export const App = () => {
  
  return (
    <ChakraProvider theme={theme}>
      <TagContextProvider>
        <RouterProvider router={router} />
      </TagContextProvider>
    </ChakraProvider>
)}
