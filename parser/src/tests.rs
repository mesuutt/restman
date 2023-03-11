#[cfg(test)]
mod test {
    use indoc::{formatdoc, indoc};
    use nom_locate::LocatedSpan;

    use crate::parsers::*;
    
    use crate::ast::{Header, MessageBody, Method, ScriptHandler, Version};

    #[test]
    fn it_should_parse_request_line_with_version() {
        let input = LocatedSpan::new("GET /index.html HTTP/1.1");
        let result = request_line(input);
        let expected = RequestLine {
            method: LocatedSpan::new("GET"),
            target: LocatedSpan::new("/index.html"),
            version: Version::V11,
        };

        assert!(result.is_ok());
        let (_span, line) = result.unwrap();

        assert_eq!(line.method, expected.method);
        assert_eq!(line.version, expected.version);
        assert_eq!(line.target.fragment(), expected.target.fragment());
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

        let (_span, header) = parse_header(input).expect("header parse failed");

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

        assert!(parse_header(input).is_err())
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
    fn it_should_parse_a_full_featured_request() {
        let input = LocatedSpan::new(indoc! {
            "### My request
            GET /index.html HTTP/1.1
            Content-type: application/json
            Authorization: bearer token

            {\"foo\": \"bar\"}"
        });

        let (_span, result) = parse_request(input).unwrap();

        assert_eq!(*result.title.unwrap().fragment(), "My request");
        assert_eq!(result.method, Method::Get);
        assert_eq!(result.target, "/index.html".to_string());
        assert_eq!(result.version, Version::V11);

        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.body.get_span().unwrap().fragment(), &r#"{"foo": "bar"}"#);
    }

    #[test]
    fn parse_multiline_body() {
        let body = "{\n\n  \"foo\": \"bar\"\n\n}";

        let request_input = formatdoc! {
            "
            GET /index.html HTTP/1.1

            {body}",
            body = body
        };
        let input = LocatedSpan::new(request_input.as_str());

        let (span, result) = parse_request(input).unwrap();

        assert_eq!(span.fragment(), &"");
        assert_eq!(result.body.get_span().unwrap().fragment(), &body);
    }

    #[test]
    fn it_should_parse_request_without_body() {
        let input = LocatedSpan::new("GET /index.html");
        let (_, result) = parse_request(input).unwrap();

        assert_eq!(result.body, MessageBody::Empty);
    }

    #[test]
    fn multiple_request_parser_test() {
        let input = LocatedSpan::new(indoc! {
            "### Request 1
            GET /first.html

            {foo: bar}

            > ./foo.js

            ### Request 2
            GET /last.html"
        });
        let (_i, result) = parse_multiple_request(input).unwrap();

        assert_eq!(result.len(), 2);
        let first_req = result.get(0).unwrap();
        assert_eq!(*first_req.title.unwrap().fragment(), "Request 1");
        assert_eq!(first_req.headers.len(), 0);
        assert_eq!(first_req.script, ScriptHandler::File(Span::new("./foo.js")));
    }


    #[test]
    fn it_should_parse_if_input_file_ref_given_as_body() {
        let input = LocatedSpan::new("< ./input.json");

        let (i, body) = parse_input_file_ref(input).unwrap();

        assert!(i.is_empty());
        assert_eq!(body.get_span().unwrap().fragment(), &"./input.json");
    }

    #[test]
    fn it_should_parse_request_body_with_multiple_lines() {
        let input = LocatedSpan::new(indoc! {r#"{
                1: 1,

                "foo": "bar"

            }"#
        });

        let result = parse_request_body(input);

        assert!(result.unwrap().0.is_empty());
    }

    #[test]
    fn it_should_parse_inline_script_handler() {
        let input = LocatedSpan::new("> {% my script %}\n");
        let (i, h) = parse_inline_script(input).unwrap();

        assert!(i.is_empty());
        assert_eq!(h, ScriptHandler::Inline(LocatedSpan::new(" my script ")));
    }

    #[test]
    fn it_should_parse_external_script_handler() {
        let input = LocatedSpan::new("> ./my-script.js\n");
        let (i, h) = parse_external_script(input).unwrap();

        assert!(i.is_empty());
        assert_eq!(h, ScriptHandler::File(LocatedSpan::new("./my-script.js")));
    }

    #[test]
    fn  it_should_parse_script_handle() {
        let (i1, _) = parse_script(LocatedSpan::new("> ./foo.js\n")).unwrap();
        let (i2, _) = parse_script(LocatedSpan::new("> {% my inline script %}\n")).unwrap();

        assert!(i1.is_empty());
        assert!(i2.is_empty());
    }
}
