use std::io::Read;
use clatter::{PriceEvent, LogEvent, remove_ansi_escape_codes, OrderEvent, Action, OrderDirection};


pub fn parse_log(string: String) -> Vec<(i64, f64, f64)> {
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
                let minute = price_event.time / 1000 / 60;
                if minute != last_minute {
                    tick.push((price_event.time, price_event.mid, profit_summary));
                    last_minute = minute;
                }
            }
            LogEvent::Order => {
                let order_event = OrderEvent::parse(line.as_str()).unwrap();
                let price = match order_event.direction {
                    OrderDirection::Short => order_event.price,
                    OrderDirection::Long => -order_event.price
                };
                let volume = match order_event.action {
                    Action::Open => (order_event.volume * 1000.0) as i64,
                    Action::Close => (-order_event.volume * 1000.0) as i64
                };
                arrive.push((price, volume));
                let volume_add = arrive.iter().map(|(_, volume)| *volume).sum::<i64>();
                if (arrive.len() > 0) & volume_add.eq(&0) {
                    let profit = arrive.iter().map(|(price, volume)| {
                        price * volume.abs() as f64 / 1000.0
                    }).sum::<f64>();
                    profit_summary += (profit * 10000.0).round() / 10000.0;
                    arrive.clear();
                }
            }
            _ => {}
        }
    }
    tick
}

fn main() {
    let mut file = std::fs::File::open("./examples/hft_hfqr.2024-11-03").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    let tick = parse_log(string);
    println!("{tick:?}");
}
