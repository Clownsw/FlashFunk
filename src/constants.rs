//! # General Enum
//! Author: Aaron Qiu
#![allow(dead_code)]

use derive_more::Display;

/// Direction of order/trade/position.
#[derive(Clone, Debug, PartialEq, Display)]
pub enum Direction {
    /// 多
    #[display(fmt = "LONG")]
    LONG,
    /// 净
    #[display(fmt = "NET")]
    NET,
    /// 空
    #[display(fmt = "SHORT")]
    SHORT,
}

/// 对冲
#[derive(Clone, Debug, PartialEq, Display)]
pub enum Offset {
    /// 无
    #[display(fmt = "None")]
    NONE,
    /// 平
    #[display(fmt = "CLOSE")]
    CLOSE,
    /// 平今
    #[display(fmt = "CLOSETODAY")]
    CLOSETODAY,
    /// 平昨
    #[display(fmt = "CLOSEYESTERDAY")]
    CLOSEYESTERDAY,
    /// 开
    #[display(fmt = "OPEN")]
    OPEN,
}

/// 状态
#[derive(Debug, PartialEq, Display, Clone)]
pub enum Status {
    #[display(fmt = "Submiting")]
    SUBMITTING,
    #[display(fmt = "NotTraded")]
    NOTTRADED,
    #[display(fmt = "PartTraded")]
    PARTTRADED,
    #[display(fmt = "AllTraded")]
    ALLTRADED,
    #[display(fmt = "Cancelled")]
    CANCELLED,
    #[display(fmt = "Rejected")]
    REJECTED,
}

/// 产品
#[derive(Clone, Debug, PartialEq, Display)]
pub enum Product {
    /// 股票
    #[display(fmt = "EQUITY")]
    EQUITY,
    /// 期货
    #[display(fmt = "FUTURES")]
    FUTURES,
    /// 期权
    #[display(fmt = "OPTION")]
    OPTION,
}

/// 订单类型
#[derive(Clone, Debug, PartialEq, Display)]
pub enum OrderType {
    /// 限价
    #[display(fmt = "LIMIT")]
    LIMIT,
    /// 市价
    #[display(fmt = "MARKET")]
    MARKET,
    /// STOP
    #[display(fmt = "STOP")]
    STOP,
    /// FAK
    #[display(fmt = "FAK")]
    FAK,
    /// FOK
    #[display(fmt = "FOK")]
    FOK,
    /// 询价
    #[display(fmt = "RFQ")]
    RFQ,
}

/// 期权类型
#[derive(Clone, Debug, PartialEq, Display)]
pub enum OptionType {
    /// 看涨期权
    #[display(fmt = "CALL")]
    CALL,
    /// 看跌期权
    #[display(fmt = "PUT")]
    PUT,
}

/// 交易所
/// 暫時只支持國內期貨交易
#[derive(Clone, Debug, PartialEq, Display)]
pub enum Exchange {
    // Chinese
    #[display(fmt = "CFETS")]
    CFETS,
    #[display(fmt = "CFFEX")]
    CFFEX,
    #[display(fmt = "CZCE")]
    CZCE,
    #[display(fmt = "DCE")]
    DCE,
    #[display(fmt = "INE")]
    INE,
    #[display(fmt = "SGE")]
    SGE,
    #[display(fmt = "SHFE")]
    SHFE,
    #[display(fmt = "SSE")]
    SSE,
    #[display(fmt = "SZSE")]
    SZSE,
    #[display(fmt = "WXE")]
    WXE,
}
