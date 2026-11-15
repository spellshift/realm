// PreviewShell wraps every design-system preview story in the same provider
// stack the real app relies on, backed by a schema-free mock Apollo client.
// Nested components call useAuthorization/useFilters/useSorts/useHost/etc., all
// of which throw without their provider, so every one is supplied here.
import * as React from "react";
import { ChakraProvider, theme } from "@chakra-ui/react";
import { ApolloProvider } from "@apollo/client";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { makeMockClient } from "./mock-link";

import { AuthorizationContextProvider } from "../src/context/AuthorizationContext";
import { UserPreferencesContextProvider } from "../src/context/UserPreferences";
import { PollingProvider } from "../src/context/PollingContext";
import { TagContextProvider } from "../src/context/TagContext";
import { FilterProvider } from "../src/context/FilterContext";
import { SortsProvider } from "../src/context/SortContext";
import { HostContextProvider } from "../src/context/HostContext";
import { CreateQuestModalProvider } from "../src/context/CreateQuestModalContext";

const client = makeMockClient();

// Router-dependent providers (Filter=useLocation, Host=useParams,
// CreateQuestModal=useNavigate) live inside a matched /hosts/:hostId route so a
// sample hostId is present and HostContext resolves a sample host from the mock.
function Providers({ children }: { children: React.ReactNode }) {
  return (
    <AuthorizationContextProvider>
      <UserPreferencesContextProvider>
        <PollingProvider>
          <TagContextProvider>
            <FilterProvider>
              <SortsProvider>
                <HostContextProvider>
                  <CreateQuestModalProvider>
                    <div className="ds-preview-root" style={{ background: "#fff" }}>
                      {children}
                    </div>
                  </CreateQuestModalProvider>
                </HostContextProvider>
              </SortsProvider>
            </FilterProvider>
          </TagContextProvider>
        </PollingProvider>
      </UserPreferencesContextProvider>
    </AuthorizationContextProvider>
  );
}

export function PreviewShell({ children }: { children?: React.ReactNode }) {
  const tree = <Providers>{children}</Providers>;
  return (
    <ChakraProvider theme={theme}>
      <ApolloProvider client={client}>
        <MemoryRouter initialEntries={["/hosts/h-sample"]}>
          <Routes>
            <Route path="/hosts/:hostId" element={tree} />
            <Route path="*" element={tree} />
          </Routes>
        </MemoryRouter>
      </ApolloProvider>
    </ChakraProvider>
  );
}

export default PreviewShell;
