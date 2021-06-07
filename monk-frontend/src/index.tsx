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
import ArticleCard from './ArticleCard'
import './monk.css'
// Import all plugins
import * as bootstrap from 'bootstrap';

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

let getArticleCardsQuery = gql`
query CardInfo {
  articlesConnection {
    nodes {
      description
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
  cardView: boolean,
  articles: typeof Article[],
  articleCards: typeof ArticleCard[],
};

class ArticleTable extends React.Component<Props, State> {
  state = { rawjson: "ohea", cardView: false, articles: [], articleCards: [] };
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

  loadArticleCards = (graphqlResponce: any) => {
    const message = JSON.stringify(graphqlResponce);
    const newArticleCards: typeof ArticleCard[] = graphqlResponce["data"]["articlesConnection"]["nodes"].map((node) => {
      const props = {
        id: node["id"],
        url: node["url"],
        name: node["name"],
        desc: node["description"],
        imageLoc: "https://media.tenor.com/images/bedf3f73ec3ecc20a941b86e548a8f23/tenor.gif"
      };
      return ArticleCard(props);
    });
    if (this.state.rawjson !== message) {
      this.setState((state) => ({
        rawjson: message,
        articleCards: newArticleCards,
      }));
    }
  }

  handle_mode_change = () => {
    this.setState((state) => ({
      cardView: !state.cardView,
    }))
  }

  render() {

    if (this.state.cardView) {
      client
        .query({
          query: getArticleCardsQuery
        })
        .then(result => this.loadArticleCards(result));
      return (
        <div>
          <h1 className="display-1">Monk</h1>
          <button onClick={this.handle_mode_change} >Change View</button>
          <div className="container d-flex justify-content-center ">
            <div className="row row-cols-3">{this.state.articleCards}</div>
          </div>
        </div >
      )
    } else {
      client
        .query({
          query: getArticlesQuery
        })
        .then(result => this.loadArticles(result));


      return (
        <div>
          <h1>Monk</h1>
          <button onClick={this.handle_mode_change} >Change View</button>
          <table className="table table-hover">
            <thead>
              <tr key="top">
                <th>Name</th>
                <th>Url</th>
                <th>Id</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {this.state.articles}
            </tbody>
          </table >
        </div >
      )
    }
  }
}



ReactDOM.render(<ArticleTable name="rtns" />, document.getElementById('root'))

client
  .query({
    query: getArticlesQuery
  })
  .then(result => console.log(result));