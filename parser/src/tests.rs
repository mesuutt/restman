#[cfg(test)]
mod test {
    use indoc::{formatdoc, indoc};
    use nom_locate::LocatedSpan;

    use crate::parsers::*;

    use crate::ast::{Header, MessageBody, Method, ScriptHandler, Version};

    #[test]
    fn it_should_parse_request_line_with_version() {
        let input = LocatedSpan::new_extra("GET /index.html HTTP/1.1", "");
        let result = request_line(input);
        let expected = RequestLine {
            method: LocatedSpan::new_extra("GET", ""),
            target: LocatedSpan::new_extra("/index.html", ""),
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
        let input = LocatedSpan::new_extra("GET /index.html", "");
        let (_, line) = request_line(input).unwrap();

        assert_eq!(line.version, Version::V11);
    }

    #[test]
    fn it_should_parse_header() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "Content-type: application/json\n"
            },
            "",
        );

        let (_span, header) = parse_header(input).expect("header parse failed");

        let expected = Header {
            name: LocatedSpan::new_extra("Content-type", ""),
            value: LocatedSpan::new_extra("application/json", ""),
        };

        assert_eq!(header.name.fragment(), expected.name.fragment());
        assert_eq!(header.value.fragment(), expected.value.fragment());
    }

    #[test]
    fn it_should_not_parse_if_there_is_no_newline_at_end_of_the_header() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "Content-type: application/json"
            },
            "",
        );

        let result = parse_header(input);
        assert!(result.is_err());
    }

    #[test]
    fn it_should_parse_header_with_newline() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "Content-type: application/json\n\n"
            },
            "",
        );

        let (_span, header) = parse_header(input).expect("header parse failed");

        let expected = Header {
            name: LocatedSpan::new_extra("Content-type", ""),
            value: LocatedSpan::new_extra("application/json", ""),
        };

        assert_eq!(header.name.fragment(), expected.name.fragment());
        assert_eq!(header.value.fragment(), expected.value.fragment());
    }

    #[test]
    fn it_should_parse_multiple_headers() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "Content-type: application/json
            Authorization: bearer token
        "},
            "",
        );

        let (span, headers) = parse_headers(input).expect("header parse failed");

        let expected1 = Header {
            name: LocatedSpan::new_extra("Content-type", ""),
            value: LocatedSpan::new_extra("application/json", ""),
        };

        let expected2 = Header {
            name: LocatedSpan::new_extra("Authorization", ""),
            value: LocatedSpan::new_extra("bearer token", ""),
        };

        assert!(span.is_empty());
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0], expected1);
        assert_eq!(headers[1], expected2);
    }

    #[test]
    fn it_should_parse_a_full_featured_request() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "### My request
            GET /index.html HTTP/1.1
            Content-type: application/json
            Authorization: bearer token

            {\"foo\": \"bar\"}"
            },
            "",
        );

        let (_span, result) = parse_request(input).unwrap();

        assert_eq!(*result.title.unwrap().fragment(), "My request");
        assert_eq!(result.method, Method::Get);
        assert_eq!(result.target, "/index.html".to_string());
        assert_eq!(result.version, Version::V11);

        assert_eq!(result.headers.len(), 2);
        assert_eq!(
            result.body.get_span().unwrap().fragment(),
            &r#"{"foo": "bar"}"#
        );
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
        let input = LocatedSpan::new_extra(request_input.as_str(), "");

        let (span, result) = parse_request(input).unwrap();

        assert_eq!(span.fragment(), &"");
        assert_eq!(result.body.get_span().unwrap().fragment(), &body);
    }

    #[test]
    fn it_should_parse_request_without_body() {
        let input = LocatedSpan::new_extra("GET /index.html", "");
        let (_, result) = parse_request(input).unwrap();

        assert_eq!(result.body, MessageBody::Empty);
    }

    #[test]
    fn multiple_request_parser_test() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "### Request 1
            GET /first.html

            {foo: bar}

            > ./foo.js

            ### Request 2
            GET /last.html"
            },
            "",
        );
        let (_i, result) = parse_multiple_request(input).unwrap();

        assert_eq!(result.len(), 2);
        let first_req = result.get(0).unwrap();
        assert_eq!(*first_req.title.unwrap().fragment(), "Request 1");
        assert_eq!(first_req.headers.len(), 0);
        assert_eq!(
            first_req.script,
            ScriptHandler::File(Span::new_extra("./foo.js", ""))
        );
        if let MessageBody::Bytes(body) = first_req.body {
            assert_eq!(body.fragment(), &"{foo: bar}\n\n"); // TODO: parse body should return without \n\n
        } else {
            assert!(false, "body not matches")
        }

        let second_req = result.get(1).unwrap();
        assert_eq!(*second_req.title.unwrap().fragment(), "Request 2");
        assert_eq!(second_req.headers.len(), 0);
        assert_eq!(second_req.body, MessageBody::Empty);
        assert_eq!(second_req.script, ScriptHandler::Empty);
    }

    #[test]
    fn should_parse_body_until_next_request_title() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "### Request 1
            GET /first.html

            {foo: bar}

            ### Request 2
            GET /last.html"
            },
            "",
        );
        let (_i, result) = parse_multiple_request(input).unwrap();

        assert_eq!(result.len(), 2);
        if let MessageBody::Bytes(body) = result.get(0).unwrap().body {
            assert_eq!(body.fragment(), &"{foo: bar}\n\n");
        } else {
            assert!(false, "body not matches")
        }
    }

    #[test]
    fn it_should_parse_request_if_request_starts_or_ends_with_newlines() {
        let input = LocatedSpan::new_extra(
            indoc! {
                "

            ### Request 1
            GET /last.html

            "
            },
            "",
        );
        let (i, _result) = parse_request(input).unwrap();
        assert_eq!(i.fragment(), &"");
    }

    #[test]
    fn it_should_parse_if_input_file_ref_given_as_body() {
        let input = LocatedSpan::new_extra("< ./input.json", "");

        let (i, body) = parse_input_file_ref(input).unwrap();

        assert!(i.is_empty());
        assert_eq!(body.get_span().unwrap().fragment(), &"./input.json");
    }

    #[test]
    fn it_should_parse_request_body_with_multiple_lines() {
        let input = LocatedSpan::new_extra(
            indoc! {r#"{
                1: 1,

                "foo": "bar"

            }"#
            },
            "",
        );

        let result = parse_request_body(input);

        assert!(result.unwrap().0.is_empty());
    }

    #[test]
    fn it_should_parse_inline_script_handler() {
        let input = LocatedSpan::new_extra("> {% my script %}\n", "");
        let (i, h) = parse_inline_script(input).unwrap();

        assert!(i.is_empty());
        assert_eq!(
            h,
            ScriptHandler::Inline(LocatedSpan::new_extra(" my script ", ""))
        );
    }

    #[test]
    fn it_should_parse_external_script_handler() {
        let input = LocatedSpan::new_extra("> ./my-script.js", "");
        let (i, h) = parse_external_script(input).unwrap();

        assert!(i.is_empty());
        assert_eq!(
            h,
            ScriptHandler::File(LocatedSpan::new_extra("./my-script.js", ""))
        );
    }

    #[test]
    fn it_should_parse_script_handle() {
        let (_i1, s1) = parse_script(Span::new_extra("> ./foo.js\n", "")).unwrap();
        let (_i2, s2) = parse_script(Span::new_extra("> {% my inline script %}\n", "")).unwrap();

        assert_eq!(s1, ScriptHandler::File(Span::new_extra("./foo.js", "")));
        assert_eq!(
            s2,
            ScriptHandler::Inline(Span::new_extra(" my inline script ", ""))
        );
    }
}
