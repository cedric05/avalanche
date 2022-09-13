#[cfg(test)]
mod test {
    use std::{error::Error, io::Read};

    use http::Request;
    use hyper::{client::HttpConnector, Body, Client};
    use hyper_tls::HttpsConnector;
    use native_tls::{Identity, TlsConnector};
    use tokio_native_tls::TlsConnector as TokioNativeTlsConnector;

    const BAD_SSL_PASSWORD: &str = "badssl.com";
    const CERTIFICATE_P12: &str = "./resources/badssl.com-client.p12";

    #[tokio::test]
    pub async fn test_certificate() -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::open(CERTIFICATE_P12)?;
        let mut certificate_der = vec![];
        file.read_to_end(&mut certificate_der)?;

        let identity = Identity::from_pkcs12(&certificate_der, BAD_SSL_PASSWORD)?;
        let native_tls_connector = TlsConnector::builder().identity(identity).build()?;
        let tokio_native_tls_connector = TokioNativeTlsConnector::from(native_tls_connector);
        let mut http_connector = HttpConnector::new();
        http_connector.enforce_http(false);
        let connector: HttpsConnector<HttpConnector> =
            HttpsConnector::from((http_connector, tokio_native_tls_connector));
        let client = Client::builder().build::<_, hyper::Body>(connector);
        let req = Request::builder()
            .uri("https://client.badssl.com/")
            .body(Body::empty())?;
        let response = client.request(req).await?;
        for header in response.headers() {
            println!("header (key: {:?}, value: {:?})", header.0, header.1);
        }
        assert_eq!(response.status(), 200);
        Ok(())
    }
}
