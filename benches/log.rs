use criterion::{criterion_group, criterion_main, Criterion};
use ftlog::{
    appender::{FileAppender, Period},
    debug, info,
};

pub fn logger(c: &mut Criterion) {
    // let time_format = time::format_description::parse_owned::<1>(
    //     "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]",
    // )
    // .unwrap();
    let _gaurd = ftlog::builder()
        .max_log_level(ftlog::LevelFilter::Info)
        // .time_format(time_format)
        // .bounded(100_1000, false)
        .root(
            FileAppender::builder()
                .path("./ftlog.log")
                .rotate(Period::Day)
                .expire(time::Duration::days(7))
                .build(),
        )
        .try_init()
        .unwrap();

    c.bench_function("fib 20", |b| {
        b.iter(|| {
            // info!("test");
            // debug!("test");
        })
    });
}

criterion_group!(log, logger);
criterion_main!(log);
