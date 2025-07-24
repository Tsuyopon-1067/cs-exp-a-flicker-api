use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use local_ip_address::local_ip;
use url::Url;

type TagPhotoDataGzipMap = HashMap<String, Vec<u8>>;

fn load_tag_photodata_map() -> Result<TagPhotoDataGzipMap, Box<dyn std::error::Error>> {
    let mut file = File::open("./tag_photodata_map.bin")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let decoded: TagPhotoDataGzipMap = bincode::deserialize(&buffer)?;
    Ok(decoded)
}

async fn handle_request(
    req: Request<Body>,
    map: Arc<TagPhotoDataGzipMap>,
) -> Result<Response<Body>, Infallible> {
    let base_url = "http://localhost/"; // dummy base url
    let url = Url::parse(&base_url)
        .unwrap()
        .join(req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""))
        .unwrap();
    let tag = url.query_pairs().find(|(key, _)| key == "tag");

    if let Some((_, tag_name)) = tag {
        if let Some(gzipped_data) = map.get(tag_name.as_ref()) {
            // 既に圧縮済みのデータをそのまま返す
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("Content-Encoding", "gzip")
                .body(Body::from(gzipped_data.clone()))
                .unwrap();
            Ok(response)
        } else {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Tag not found"))
                .unwrap();
            Ok(response)
        }
    } else {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Missing 'tag' query parameter"))
            .unwrap();
        Ok(response)
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let port = 8080;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Loading tag_photodata_map.bin...");
    let map = Arc::new(load_tag_photodata_map()?);
    println!("tag_photodata_map.bin loaded.");

    let make_svc = make_service_fn(move |_conn| {
        let map = map.clone();
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, map.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    let my_local_ip = local_ip()?;
    println!("Listening on http://{}:{}", my_local_ip, port);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
