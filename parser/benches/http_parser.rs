use criterion::{criterion_group, criterion_main, Criterion};
use indoc::indoc;
use nom_locate::LocatedSpan;
use parser::parse_request;

fn request_parsing_benchmark(c: &mut Criterion) {
    let input = LocatedSpan::new_extra(
        indoc! {r#"

        GET /index.html HTTP/1.1
        Content-type: application/json
        Authorization: bearer token

        {\"foo\": \"bar\"}

        > ./myscript.js

        "#
        },
        "",
    );

    c.bench_function("parse http request", |b| {
        b.iter(|| {
            parse_request(input).unwrap();
        })
    });
}

criterion_group!(benches, request_parsing_benchmark);
criterion_main!(benches);
