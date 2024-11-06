#![allow(internal_features)]
#![feature(core_intrinsics)]

use std::intrinsics::{maxnumf64, minnumf64};
use std::io::{Read};
use std::ops::Range;
use chrono::{Local, TimeZone};
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
                    profit_summary = 0.0;
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

pub fn cal_distance(init: f64, max: f64, min: f64) -> Range<f64> {
    let d1 = (max - init).abs();
    let d2 = (min - init).abs();
    let x = maxnumf64(d1, d2);
    init - x..(init + x)
}

pub fn plot(data: Vec<(i64, f64, f64)>) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;
    let mut data_iter = data.iter();
    let (first_timestamp, first_price, first_profit) = data_iter.next().unwrap();
    let (_, data1, data2, (max_price, min_price, max_profit, min_profit)) = data_iter.
        fold((0.0, vec![(*first_timestamp, *first_price)], vec![(*first_timestamp, *first_profit)], (*first_price, *first_price, *first_profit, *first_profit)),
             |(mut pp, mut price, mut profit, (mut max_price, mut min_price, mut max_profit, mut min_profit)), (timestamp, mid_price, signal_profit)| {
                 pp += *signal_profit;
                 price.push((*timestamp, *mid_price));
                 profit.push((*timestamp, pp));
                 max_price = maxnumf64(max_price, *mid_price);
                 min_price = minnumf64(min_price, *mid_price);
                 max_profit = maxnumf64(max_profit, pp);
                 min_profit = minnumf64(min_profit, pp);
                 (pp, price, profit, (max_price, min_price, max_profit, min_profit))
             });
    const OUT_FILE_NAME: &str = "two_scale.png";
    let root = BitMapBackend::new(OUT_FILE_NAME, (1600, 900)).into_drawing_area();
    root.fill(&RGBColor(24, 27, 31).mix(0.8))?;
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(40)
        .margin(5)
        .caption("strategy pnl", ("sans-serif", 14.0).into_font())
        .build_cartesian_2d(
            data1[0].0..data1[data1.len() - 1].0,
            cal_distance(data1[0].1, max_price, min_price),
        )?
        .set_secondary_coord(
            data1[0].0..data1[data1.len() - 1].0,
            cal_distance(data2[0].1, max_profit, min_profit),
        );
    chart
        .configure_mesh()
        .x_max_light_lines(2)
        .y_max_light_lines(2)
        .y_labels(6)
        .x_labels(10)
        .y_label_style(("sans-serif", 15).into_font().color(&RGBColor(167, 168, 181)))
        .x_label_style(("sans-serif", 15).into_font().color(&RGBColor(167, 168, 181)))
        .x_label_formatter(&|x| {
            let dt = Local.timestamp_millis_opt(*x).unwrap();
            dt.format("%H:%M").to_string()
        }).light_line_style(RGBColor(41, 45, 48).mix(0.8)).bold_line_style(RGBColor(41, 45, 48).mix(0.8)).draw()?
    ;
    chart.configure_secondary_axes().label_style(("sans-serif", 15).into_font().color(&RGBColor(167, 168, 181)))
        .y_labels(5).draw()?;
    chart
        .draw_series(LineSeries::new(
            data1,
            &YELLOW.mix(0.9),
        ))?
        .label("mid_price")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], YELLOW.mix(0.9)));

    chart
        .draw_secondary_series(LineSeries::new(
            data2,
            &GREEN.mix(0.6),
        ))?
        .label("profit")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.mix(0.6)));

    chart
        .configure_series_labels()
        .border_style(&WHITE)
        .draw()?;
    root.present()?;

    Ok(())
}

fn main() {
    // let mut file = std::fs::File::open(r"E:\CryptoHFT\hf\logs\hft_hfqr.2024-11-06").unwrap();
    let mut file = std::fs::File::open(r"./examples/hft_hfqr.2024-11-03").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    let tick = parse_log(string);
    // let mut x = OpenOptions::new().create(true).write(true).open("cc.txt").unwrap();
    // let f = format!("{tick:?}");
    // x.write(f.as_bytes()).unwrap();
    // println!("{tick:?}");
    plot(tick).unwrap()
}
