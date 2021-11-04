#[cfg(test)]
mod test {
    use crate::http::parser::{header_line, parse_request, request_line, Header, MessageBody, Method, Request, RequestLine, Version, parse_headers, parse_multiple_request};
    use nom_locate::LocatedSpan;

    #[test]
    fn request_line_test() {
        let input = LocatedSpan::new("GET /index.html HTTP/1.1\r\n");
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
    fn request_line_without_http_version() {
        let input = LocatedSpan::new("GET /index.html\r\n");
        let (_, l) = request_line(input).unwrap();

        assert_eq!(l.version, Version::V11);
    }

    #[test]
    fn header_parser_test() {
        let input = LocatedSpan::new("Content-type: application/json\r\n");

        let (_span, header) = header_line(input).expect("header parse failed");

        let expected = Header {
            name: LocatedSpan::new("Content-type"),
            value: LocatedSpan::new("application/json"),
        };

        assert_eq!(header.name.fragment(), expected.name.fragment());
        assert_eq!(header.value.fragment(), expected.value.fragment());
    }

    #[test]
    fn multiple_header_parse_test() {
        let input = LocatedSpan::new("Content-type: application/json\r\nAuthorization: bearer token\r\n");

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
    fn request_parser_test() {
        let input = LocatedSpan::new(
            "###My request\r\nGET /index.html HTTP/1.1\r\n\
        Content-type: application/json\r\n\
        Authorization: bearer token\r\n\
        \r\n\
        {\"foo\": \"bar\"}\r\n",
        );

        let (span, result) = parse_request(input).unwrap();

        assert_eq!(result.title, "My request");
        assert_eq!(result.method, Method::Get);
        assert_eq!(result.path, "/index.html".to_string());
        assert_eq!(result.version,Version::V11);

        assert_eq!(result.headers.len(), 2);

        if let MessageBody::Bytes(span) = result.body {
            assert_eq!(span.fragment(), &"{\"foo\": \"bar\"}");
        } else {
            assert!(false, "message body not matches")
        }
    }

    #[test]
    fn request_without_body_test() {
        let input = LocatedSpan::new("GET /index.html\r\n\r\n");
        let (_, result) = parse_request(input).unwrap();

        assert_eq!(result.body, MessageBody::Empty);
    }


    #[test]
    fn multiple_request_parser_test() {
        let input = LocatedSpan::new("### Request 1\r\n\
        GET /first.html\r\n\r\n\
        ###Request 2\r\n\
        GET /last.html\r\n\r\n");
        let (_i, result) = parse_multiple_request(input).unwrap();

        assert_eq!(result.len(), 2);
    }
}
