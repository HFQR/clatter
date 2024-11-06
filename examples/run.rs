use std::io::Read;

use plotters::prelude::*;

const OUT_FILE_NAME: &str = "test.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open("./examples/hft_hfqr.2024-11-03").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();

    let tick = clatter::parse(string);

    let root = BitMapBackend::new(OUT_FILE_NAME, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;

    let (upper, lower) = root.split_vertically(1080 - 324);

    let (to_date, from_date) = (
        tick.last().unwrap().time.and_utc().timestamp(),
        tick.first().unwrap().time.and_utc().timestamp(),
    );

    let min = tick
        .iter()
        .min_by(|a, b| a.mid_price.total_cmp(&b.mid_price))
        .unwrap()
        .mid_price;

    let max = tick
        .iter()
        .max_by(|a, b| a.mid_price.total_cmp(&b.mid_price))
        .unwrap()
        .mid_price;

    let mut chart = ChartBuilder::on(&upper)
        .caption("Trading momentum", ("sans-serif", 30))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .build_cartesian_2d((from_date..to_date).into_segmented(), min..max)?;

    chart.configure_mesh().y_desc("Price").draw()?;

    let line = LineSeries::new(
        tick.iter().map(|tick| {
            (
                SegmentValue::CenterOf(tick.time.and_utc().timestamp()),
                tick.mid_price,
            )
        }),
        &BLACK,
    );

    chart.draw_series(line)?;

    let min = tick
        .iter()
        .min_by(|a, b| a.profit.total_cmp(&b.profit))
        .unwrap()
        .profit;

    let max = tick
        .iter()
        .max_by(|a, b| a.profit.total_cmp(&b.profit))
        .unwrap()
        .profit;

    let mut chart = ChartBuilder::on(&lower)
        .caption("Trading volume", ("sans-serif", 30))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .build_cartesian_2d((from_date..to_date).into_segmented(), min..max)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .y_desc("Profit")
        .draw()?;

    let actual = Histogram::vertical(&chart).style(GREEN.filled()).data(
        tick.iter()
            .map(|tick| (tick.time.and_utc().timestamp(), tick.profit)),
    );

    chart.draw_series(actual)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}
