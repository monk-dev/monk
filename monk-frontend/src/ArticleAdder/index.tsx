import * as React from "react";
import * as bootstrap from 'bootstrap';
type Props = {}
type State = {
  url: string;
};

// Function component
class ArticleAdder extends React.Component<Props, State> {
  state = {
    url: "",
  };
  onChange = (e: React.FormEvent<HTMLInputElement>): void => {
    this.setState({ url: e.currentTarget.value });
  };
  render() {
    return (
      <div className="container-fluid" >
        <form>
          <div className="row g-2">
            <div className="form-floating">
              <input type="URL" className="form-control" value={this.state.url} id="floatingInput" placeholder="example.com" onChange={this.onChange}></input>
              <label htmlFor="floatingInput">Url</label>
            </div>
            <div className="col-md">
              <div className="form-floating">
                <input type="email" className="form-control" id="floatingInput" placeholder="name@example.com"></input>
                <label htmlFor="floatingInput">Title</label>
              </div>
            </div>
            <div className="col-md">
              <div className="form-floating">
                <input type="password" className="form-control" id="floatingPassword" placeholder="Password"></input>
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
        <iframe src={this.state.url} title="W3Schools Free Online Web Tutorials" sandbox="" height="500" width="100%" ></iframe>
      </div>
    )
  }
}

export default ArticleAdder;