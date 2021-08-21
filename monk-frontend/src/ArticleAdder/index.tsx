import * as React from "react";
type Props = {}
type State = {
  url: string;
  description: string;
  name: string;
};

import Article from "../Article";

import {
  ApolloClient,
  InMemoryCache,
  ApolloProvider,
  useMutation,
  gql
} from "@apollo/client";

let ADD_ARTICLE = gql`
mutation CreateArticle($art: CreateArticleInput) {
  createArticle(input: $art) {
      name
      description
      url
    }
  }
`

const client = new ApolloClient({
  uri: 'http://localhost:5555/graphql',
  cache: new InMemoryCache()
});

// Function component
class ArticleAdder extends React.Component<Props, State> {
  state = {
    url: "",
    description: "",
    name: ""
  };

  UrlOnChange = (e: React.FormEvent<HTMLInputElement>): void => {
    this.setState({ url: e.currentTarget.value });
  };

  NameOnChange = (e: React.FormEvent<HTMLInputElement>): void => {
    this.setState({ name: e.currentTarget.value });
  };

  DescriptionOnChange = (e: React.FormEvent<HTMLInputElement>): void => {
    this.setState({ description: e.currentTarget.value });
  };

  onSubmit = (e: React.SyntheticEvent) => {
    client.mutate({
      mutation: ADD_ARTICLE,
      variables: {
        "art": {
          "name": this.state.name,
          "description": this.state.description,
          "url": this.state.url
        }
      }
    })
      .then(result => console.log(result)) //TODO Remove Logging
  };

  render() {
    return (
      <div className="container-fluid" >
        <form onSubmit={this.onSubmit}>
          <div className="row g-2">
            <div className="form-floating">
              <input type="URL" className="form-control" value={this.state.url} id="floatingInput" placeholder="example.com" onChange={this.UrlOnChange}></input>
              <label htmlFor="floatingInput">Url</label>
            </div>
            <div className="col-md">
              <div className="form-floating">
                <input onChange={this.NameOnChange} className="form-control" id="floatingInput" placeholder="Cool Article"></input>
                <label htmlFor="floatingInput">Title</label>
              </div>
            </div>
            {/* TODO Make this smarter by adding comma seperated values, and auto suggestions for existing tags */}
            <div className="col-md">
              <div className="form-floating">
                <input className="form-control" id="floatingPassword" placeholder="Password"></input>
                <label htmlFor="floatingPassword">Tags</label>
              </div>
            </div>
            <div className="row g-2">
              <div className="form-floating">
                <input onChange={this.DescriptionOnChange} className="form-control" id="floatingPassword" placeholder="Password"></input>
                <label htmlFor="floatingPassword">Description</label>
              </div>
            </div>
            <div>
              <button type="submit" className="btn btn-primary mb-3">Add to monk</button>
            </div>
          </div>
        </form >
        {/* This needs to be cleaned up so that it only shows with valid urls that allow themselves to be displayed in iframes */}
        <h2>Preview</h2>
        <iframe src={this.state.url} title="Preview window" sandbox="" height="500" width="100%" ></iframe>
      </div>
    )
  }
}

export default ArticleAdder;