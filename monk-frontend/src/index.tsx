import React from "react";
import { render } from "react-dom";

import {
    ApolloClient,
    InMemoryCache,
    ApolloProvider,
    useQuery,
    gql
} from "@apollo/client";

const client = new ApolloClient({
    uri: 'http://localhost:5433/graphql',
    cache: new InMemoryCache()
});

render(<h1>Hello, World!</h1>, document.getElementById("root"));

client
    .query({
        query: gql`
      query MyQuery {
  article(id: "77fb3d58-c41d-11eb-a4b0-37f7c8707735") {
    id
    name
  }
}
    `
    })
    .then(result => console.log(result));