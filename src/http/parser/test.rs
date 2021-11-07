#[cfg(test)]
mod test {
    use indoc::indoc;
    use nom_locate::LocatedSpan;

    use crate::http::parser::{
        header_line, parse_headers, parse_input_file_ref, parse_multiple_request, parse_request,
        parse_request_body, request_line, Header, MessageBody, Method, RequestLine, Version,
    };

    #[test]
    fn it_should_parse_request_line_with_version() {
        let input = LocatedSpan::new("GET /index.html HTTP/1.1");
        let result = request_line(input);
        let expected = RequestLine {
            method: LocatedSpan::new("GET"),
            path: LocatedSpan::new("/index.html"),
            version: Version::V11,
        };

        assert!(result.is_ok());
        let (_span, line) = result.unwrap();

        assert_eq!(line.method, expected.method);
        assert_eq!(line.version, expected.version);
        assert_eq!(line.path.fragment(), expected.path.fragment());
    }

    #[test]
    fn it_should_parse_request_line_even_if_version_not_given() {
        let input = LocatedSpan::new("GET /index.html");
        let (_, l) = request_line(input).unwrap();

        assert_eq!(l.version, Version::V11);
    }

    #[test]
    fn it_should_parse_header() {
        let input = LocatedSpan::new(indoc! {
            "Content-type: application/json
            "
        });

        let (_span, header) = header_line(input).expect("header parse failed");

        let expected = Header {
            name: LocatedSpan::new("Content-type"),
            value: LocatedSpan::new("application/json"),
        };

        assert_eq!(header.name.fragment(), expected.name.fragment());
        assert_eq!(header.value.fragment(), expected.value.fragment());
    }

    #[test]
    fn it_should_return_err_if_header_not_ends_with_new_line() {
        let input = LocatedSpan::new("Header: value");

        assert!(header_line(input).is_err())
    }

    #[test]
    fn it_should_parse_multiple_headers() {
        let input = LocatedSpan::new(indoc! {
            "Content-type: application/json
            Authorization: bearer token
        "});

        let (span, headers) = parse_headers(input).expect("header parse failed");

        let expected1 = Header {
            name: LocatedSpan::new("Content-type"),
            value: LocatedSpan::new("application/json"),
        };

        let expected2 = Header {
            name: LocatedSpan::new("Authorization"),
            value: LocatedSpan::new("bearer token"),
        };

        assert!(span.is_empty());
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0], expected1);
        assert_eq!(headers[1], expected2);
    }

    #[test]
    fn it_should_parse_request_with_title_and_body() {
        let input = LocatedSpan::new(indoc! {
            "###My request
            GET /index.html HTTP/1.1
            Content-type: application/json
            Authorization: bearer token

            {\"foo\": \"bar\"}"
        });

        let (_span, result) = parse_request(input).unwrap();

        assert_eq!(result.title, "My request");
        assert_eq!(result.method, Method::Get);
        assert_eq!(result.path, "/index.html".to_string());
        assert_eq!(result.version, Version::V11);

        assert_eq!(result.headers.len(), 2);

        if let MessageBody::Bytes(span) = result.body {
            assert_eq!(span.fragment(), &r#"{"foo": "bar"}"#);
        } else {
            assert!(false, "message body not matches")
        }
    }

    #[test]
    fn it_should_parse_request_without_body() {
        let input = LocatedSpan::new("GET /index.html");
        let (_, result) = parse_request(input).unwrap();

        assert_eq!(result.body, MessageBody::Empty);
    }

    // #[test]
    fn multiple_request_parser_test() {
        let input = LocatedSpan::new(indoc! {
            "### Request 1
            GET /first.html

            ###Request 2
            GET /last.html"
        });
        let (_i, result) = parse_multiple_request(input).unwrap();

        // Bu fonksiyon icerisindeki yorumu oku
        assert!(false);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn it_should_parse_if_input_file_ref_given_as_body() {
        let input = LocatedSpan::new("> input.json");

        let (i, s) = parse_input_file_ref(input).unwrap();

        assert!(i.is_empty());

        if let MessageBody::File(s) = s {
            assert_eq!(s.fragment(), &"input.json")
        } else {
            assert!(false, "input file ref cannot parsed");
        };
    }

    #[test]
    fn it_should_parse_request_body_with_multiple_lines() {
        let input = LocatedSpan::new(indoc! {r#"{
                1: 1,

                "foo": "bar"

            }"#
        });

        let (i, _) = parse_request_body(input).unwrap();

        assert!(i.is_empty());
    }
}
