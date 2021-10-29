#[cfg(test)]
mod test {
    use crate::http::parser::{request_line, RequestLine, Version};

    #[test]
    fn request_line_test() {
        let input = b"GET /index.html HTTP/1.1\r\n";
        let result = request_line(input);
        let expected = RequestLine {
            method: b"GET",
            path: b"/index.html",
            version: Version::V11
        };

        assert_eq!(result, Ok((&[][..], expected)));
    }
}