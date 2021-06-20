import * as React from "react";

type Props = {
    id: string;
    url: string;
    name: string;
};
// Function component
const Article = ({ id, url, name }: Props) => {
    return (
        <tr key={id}>
            <td>{name}</td>
            <td>{url}</td>
            <td>{id}</td>
            <td><div className="btn-group">
                <button className="btn btn-success" onClick={(event) => {
                    window.open(
                        url,
                        '_blank' // <- This is what makes it open in a new window.
                    );
                }}>Open Article</button>
                <button type="button" className="btn btn-success dropdown-toggle dropdown-toggle-split" data-bs-toggle="dropdown" aria-expanded="false">
                    <span className="visually-hidden">Toggle Dropdown</span>
                </button>
                <ul className="dropdown-menu">
                    <li><a className="dropdown-item" href="http://ohea.xyz/" target="_blank">Open Offline</a></li>
                    <li><a className="dropdown-item" href={url} target="_blank">Open Origin Webpage</a></li>
                </ul>
            </div>
            </td >
        </tr >
    )
};

export default Article;