import { browser, Runtime } from "webextension-polyfill-ts"
import { Msg, MsgType } from "../message";

(function() {
    console.log("running monk background script");

    browser.runtime.onMessage.addListener((msg: Msg, sender: Runtime.MessageSender) => {
        console.log({ msg });
        
        switch (msg.type) {
            case MsgType.GetPageInfo:
                browser.runtime.sendMessage({
                    type: MsgType.PageInfo,
                    payload: {
                        title: document.title,
                        url: document.location.href,
                    }
                });

                break;
            default:
                console.warn("unknown message type: ", msg.type)
        }
    })

})();