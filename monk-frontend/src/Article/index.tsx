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
                <button type="button" className="btn btn-success dropdown-toggle dropdown-toggle-split" dataBsToggle="dropdown" ariaExpanded="false">
                    <span className="visually-hidden">Toggle Dropdown</span>
                </button>
                <ul className="dropdown-menu">
                    <li><a className="dropdown-item" href="ohea.xyz">Action</a></li>
                    <li><a className="dropdown-item" href="#">Another action</a></li>
                </ul>
            </div>
            </td >
        </tr >
    )
};

export default Article;