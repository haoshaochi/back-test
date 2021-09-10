use std::collections::HashMap;
use std::fs::{File, OpenOptions, read_dir};
use std::io::Write;
use std::thread;

use flate2::read::GzDecoder;
use tar::Archive;
use std::ops::{Div};
use back_test::{BackResult, Deal, MinData, config, FinalResult};

///
/// 根据分钟k先数据文件回测交易策略
///
fn main() {
    //读取数据
    let all_data = extract();

    //回测数据
    let final_result = back_test(all_data);

    //详细结果(包含交易流水)落地
    write_result(final_result)
}

///
/// 输出结果，并将交易明细写入文件
///
fn write_result(final_result: FinalResult) {
    println!("------------------RESULT-----------------------");
    let result_path = "./result.data";
    File::create(result_path).unwrap();
    let mut file = OpenOptions::new().append(true).open(result_path).expect("cannot open file");
    file.write("[tips: buy_time(sell_time): minutes after base time 2021-06-23 00:00:00]\n\n".as_bytes()).unwrap();
    file.write(format!("成功率:{}\t平均收益率:{}\n\n",final_result.suc_ratio(),final_result.avg_profit()).as_bytes()).unwrap();
    for x in final_result.result_vec() {
        if x.deal_vec().len() > 0 {
            println!("{}", x.to_line());
            file.write(x.to_line().as_bytes()).unwrap();
            file.write("\n-----------------------------\n".as_bytes()).unwrap();
            for d in x.deal_vec() {
                file.write(d.to_line().as_bytes()).unwrap();
            }
            file.write("\n-----------------------------\n".as_bytes()).unwrap();
        }
    }
    println!("交易详情见文件:{}", result_path);
}


///
/// 回测数据,多线程并行执行
/// 不同type之间可并行计算
///
fn back_test(all_date: HashMap<String, HashMap<i64, String>>) -> FinalResult{
    println!("back start...");
    // 并发线程数
    let thread_size = 8;
    let mut i = 0;
    let mut conditions = Vec::new();

    let mut suc_sum = 0.0;
    let mut value_sum = 0.0;

    //回溯结果
    let mut result_vec: Vec<BackResult> = Vec::new();

    for (k, v) in all_date {
        i = i + 1;
        let p = thread::spawn(move || {
            back_by_type(k, v)
        });
        conditions.push(p);

        if i % thread_size == 0 {
            // println!("condition size:{},wait...", conditions.len());
            for x in conditions {
                let result = x.join().unwrap();
                value_sum = value_sum + result.value();
                if !result.suc_ratio().is_nan() {
                    suc_sum = suc_sum + result.suc_ratio();
                }
                result_vec.push(result);
            }
            conditions = Vec::new();
        }
    }

    println!("back end...");
    let len = result_vec.len() as f64;
    FinalResult::new(result_vec,value_sum.div(&len) - 1.0,suc_sum.div(&len))
}


///
/// 回测指定type的数据
///
fn back_by_type(k: String, v: HashMap<i64, String>) -> BackResult {
    println!("async back :{}...", k);
    let mut deal_vec: Vec<Deal> = Vec::new();
    // 净值
    let mut value = 1.0;
    // 成功次数
    let mut suc_cnt = 0.0;
    for (t, p) in &v {
        let fp = p.parse::<f64>().unwrap();

        if let Some(sp) = v.get(&(t + 1)) {
            let sp = sp.parse::<f64>().unwrap();

            if sp.div(fp).ge(&1.01) {
                if let Some(tp) = v.get(&(t + 2)) {
                    let tp = tp.parse::<f64>().unwrap();
                    let profit = tp / sp - 1.0;
                    let source: Vec<_> = k.split("-").collect();
                    let deal = Deal::new(sp, tp, t + 1, t + 2, source.get(0).unwrap().to_string(),
                                         source.get(1).unwrap().to_string(), source.get(2).unwrap().to_string(), profit);

                    // println!("{:#?}", deal);
                    value = value * (1.0 + profit);
                    if profit.ge(&0.0) {
                        suc_cnt = suc_cnt + 1.0;
                    }
                    deal_vec.push(deal);
                }
            }
        }
    }
    // println!("async back :{} end",k);
    let len = deal_vec.len() as f64;
    let mut suc_ratio = 0.0;
    if len.ge(&0.0){
        suc_ratio = suc_cnt.div(len);
    }

    BackResult::new(k, deal_vec, value, suc_ratio)
}

///
/// 从压缩文件获取数据，并放入map
///
fn extract() -> HashMap<String, HashMap<i64, String>> {
    println!("extract start...");
    let path = "v3_kline_2021_06_23.tar.gz";
    let xz_path = "./v3_kline_2021_06_23";

    let tar_gz = File::open(path).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(".").expect("failed to unpack");

    let content_paths = read_dir(&xz_path).unwrap();


    let mut all_data: HashMap<String, HashMap<i64, String>> = HashMap::new();

    for path in content_paths {
        let file_p = path.unwrap().path();

        if file_p.extension().is_some() && file_p.extension().unwrap().to_str().unwrap().eq("xz") {
            println!("extract {}....", file_p.to_str().unwrap());
            // println!("deal with {}",file_p.to_str().unwrap());
            let mut f = std::io::BufReader::new(std::fs::File::open(file_p).unwrap());
            let mut decomp: Vec<u8> = Vec::new();
            lzma_rs::xz_decompress(&mut f, &mut decomp).unwrap();
            let content = std::str::from_utf8(&decomp).unwrap();

            let lines: Vec<&str> = content.split("\n").collect();


            for i in 0..lines.len() {
                let l = lines.get(i);

                if l.is_some() && l.unwrap().to_string().len() > 0 {
                    let line: Vec<&str> = lines.get(i).unwrap().split("\t").collect();
                    // println!("{:?}",line);
                    let data = generate_min_data(&line);
                    let id = data.get_id();

                    if let Some(x) = all_data.get_mut(id.as_str()) {
                        x.insert(data.get_time(), data.get_price());
                    } else {
                        let mut map: HashMap<i64, String> = HashMap::new();
                        map.insert(data.get_time(), data.get_price());
                        all_data.insert(id, map);
                    }
                    // println!("{}|{}|{}",data.get_id(),data.get_time(),data.get_price());
                }
            }
        }
    }
    println!("extract end, len:{}...", all_data.len());
    all_data
}

///
/// 根据文件行内容生成 分钟数据
///
fn generate_min_data(line: &Vec<&str>) -> MinData {
    MinData::new(line.get(9).unwrap().to_string(), (line.get(5).unwrap().parse::<i64>().unwrap() - config::BASE_TIME) / config::MILLIS_IN_MIN, line.get(2).unwrap().to_string(), line.get(3).unwrap().to_string(), line.get(4).unwrap().to_string())
}
