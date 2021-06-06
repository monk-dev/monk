import * as React from "react";

type Props = {
    id: string;
    url: String;
    name: String;
};

// Function component
const Article = ({ id, url, name }: Props) => {
    return (
        <tr>
            <td>{name}</td>
            <td>{url}</td>
            <td>{id}</td>
            <td><a href={url}><button>Open Article</button></a></td >
        </tr >
    )
};

export default Article;