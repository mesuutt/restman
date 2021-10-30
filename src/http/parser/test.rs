#[cfg(test)]
mod test {
    use crate::http::parser::{
        block_parser, header_line, request, request_line, Header, MessageBody, Method, Request,
        RequestLine, Version,
    };

    #[test]
    fn request_line_test() {
        let input = b"GET /index.html HTTP/1.1\r\n";
        let result = request_line(input);
        let expected = RequestLine {
            method: b"GET",
            path: b"/index.html",
            version: Version::V11,
        };

        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn request_line_without_http_version() {
        let input = b"GET /index.html\r\n";
        let (_, l) = request_line(input).unwrap();

        assert_eq!(l.version, Version::V11);
    }

    #[test]
    fn header_parser_test() {
        let input = b"Content-type: application/json\r\n";

        let result = header_line(input);

        let expected = Header {
            name: b"Content-type",
            value: b"application/json",
        };

        assert_eq!(result, Ok((&b""[..], expected)));
    }

    #[test]
    fn request_parser_test() {
        let input = b"GET /index.html HTTP/1.1\r\n\
        Content-type: application/json\r\n\
        Authorization: bearer token\r\n\
        \r\n\
        {\"foo\": \"bar\"}\r\n";

        let result = request(input).unwrap();

        let expected = Request {
            method: Method::Get,
            path: "/index.html".to_string(),
            version: Version::V11,
            headers: vec![
                Header {
                    name: b"Content-type",
                    value: b"application/json",
                },
                Header {
                    name: b"Authorization",
                    value: b"bearer token",
                },
            ],
            body: MessageBody::Bytes(b"{\"foo\": \"bar\"}"),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn request_body_parser_test() {
        let input = b"\r\nfoo=bar\r\n";
        let (_i, u) = block_parser(input, b"\r\n", b"\r\n").unwrap();

        assert_eq!(&b"foo=bar"[..], u);
    }

    #[test]
    fn request_without_body() {
        let input = b"GET /index.html HTTP/1.1\r\n\r\nfoo=bar\r\n";
        let result = request(input).unwrap();

        let expected = Request {
            method: Method::Get,
            path: "/index.html".to_string(),
            version: Version::V11,
            headers: vec![],
            body: MessageBody::Bytes(b"foo=bar"),
        };

        assert_eq!(result, expected);
    }
}
