use ahash::AHashMap;
use std::convert::Infallible;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hyper::service::service_fn;
use hyper::{Body, Request, Response, StatusCode};
use local_ip_address::local_ip;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use url::Url;

type TagPhotoDataGzipMap = AHashMap<String, Vec<u8>>;

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

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let port = 8080;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Loading tag_photodata_map.bin...");
    let map = Arc::new(load_tag_photodata_map()?);
    println!("tag_photodata_map.bin loaded.");

    let certs = load_certs(&PathBuf::from("cert.pem"))?;
    let mut keys = load_keys(&PathBuf::from("key.pem"))?;

    let mut config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&addr).await?;

    let my_local_ip = local_ip()?;
    println!("Listening on https://{}:{}", my_local_ip, port);

    loop {
        let (stream, _peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let map = map.clone();

        let service = service_fn(move |req| handle_request(req, map.clone()));

        tokio::spawn(async move {
            let stream = match acceptor.accept(stream).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Tls error: {:?}", err);
                    return;
                }
            };
            if let Err(_) = hyper::server::conn::Http::new()
                .http2_only(true)
                .serve_connection(stream, service)
                .await
            {
                // eprintln!("Application error: {:?}", err);
            }
        });
    }
}
