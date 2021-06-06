import * as React from "react";
import * as ReactDOM from "react-dom";

import {
  ApolloClient,
  InMemoryCache,
  ApolloProvider,
  useQuery,
  gql
} from "@apollo/client";

import Article from "./Article";
import './monk.css'

let getArticlesQuery = gql`
query MyQuery {
  articlesConnection {
    nodes {
      id
      name
      url
    }
  }
}

`

const client = new ApolloClient({
  uri: 'http://localhost:5433/graphql',
  cache: new InMemoryCache()
});


type Props = {
  name: string,
};

type State = {
  rawjson: string,
  articles: typeof Article[],
};

class ArticleTable extends React.Component<Props, State> {
  state = { rawjson: "ohea", articles: [] };
  message: string;

  loadArticles = (graphqlResponce: any) => {
    const message = JSON.stringify(graphqlResponce);
    const newArticles: typeof Article[] = graphqlResponce["data"]["articlesConnection"]["nodes"].map((node) => {
      const props = {
        id: node["id"],
        url: node["url"],
        name: node["name"],
      };
      return Article(props);
    });
    if (this.state.rawjson !== message) {
      this.setState((state) => ({
        rawjson: message,
        articles: newArticles,
      }));
    }
  }

  render() {
    client
      .query({
        query: getArticlesQuery
      })
      .then(result => this.loadArticles(result));


    return (
      <div>
        <h1>Monk</h1>
        <table>
          <tr>
            <th>Name</th>
            <th>Url</th>
            <th>Id</th>
            <th></th>
          </tr>
          {this.state.articles}
        </table>
      </div >
    )
  }
}



ReactDOM.render(<ArticleTable name="rtns" />, document.getElementById('root'))

client
  .query({
    query: getArticlesQuery
  })
  .then(result => console.log(result));