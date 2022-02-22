

export enum MsgType {
    GetPageInfo = "GetPageInfo",
    PageInfo = "PageInfo",
};

export interface Msg {
    type: MsgType,
    payload: any,
}