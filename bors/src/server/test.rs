use super::Server;
use hyper::{Body, Method, Request, StatusCode, Uri, Version};

#[tokio::test]
async fn pull_request_event() {
    static PAYLOAD: &str = include_str!("../../test-input/pull-request-event-payload");
    let request = request_from_raw_http(PAYLOAD);

    let mut service = Server::new(vec![]);

    let resp = service.route_github(request).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    println!("{:?}", resp);
}

// Super quick and dirty parsing of raw http into a `Request<Body>` type.
// This assumes that the content is JSON
fn request_from_raw_http(raw: &'static str) -> Request<Body> {
    let (headers, payload) = raw.split_at(raw.find("{\n").unwrap());

    let mut headers = headers.lines();
    let (method, uri, version) = parse_method_line(headers.next().unwrap());
    let headers = headers.map(|line| {
        let (key, value) = line.split_at(line.find(":").unwrap());
        // remove the ':' from the value
        (key.trim(), value[1..].trim())
    });

    let mut request_builder = Request::builder().method(method).uri(uri).version(version);
    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }

    request_builder.body(Body::from(payload)).unwrap()
}

fn parse_method_line(line: &str) -> (Method, Uri, Version) {
    let mut iter = line.split_whitespace();

    let method = iter.next().unwrap().parse().unwrap();

    let uri = iter.next().unwrap().parse().unwrap();

    let version = match iter.next().unwrap() {
        "HTTP/1.1" => Version::HTTP_11,
        _ => panic!("unknown version"),
    };

    (method, uri, version)
}
