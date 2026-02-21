import * as React from "react"
import * as ReactDOM from "react-dom/client"
import { App } from "./App"
import reportWebVitals from "./reportWebVitals"
import * as serviceWorker from "./serviceWorker"
import { ApolloClient, InMemoryCache, ApolloProvider } from '@apollo/client';
import { relayStylePagination } from "@apollo/client/utilities";


const container = document.getElementById("root")
if (!container) throw new Error('Failed to find the root element');
const root = ReactDOM.createRoot(container);
const REACT_APP_API_ENDPOINT = process.env.REACT_APP_API_ENDPOINT ?? 'http://localhost:8000';


const cache = new InMemoryCache({
  typePolicies: {
    Query: {
      fields: {
        tasks: relayStylePagination(["where"]),
        quests: relayStylePagination(["where"]),
        hosts: relayStylePagination(["where"])
      },
    },
  },
});

const client = new ApolloClient({
  uri: `${REACT_APP_API_ENDPOINT}/graphql`,
  cache: cache,
});

root.render(
  <React.StrictMode>
    <ApolloProvider client={client}>

      <App />
    </ApolloProvider>
  </React.StrictMode>,
)

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://cra.link/PWA
serviceWorker.unregister()

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
