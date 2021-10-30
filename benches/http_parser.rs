use criterion::{criterion_group, criterion_main, Criterion};
use nom_locate::LocatedSpan;
use restman::http::parse_request;

fn request_parsing_benchmark(c: &mut Criterion) {
    let input = LocatedSpan::new(
        "GET /index.html HTTP/1.1\r\n\
        Content-type: application/json\r\n\
        Authorization: bearer token\r\n\
        \r\n\
        {\"foo\": \"bar\"}\r\n",
    );

    c.bench_function("parse http request", |b| {
        b.iter(|| {
            parse_request(input).unwrap();
        })
    });
}

criterion_group!(benches, request_parsing_benchmark);
criterion_main!(benches);
