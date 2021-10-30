#[cfg(test)]
mod test {
    use crate::http::parser::{
        header_line, request, request_line, Header, MessageBody, Method, Request, RequestLine,
        Version,
    };
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

        // assert_eq!(result, Ok((LocatedSpan::new(""), expected)));
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
    fn request_parser_test() {
        let input = LocatedSpan::new(
            "GET /index.html HTTP/1.1\r\n\
        Content-type: application/json\r\n\
        Authorization: bearer token\r\n\
        \r\n\
        {\"foo\": \"bar\"}\r\n",
        );

        let result = request(input).unwrap();

        let expected = Request {
            method: Method::Get,
            path: "/index.html".to_string(),
            version: Version::V11,
            headers: vec![
                Header {
                    name: LocatedSpan::new("Content-type"),
                    value: LocatedSpan::new("application/json"),
                },
                Header {
                    name: LocatedSpan::new("Authorization"),
                    value: LocatedSpan::new("bearer token"),
                },
            ],
            body: MessageBody::Bytes(LocatedSpan::new("{\"foo\": \"bar\"}")),
        };

        assert_eq!(result.method, expected.method);
        assert_eq!(result.path, expected.path);
        assert_eq!(result.version, expected.version);

        for (i, h) in result.headers.iter().enumerate() {
            assert_eq!(h, &expected.headers[i]);
        }

        if let MessageBody::Bytes(span) = result.body {
            assert_eq!(span.fragment(), &"{\"foo\": \"bar\"}");
        } else {
            assert!(false, "message body not matches")
        }
    }

    /*
    #[test]
    fn request_body_parser_test() {
        let input = b"\r\nfoo=bar\r\n";
        let (_i, u) = block_parser(input, b"\r\n", b"\r\n").unwrap();

        assert_eq!(&b"foo=bar"[..], u);
    }*/

    /*#[test]
    fn request_without_body() {
        let input = LocatedSpan::new("GET /index.html HTTP/1.1\r\n\r\n");
        let result = request(input).unwrap();

        let expected = Request {
            method: Method::Get,
            path: "/index.html".to_string(),
            version: Version::V11,
            headers: vec![],
            body: MessageBody::Empty,
        };

        assert_eq!(result.method, expected.method);
        assert_eq!(result.path, expected.path);
        assert_eq!(result.body, expected.body);
    }*/
}
