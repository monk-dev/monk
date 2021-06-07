import * as React from "react";
import * as bootstrap from 'bootstrap';
type Props = {
    id: string;
    url: string;
    name: string;
    desc: string;
    imageLoc: string;
};

const mystyle = {
    width: "18rem",
    margin: "1em",
};
// Function component
const ArticleCard = ({ id, url, name, desc, imageLoc }: Props) => {

    return (
        <div className="card" style={mystyle}>
            <img src="https://media.tenor.com/images/bedf3f73ec3ecc20a941b86e548a8f23/tenor.gif" className="card-img-top" alt="..." />
            <div className="card-body">
                <h5 className="card-title">{name}</h5>
                <p className="card-text">{desc}</p>
            </div>
            <div className="card-body">
                <a href={url} target="_blank" className="card-link">Local Copy</a>
                <a href="#" className="card-link">Original source</a>
            </div>
        </div>
    );
};

export default ArticleCard;