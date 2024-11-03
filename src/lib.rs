use chrono::NaiveDateTime;
use regex::Regex;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub enum LogEvent {
    #[serde(alias = "price")]
    Price,
    #[serde(alias = "order")]
    Order,
    Message,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
    #[serde(alias = "open")]
    Open,
    #[serde(alias = "close")]
    Close,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OrderDirection {
    Short,
    Long,
}

impl LogEvent {
    pub fn parse(str: &str) -> Self {
        let mut params = str.split(' ');
        params.next().unwrap();
        params.next().unwrap();
        params.next().unwrap();
        params.next().unwrap();
        let Some(event) = params.next() else {
            return LogEvent::Message;
        };
        let mut event = event.split(':');

        let Some(head) = event.next() else {
            return LogEvent::Message;
        };
        if !head.starts_with("type") {
            return LogEvent::Message;
        }
        let Some(body) = event.next() else {
            return LogEvent::Message;
        };
        let x = format!("\"{}\"", body);
        serde_json::from_str(x.as_str()).unwrap()
    }
}

#[derive(Debug)]
pub struct PriceEvent {
    pub time: i64,
    pub mid: f64,
    pub open: (f64, i64),
    pub std: f64,
    pub lob: (f64, i64, f64, i64),
    pub profit: f64,
}

pub fn remove_ansi_escape_codes(text: &str) -> String {
    let ansi_escape = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    ansi_escape.replace_all(text, "").to_string()
}

pub fn into_timestamp(str: &str) -> i64 {
    // todo: NaiveDateTime的timestamp_millis方法未来会被移除
    let date_time = NaiveDateTime::parse_from_str(str, "%Y-%m-%dT%H:%M:%S%.f").unwrap();
    // 转换为时间戳（以毫秒为单位）
    let timestamp_millis = date_time.and_utc().timestamp_millis();

    timestamp_millis
}


impl PriceEvent {
    pub fn parse(str: &str) -> Option<Self> {
        let mut params = str.split(' ');
        let time = params.next().unwrap().to_string(); // 这里应该转换为时间戳
        let timestamp = into_timestamp(time.as_str());
        params.next().unwrap();
        params.next().unwrap();
        params.next().unwrap();

        let Some(_type) = params.next() else {
            return None;
        };

        let Some(mid) = params.next() else {
            return None;
        };

        let mut mid = mid.split(':');

        if mid.next().filter(|name| *name == "mid").is_none() {
            return None;
        }

        let mid = mid.next().unwrap().parse().unwrap();

        let mut open = params.next().unwrap().split(':').last().unwrap().split('-');
        let open = (
            open.next().unwrap().parse().unwrap(),
            open.next().unwrap().parse().unwrap(),
        );

        let std = params
            .next()
            .unwrap()
            .split(':')
            .last()
            .unwrap()
            .parse()
            .unwrap();

        let lob1 = params.next().unwrap().split(':').last().unwrap();

        let lob = (
            lob1.parse().unwrap(),
            params.next().unwrap().parse().unwrap(),
            params.next().unwrap().parse().unwrap(),
            params.next().unwrap().parse().unwrap(),
        );

        let profit = params
            .next()
            .unwrap()
            .split(':')
            .last()
            .unwrap()
            .parse()
            .unwrap();

        Some(Self {
            time: timestamp,
            mid,
            open,
            std,
            lob,
            profit,
        })
    }
}


#[derive(Debug)]
pub struct OrderEvent {
    pub time: i64,
    pub price: f64,
    pub volume: f64,
    pub direction: OrderDirection,
    pub action: Action,
}

impl OrderEvent {
    pub fn parse(str: &str) -> Option<OrderEvent> {
        // println!("{}", str);
        let mut params = str.split(' ');
        let time = params.next().unwrap().to_string(); // 这里应该转换为时间戳
        let timestamp = into_timestamp(time.as_str());
        params.next().unwrap();
        params.next().unwrap();
        params.next().unwrap();

        let Some(_type) = params.next() else {
            return None;
        };


        let traded_price = params
            .next()
            .unwrap()
            .split(':')
            .last()
            .unwrap()
            .parse()
            .unwrap();

        let volume = params
            .next()
            .unwrap()
            .split(':')
            .last()
            .unwrap()
            .parse()
            .unwrap();

        let direction = params
            .next()
            .unwrap()
            .split(':')
            .last()
            .unwrap().to_string();


        let action_string = params
            .next()
            .unwrap()
            .split(':')
            .last()
            .unwrap();

        let action_string = format!("\"{action_string}\"");
        let action = serde_json::from_str(action_string.as_str()).unwrap();
        let order_direction_string = format!("\"{}\"", direction);
        let order_direction = serde_json::from_str(order_direction_string.as_str()).unwrap();
        Some(Self {
            time: timestamp,
            price: traded_price,
            volume,
            direction: order_direction,
            action,
        })
    }
}