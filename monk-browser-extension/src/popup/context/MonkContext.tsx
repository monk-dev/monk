import React, { useContext, useEffect, useState } from "react";
import { browser } from "webextension-polyfill-ts";
import { Msg, MsgType } from "../../message";
import { PageInfo } from "../../page_info";

interface MonkContextData {
  pageInfo: PageInfo;
  monkUrl: string;
  setPageInfo: React.Dispatch<PageInfo>;
  uploadToMonk: () => void;
  error?: any;
}

const MonkContext = React.createContext<MonkContextData>(null);
export const useMonkContext = () => useContext(MonkContext);

export const MonkProvider = ({
  children,
}: {
  children: React.PropsWithChildren<{}>;
}) => {
  const monkUrl = "http://localhost:3000";
  const [pageInfo, setPageInfo] = useState<PageInfo>({
    title: "This is a title",
    url: "http://localhost:3000",
  });
  const [error, setError] = useState(null);

  const listener = (msg: Msg) => {
    setPageInfo(msg.payload);
  };

  if (!browser.runtime.onMessage.hasListener(listener)) {
    browser.runtime.onMessage.addListener(listener);
  }

  const uploadToMonk = () => {
    fetch(monkUrl + "/items", {
      method: "POST",
      body: JSON.stringify({
        name: pageInfo.title,
        url: pageInfo.url,
        tags: [],
      }),
      headers: {
        "Content-Type": "application/json",
      },
    }).catch((e) => {
      setError(e);
    });
  };

  useEffect(() => {
    sendGetPageInfo();
  }, [document.location.href]);

  return (
    <MonkContext.Provider
      value={{ pageInfo, monkUrl, setPageInfo, uploadToMonk, error }}
    >
      {children}
    </MonkContext.Provider>
  );
};

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
