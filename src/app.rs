use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::time::Duration;

use std::borrow::Cow;
use std::collections::HashMap;

use crossbeam_queue::spsc::{self, Consumer, Producer};

use super::interface::Interface;
use crate::ac::{IntoStrategy, __Strategy};
use crate::account::Account;
use crate::ctp::md_api::MdApi;
use crate::ctp::td_api::TdApi;
use crate::structs::{
    AccountData, CancelRequest, ContractData, LoginForm, OrderData, OrderRequest,
    PositionData, SubscribeRequest, TickData, TradeData,
};

/// ctpbee核心运行器
/// 作为该运行器
pub struct CtpbeeR {
    name: String,
    acc: Account,
    login_info: HashMap<String, String>,
}

impl CtpbeeR {
    pub fn new<'a>(name: impl Into<Cow<'a, str>>) -> CtpBuilder<'a> {
        CtpBuilder {
            name: name.into(),
            id: Default::default(),
            pwd: Default::default(),
            path: Default::default(),
            strategy: vec![],
            login_form: Default::default(),
        }
    }

    pub fn login(&mut self) -> bool {
        true
    }

    /// 从HashMap载入登录信息
    pub fn load_setting(&mut self, map: HashMap<String, String>) {
        let mut dt = HashMap::new();
        for (key, value) in map {
            dt.insert(key, value);
        }
        self.login_info = dt;
    }
}

impl Debug for CtpbeeR {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Ctpbee >>> : {}", self.name)
    }
}

pub struct ProducerMdApi(Vec<Producer<MdApiMessage>>, Vec<Vec<usize>>);

const OVERFLOW_PANIC_MSG: &str = "Strategy index overflow";

impl ProducerMdApi {
    // 无视分组发送给所有策略
    pub fn send_all(&self, msg: impl Into<MdApiMessage> + Clone) {
        self.0.iter().for_each(|p| {
            p.push(msg.clone().into()).unwrap();
        });
    }

    // 根据订阅分组尝试发送，失败时返回消息
    pub fn try_send_group<M>(&self, msg: M, index: usize) -> Result<(), M>
    where
        M: Into<MdApiMessage> + Clone,
    {
        match self.1.get(index) {
            Some(i) => {
                i.iter().for_each(|i| self._send(msg.clone().into(), *i));
                Ok(())
            }
            None => Err(msg),
        }
    }

    // 根据index直接发送，失败会panic
    pub fn send_to(&self, msg: impl Into<MdApiMessage>, index: usize) {
        self._send(msg.into(), index);
    }

    // 内部方法，发送给指定的索引策略
    fn _send(&self, msg: MdApiMessage, index: usize) {
        self.0
            .get(index)
            .expect(OVERFLOW_PANIC_MSG)
            .push(msg)
            .unwrap();
    }
}

pub enum MdApiMessage {
    TickData(TickData),
    AccountData(AccountData),
    PositionData(PositionData),
    ContractData(ContractData),
    SubscribeRequest(SubscribeRequest),
}

impl From<SubscribeRequest> for MdApiMessage {
    fn from(data: SubscribeRequest) -> Self {
        Self::SubscribeRequest(data)
    }
}

impl From<TickData> for MdApiMessage {
    fn from(data: TickData) -> Self {
        Self::TickData(data)
    }
}

pub enum TdApiMessage {
    OrderData(OrderData),
    TradeData(TradeData),
}

impl From<OrderData> for TdApiMessage {
    fn from(data: OrderData) -> Self {
        Self::OrderData(data)
    }
}

impl From<TradeData> for TdApiMessage {
    fn from(data: TradeData) -> Self {
        Self::TradeData(data)
    }
}

pub struct ProducerTdApi(Vec<Producer<TdApiMessage>>);

impl ProducerTdApi {
    // 无视分组发送给所有策略
    pub fn send_all(&self, msg: impl Into<TdApiMessage> + Clone) {
        self.0.iter().for_each(|p| {
            p.push(msg.clone().into()).unwrap();
        });
    }

    // 根据index直接发送，失败会panic
    pub fn send_to(&self, msg: impl Into<TdApiMessage>, index: usize) {
        self._send(msg.into(), index);
    }

    // 内部方法，发送给指定的索引策略
    fn _send(&self, msg: TdApiMessage, index: usize) {
        self.0
            .get(index)
            .expect(OVERFLOW_PANIC_MSG)
            .push(msg)
            .unwrap();
    }
}

pub enum StrategyMessage {
    OrderRequest(OrderRequest),
    CancelRequest(CancelRequest),
}

impl From<OrderRequest> for StrategyMessage {
    fn from(data: OrderRequest) -> Self {
        Self::OrderRequest(data)
    }
}

impl From<CancelRequest> for StrategyMessage {
    fn from(data: CancelRequest) -> Self {
        Self::CancelRequest(data)
    }
}

pub struct ConsumerStrategy(Vec<Consumer<StrategyMessage>>);

impl ConsumerStrategy {
    // 策略消息从此处进入td_api
    pub fn block_handle(&self, td_api: &mut TdApi) {
        loop {
            self.0
                .iter()
                .enumerate()
                .filter_map(|(idx, c)| c.pop().map(|msg| (idx, msg)).ok())
                // 此处idx为策略通道的index，可以交给回调用以确认回程消息的策略
                .for_each(|(idx, msg)| match msg {
                    StrategyMessage::OrderRequest(req) => {
                        td_api.send_order(idx.clone(), req);
                    }
                    StrategyMessage::CancelRequest(req) => {
                        td_api.cancel_order(req);
                    }
                });
        }
    }
}

pub struct CtpBuilder<'a> {
    name: Cow<'a, str>,
    id: Cow<'a, str>,
    pwd: Cow<'a, str>,
    path: Cow<'a, str>,
    strategy: Vec<__Strategy>,
    login_form: LoginForm,
}

impl<'a> CtpBuilder<'a> {
    pub fn id(mut self, id: impl Into<Cow<'a, str>>) -> Self {
        self.id = id.into();
        self
    }

    pub fn pwd(mut self, pwd: impl Into<Cow<'a, str>>) -> Self {
        self.pwd = pwd.into();
        self
    }

    pub fn path(mut self, path: impl Into<Cow<'a, str>>) -> Self {
        self.path = path.into();
        self
    }

    pub fn strategy(mut self, strategy: impl IntoStrategy) -> Self {
        self.strategy.push(strategy.into_str());
        self
    }

    pub fn strategies(mut self, strategy: Vec<__Strategy>) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn login_form(mut self, login_form: LoginForm) -> Self {
        self.login_form = login_form;
        self
    }

    pub fn start(mut self) {
        // 启动所有策略线程，并返回对应的通道制造（消费）端 和订阅symbols集合
        // * 策略已被此函数消费无法再次调用。
        let (symbols, p_md, p_td, c_st) = start_sts(&mut self);

        let id = self.id.into();
        let pwd = self.pwd.into();
        let path = self.path.into();

        // 构造api
        let mut md_api = MdApi::new(id, pwd, path, p_md);
        let mut td_api = TdApi::new("".parse().unwrap(), p_td);

        let login_form = self.login_form.into();

        // 连接
        md_api.connect(&login_form);
        td_api.connect(&login_form);

        std::thread::sleep(Duration::from_secs(1));

        // 订阅
        md_api.subscribe(symbols);

        // 此处策略消费端会阻塞线程并发送消息给td_api。
        c_st.block_handle(&mut td_api);
    }
}

fn start_sts<'a>(
    builder: &mut CtpBuilder<'a>,
) -> (
    Vec<&'static str>,
    ProducerMdApi,
    ProducerTdApi,
    ConsumerStrategy,
) {
    let sts = std::mem::take(&mut builder.strategy);

    let cap = sts.len();

    // 单向spsc:
    // MpApi -> Strategies.
    let mut producer_md = Vec::with_capacity(cap);
    // Strategies -> TdApi.
    let mut producer_td = Vec::with_capacity(cap);
    // TdApi -> Strategies.
    let mut consumer_st = Vec::with_capacity(cap);

    // symbols为订阅symbol &str的非重复vec集合
    let mut symbols = Vec::new();
    // groups为与symbols相对应(vec index)的策略们的发送端vec，该vec保存策略发送端在producer_md vec中的index
    let mut groups: Vec<Vec<usize>> = Vec::new();

    sts.into_iter().enumerate().for_each(|(st_index, st)| {
        st.symbols().into_iter().for_each(|symbol| {
            symbols
                .iter()
                .enumerate()
                .find_map(|(index, s)| if s == symbol { Some(index) } else { None })
                .map(|index| {
                    groups.get_mut(index).unwrap().push(st_index);
                })
                .unwrap_or_else(|| {
                    groups.push(vec![st_index]);
                    symbols.push(*symbol);
                });
        });

        // 通道容量暂设为128.如果单tick中每个策略的消息数量超过这个数值（或者有消息积压），可以考虑放松此上限。
        // 只影响内存占用。
        // MpApi -> Strategies.
        let (p_md, c_md) = spsc::new(128);

        // Strategies -> TdApi.
        let (p_st, c_st) = spsc::new(128);

        // TdApi -> Strategies.
        let (p_td, c_td) = spsc::new(128);

        let mut worker = StrategyWorker::new(st, c_md, c_td, p_st);

        std::thread::spawn(move || worker.start());

        producer_md.push(p_md);
        producer_td.push(p_td);
        consumer_st.push(c_st);
    });

    (
        symbols,
        ProducerMdApi(producer_md, groups),
        ProducerTdApi(producer_td),
        ConsumerStrategy(consumer_st),
    )
}

struct StrategyWorker {
    st: __Strategy,
    c_md: Consumer<MdApiMessage>,
    c_td: Consumer<TdApiMessage>,
    p_st: Producer<StrategyMessage>,
}

impl StrategyWorker {
    fn new(
        st: __Strategy,
        c_md: Consumer<MdApiMessage>,
        c_td: Consumer<TdApiMessage>,
        p_st: Producer<StrategyMessage>,
    ) -> Self {
        Self {
            st,
            c_md,
            c_td,
            p_st,
        }
    }

    fn start(&mut self) {
        loop {
            // 接收md_api的消息并处理。
            match self.c_md.pop() {
                Ok(msg) => match msg {
                    // tick会返回订单vec，我们转发至td api
                    MdApiMessage::TickData(data) => self
                        .st
                        .on_tick(&data)
                        .into_iter()
                        .for_each(|m| self.p_st.push(m).unwrap()),
                    MdApiMessage::AccountData(data) => self.st.on_account(&data),
                    MdApiMessage::PositionData(data) => self.st.on_position(&data),
                    MdApiMessage::ContractData(data) => self.st.on_contract(&data),
                    _ => {}
                },
                Err(_) => (),
            }
            // 接收td_api的消息并处理。
            match self.c_td.pop() {
                Ok(msg) => match msg {
                    TdApiMessage::OrderData(data) => self.st.on_order(&data),
                    TdApiMessage::TradeData(data) => self.st.on_trade(&data),
                },
                Err(_) => (),
            }
        }
    }
}

impl Drop for StrategyWorker {
    fn drop(&mut self) {
        // worker退出时如果线程panic则恢复运行
        if std::thread::panicking() {
            self.start();
        }
    }
}
