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
import ArticleCard from './ArticleCard';
import ArticleAdder from './ArticleAdder'
import './monk.css';
// Import all plugins
import * as bootstrap from 'bootstrap';

let getArticlesQuery = gql`
query AllArticles {
  articles {
    id
    name
    url
  }
}
`

let getArticleCardsQuery = gql`
query CardInfo {
  articles {
    id
    name
    description
    url
  }
}
`



const client = new ApolloClient({
  uri: 'http://localhost:5555/graphql/',
  cache: new InMemoryCache()
});


type Props = {
  name: string,
};

type State = {
  rawjson: string,
  View: string,
  articles: typeof Article[],
  articleCards: typeof ArticleCard[],
};

class App extends React.Component<Props, State> {
  state = { rawjson: "ohea", View: "card", articles: [], articleCards: [] };
  message: string;

  loadArticles = (graphqlResponce: any) => {
    const message = JSON.stringify(graphqlResponce);
    const newArticles: typeof Article[] = graphqlResponce["data"]["articles"].map((article) => {
      const props = {
        id: article["id"],
        url: article["url"],
        name: article["name"],
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
    const newArticleCards: typeof ArticleCard[] = graphqlResponce["data"]["articles"].map((article) => {
      const props = {
        id: article["id"],
        url: article["url"],
        name: article["name"],
        desc: article["description"],
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

  change_view_to_card = () => {
    this.setState((state) => ({
      View: "card",
    }))
  }
  change_view_to_table = () => {
    this.setState((state) => ({
      View: "table",
    }))
  }
  change_view_to_adder = () => {
    this.setState((state) => ({
      View: "adder",
    }))
  }

  render() {
    var navbar = (
      <nav className="navbar navbar-expand-lg navbar-light bg-light">
        <div className="container-fluid">
          <a className="navbar-brand" href="#">Monk</a>
          <button className="navbar-toggler" onClick={this.change_view_to_card} type="button" data-bs-toggle="collapse" data-bs-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation" >
            <span className="navbar-toggler-icon"></span>
          </button>
          <div className="collapse navbar-collapse" id="navbarSupportedContent">
            <ul className="navbar-nav me-auto mb-2 mb-lg-0">
              <li className="nav-item">
                <a className="nav-link" href="#" onClick={this.change_view_to_card} aria-current="page">Articles</a>
              </li>
              <li className="nav-item">
                <a className="nav-link" href="#" onClick={this.change_view_to_adder}>Add new Article</a>
              </li>
              <li className="nav-item">
                <a className="nav-link" href="#" onClick={this.change_view_to_table}>View Table</a>
              </li>
              {/*<li className="nav-item dropdown">
        <a className="nav-link dropdown-toggle" href="#" id="navbarDropdown" role="button" data-bs-toggle="dropdown" aria-expanded="false">
          Dropdown
        </a>
      </li>*/}
            </ul>
            <ul className="dropdown-menu" aria-labelledby="navbarDropdown">
              <li><a className="dropdown-item" href="#">Action</a></li>
              <li><a className="dropdown-item" href="#">Another action</a></li>
              <li><hr className="dropdown-divider" /></li>
              <li><a className="dropdown-item" href="#">Something else here</a></li>
              <li className="nav-item">
                <a className="nav-link disabled" href="#" tab-index="-1" aria-disabled="true">Disabled</a>
              </li>
            </ul>
            <form className="d-flex">
              <input className="form-control me-2" type="search" placeholder="Search" aria-label="Search" />
              <button className="btn btn-outline-success" type="submit">Search</button>
            </form>
          </div>
        </div>
      </nav >
    )

    if (this.state.View === "card") {
      client
        .query({
          query: getArticleCardsQuery
        })
        .then(result => this.loadArticleCards(result));
      return [
        navbar,
        <div className="container d-flex justify-content-center ">
          <div className="row row-cols-3">{this.state.articleCards}</div>
        </div>
      ];
    } else if (this.state.View === "table") {
      client
        .query({
          query: getArticlesQuery
        })
        .then(result => this.loadArticles(result));


      return [
        navbar,
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
      ]
    } else if (this.state.View = "adder") {
      return [
        navbar,
        <ArticleAdder />
      ]
    }
  }
}



ReactDOM.render(<App name="rtns" />, document.getElementById('root'))

client
  .query({
    query: getArticlesQuery
  })
  .then(result => console.log(result));