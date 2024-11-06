use std::sync::LazyLock;

use chrono::{NaiveDateTime, Timelike};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub fn parse(string: String) -> Vec<Tick> {
    let mut arrive = vec![];
    let mut profit_summary = 0.0;
    let mut last_minute = 0;

    let mut tick = vec![];
    for line in string.lines() {
        let line = remove_ansi_escape_codes(line);
        let event = LogEvent::parse(line.as_str());
        match event {
            LogEvent::Price => {
                let price_event = PriceEvent::parse(line.as_str()).unwrap();
                let minute = price_event.time.minute();
                if minute != last_minute {
                    tick.push(Tick {
                        time: price_event.time,
                        mid_price: price_event.mid,
                        profit: profit_summary,
                    });
                    last_minute = minute;
                }
            }
            LogEvent::Order => {
                let order_event = OrderEvent::parse(line.as_str()).unwrap();
                let price = match order_event.direction {
                    OrderDirection::Short => order_event.price,
                    OrderDirection::Long => -order_event.price,
                };
                let volume = match order_event.action {
                    Action::Open => (order_event.volume * 1000.0) as i64,
                    Action::Close => (-order_event.volume * 1000.0) as i64,
                };
                arrive.push((price, volume));
                let volume_add = arrive.iter().map(|(_, volume)| *volume).sum::<i64>();
                if (arrive.len() > 0) & volume_add.eq(&0) {
                    let profit = arrive
                        .iter()
                        .map(|(price, volume)| price * volume.abs() as f64 / 1000.0)
                        .sum::<f64>();
                    profit_summary += (profit * 10000.0).round() / 10000.0;
                    arrive.clear();
                }
            }
            _ => {}
        }
    }
    tick
}

#[derive(Debug)]
pub struct Tick {
    pub time: NaiveDateTime,
    pub mid_price: f64,
    pub profit: f64,
}

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
    pub time: NaiveDateTime,
    pub mid: f64,
    pub open: (f64, i64),
    pub std: f64,
    pub lob: (f64, i64, f64, i64),
    pub profit: f64,
}

static R: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap());

pub fn remove_ansi_escape_codes(text: &str) -> String {
    R.replace_all(text, "").to_string()
}

impl PriceEvent {
    pub fn parse(str: &str) -> Option<Self> {
        let mut params = str.split(' ');
        let time = params.next().unwrap();
        let time = NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M:%S%.f").unwrap();
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
            time,
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
    pub time: NaiveDateTime,
    pub price: f64,
    pub volume: f64,
    pub direction: OrderDirection,
    pub action: Action,
}

impl OrderEvent {
    pub fn parse(str: &str) -> Option<OrderEvent> {
        // println!("{}", str);
        let mut params = str.split(' ');
        let time = params.next().unwrap();
        let time = NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M:%S%.f").unwrap();
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
            .unwrap()
            .to_string();

        let action_string = params.next().unwrap().split(':').last().unwrap();

        let action_string = format!("\"{action_string}\"");
        let action = serde_json::from_str(action_string.as_str()).unwrap();
        let order_direction_string = format!("\"{}\"", direction);
        let order_direction = serde_json::from_str(order_direction_string.as_str()).unwrap();
        Some(Self {
            time,
            price: traded_price,
            volume,
            direction: order_direction,
            action,
        })
    }
}
