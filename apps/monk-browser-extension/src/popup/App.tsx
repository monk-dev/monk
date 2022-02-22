// function sendTabInfoReq(tabs)

import { useState } from "react";
import { useEffect } from "react/cjs/react.production.min";
import { Browser, browser } from "webextension-polyfill-ts";
import { Msg, MsgType } from "../message";
import { Monk } from "./components/Monk";
import { MonkProvider } from "./context/MonkContext";

async function sendGetPageInfo() {
  const tabs = await browser.tabs.query({
    active: true,
    currentWindow: true,
  });
  browser.tabs.sendMessage(tabs[0].id, {
    type: MsgType.GetPageInfo,
    payload: undefined,
  });
}

export function App() {
  return (
    <MonkProvider>
      <Monk />
    </MonkProvider>
  );
}
