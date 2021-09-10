use std::ops::Mul;


pub mod config {
    ///起始时间:2021-06-23 00:00:00
    pub const BASE_TIME: i64 = 1624377600000;
    pub const MILLIS_IN_MIN: i64 = 60000;
}

///
/// 分钟数据
///
#[derive(Debug)]
pub struct MinData {
    price: String,
    time: i64,
    exchange_id: String,
    pre_coin: String,
    post_coin: String,
}

impl MinData {
    pub fn new(price: String, time: i64, exchange_id: String, pre_coin: String, post_coin: String) -> MinData {
        MinData {
            price,
            time,
            exchange_id,
            pre_coin,
            post_coin,
        }
    }
    //根据 exchange_id, pre_coin, post_coin 生成编号
    pub fn get_id(&self) -> String {
        format!("{}-{}-{}", self.exchange_id, self.pre_coin, self.post_coin)
    }
    pub fn get_time(&self) -> i64 {
        self.time
    }
    pub fn get_price(&self) -> String {
        format!("{}", self.price)
    }
}

///
/// 回测结果
///
#[derive(Debug)]
pub struct FinalResult {
    result_vec: Vec<BackResult>,
    // 平均收益率
    avg_profit: f64,
    // 成功率
    suc_ratio: f64,
}
impl FinalResult{
    pub fn new(result_vec: Vec<BackResult>,avg_profit: f64,suc_ratio: f64) -> FinalResult {
        FinalResult {
            result_vec,
            avg_profit,
            suc_ratio,
        }
    }

    pub fn result_vec(&self) -> &Vec<BackResult> {
        &(self.result_vec)
    }

    pub fn avg_profit(&self) -> &f64 {
        &(self.avg_profit)
    }
    pub fn suc_ratio(&self) -> &f64 {
        &(self.suc_ratio)
    }
}

///
/// 单交易类型回溯结果
///
#[derive(Debug)]
pub struct BackResult {
    id: String,
    deal_vec: Vec<Deal>,
    value: f64,
    suc_ratio: f64,
}

impl BackResult {
    pub fn new(id: String, deal_vec: Vec<Deal>, value: f64,suc_ratio: f64) -> BackResult {
        BackResult {
            id,
            deal_vec,
            value,
            suc_ratio
        }
    }
    pub fn to_line(&self) -> String {
        format!("deal_type:{} npv:{} deal_cnt:{} suc_ratio:{}", self.id, self.value, self.deal_vec.len(),self.suc_ratio)
    }

    pub fn deal_vec(&self) -> &Vec<Deal> {
        &(self.deal_vec)
    }
    pub fn value(&self) -> &f64 {
        &(self.value)
    }
    pub fn suc_ratio(&self) -> &f64 {
        &(self.suc_ratio)
    }
}


///
/// 交易
/// buy_time 和 sell_time都用距离基准时间(2021-06-23 00:00:00)的分钟数表示
///
#[derive(Debug)]
pub struct Deal {
    buy_price: f64,
    sell_price: f64,
    buy_time: i64,
    sell_time: i64,
    exchange_id: String,
    pre_coin: String,
    post_coin: String,
    profit: f64,
}

impl Deal {
    pub fn new(buy_price: f64, sell_price: f64, buy_time: i64, sell_time: i64, exchange_id: String, pre_coin: String, post_coin: String, profit: f64) -> Deal {
        Deal {
            buy_price,
            sell_price,
            buy_time,
            sell_time,
            exchange_id,
            pre_coin,
            post_coin,
            profit,
        }
    }
    pub fn to_line(&self) -> String {
        //格式化输出，收益率*100% 并保留3位小数
        format!("buy_price:{}\tsell_price:{}\tbuy_time:{}\tsell_time:{}\texchange_id:{}\tpre_coin:{}\tpost_coin:{}\tprofit:{:.3}%\n",
                self.buy_price, self.sell_price, self.buy_time, self.sell_time, self.exchange_id, self.pre_coin, self.post_coin, self.profit.mul(100.0))
    }

    pub fn get_id(&self) -> String {
        format!("{}-{}-{}", self.exchange_id, self.pre_coin, self.post_coin)
    }
}