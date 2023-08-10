import * as React from "react"
import {
  ChakraProvider,
  theme,
} from "@chakra-ui/react";
import './style.css';
import { useQuery, gql } from '@apollo/client';

import { JobList } from "./pages/job-list";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Home } from "./pages/home";
import { CreateJob } from "./pages/create-job";
import 'react-virtualized/styles.css';
import { TagContextProvider } from "./context/TagContext";
import { Job } from "./pages/job";

const GET_TOMES = gql`
    query get_tomes{
      tomes {
        id
        name
        paramDefs
        description
        eldritch
      }
    }
`;

const router = createBrowserRouter([
  {
    path: "/",
    element: <JobList />,
  },
  {
    path: "/jobs",
    element: <JobList />,
  },
  {
    path: "/jobs/:jobId",
    element: <Job />,
  },
  {
    path: "/createJob",
    element: <CreateJob />,
  },
]);

export const App = () => {
  const { loading, error, data } = useQuery(GET_TOMES);

  return (
    <ChakraProvider theme={theme}>
      <TagContextProvider>
        <RouterProvider router={router} />
      </TagContextProvider>
    </ChakraProvider>
)}
