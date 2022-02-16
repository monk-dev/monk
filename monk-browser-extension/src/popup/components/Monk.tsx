import { Browser, browser } from "webextension-polyfill-ts";
import { Msg, MsgType } from "../../message";
import { useMonkContext } from "../context/MonkContext";

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

export function Monk() {
  const { pageInfo, monkUrl, error, setPageInfo, uploadToMonk } =
    useMonkContext();

  if (error) {
    return <>Error: {error}</>;
  }

  return (
    <>
      <p className="font-mono">Add to Monk?</p>
      <pre>{JSON.stringify(pageInfo, null, 4)}</pre>
      <button
        className="border-2 border-black p-0.5 rounded"
        onClick={() => {
          uploadToMonk();
        }}
      >
        Upload to Monk
      </button>
    </>
  );
}
