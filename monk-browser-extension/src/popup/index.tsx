import ReactDOM from "react-dom";
import { App } from "./App";
import { browser } from "webextension-polyfill-ts";
import { MsgType } from "../message";

const app = document.getElementById("app");
ReactDOM.render(<App />, app);
