use std::{future::Future, pin::Pin, str::FromStr};

use http::{Request, Response};
use mars_config::{ServiceConfig, AVALANCHE_TOKEN};
use tower::{Layer, Service};

use http::{header::HeaderName, HeaderValue, Uri};
use mars_config::Action;
use std::error::Error;

use url::Url;

use crate::response_from_status_message;

lazy_static::lazy_static! {
    static ref HOP_HEADERS: Vec<HeaderName> = vec![
        HeaderName::from_str("Connection").unwrap(),
        HeaderName::from_str("Keep-Alive").unwrap(),
        HeaderName::from_str("Proxy-Authenticate").unwrap(),
        HeaderName::from_str("Proxy-Authorization").unwrap(),
        HeaderName::from_str("Te").unwrap(),
        HeaderName::from_str("Trailers").unwrap(),
        HeaderName::from_str("Transfer-Encoding").unwrap(),
        HeaderName::from_str("Upgrade").unwrap(),
        HeaderName::from_str("Host").unwrap(),
    ];
}

pub struct ProxyUrlPath(pub String);

#[derive(Clone)]
pub struct CommonUpdateQueryNHeaders<S> {
    service_config: ServiceConfig,
    inner: S,
}

impl<S> CommonUpdateQueryNHeaders<S> {
    pub(crate) fn get_updated_request<ReqBody>(
        &self,
        rest: &str,
        req: &mut Request<ReqBody>,
    ) -> Result<(), Box<dyn Error>> {
        let service_config = &self.service_config;
        let uri = get_updated_url(service_config, rest)?;
        *req.uri_mut() = Uri::from_str(uri.as_ref())?;
        update_headers(req.headers_mut(), service_config)?;
        Ok(())
    }
}

fn get_updated_url(service_config: &ServiceConfig, rest: &str) -> Result<Url, Box<dyn Error>> {
    let uri = Url::from_str(&service_config.url.clone())?;
    let mut rest_path = uri.path().to_string();
    if !rest_path.ends_with("/") {
        rest_path.push('/');
    }
    let (mut path, query) = if rest.contains('?') {
        let mut iter = rest.split('?');
        let path = iter.next().unwrap();
        let query = iter.next().unwrap();
        (path.to_string(), query)
    } else {
        (rest.to_string(), "")
    };
    if path.starts_with('/') {
        path.remove(0);
    }
    let mut url_query_pairs =
        Vec::from_iter(url::form_urlencoded::parse(query.as_bytes()).into_owned());
    url_query_pairs.append(&mut uri.query_pairs().into_owned().collect());
    let join = uri.join(&rest_path);
    let uri = join.and_then(|x| x.join(&path))?;
    for url_param in &service_config.query_params {
        match url_param.action {
            Action::Add => {
                url_query_pairs.push((url_param.key.clone(), url_param.value.clone()));
            }
            Action::Discard => url_query_pairs.retain_mut(|x| x.0 != url_param.key),
            Action::Pass => {}
        }
    }
    let uri: Url = Url::parse_with_params(uri.as_ref(), &url_query_pairs)?;
    Ok(uri)
}

fn update_headers(
    headers_mut: &mut http::HeaderMap,
    service_config: &ServiceConfig,
) -> Result<(), Box<dyn Error>> {
    headers_mut.remove(AVALANCHE_TOKEN);
    for header in &service_config.headers {
        match header.action {
            Action::Add => {
                let key = HeaderName::from_str(&header.key)?;
                headers_mut.append(key, HeaderValue::from_str(&header.value)?);
            }
            Action::Discard => {
                headers_mut.remove(HeaderName::from_str(&header.key)?);
            }
            Action::Pass => {}
        }
    }
    Ok(for header in HOP_HEADERS.iter() {
        headers_mut.remove(header);
    })
}

#[cfg(test)]
mod test {
    use mars_config::{GeneralParams, MarsAuth, ServiceConfig, UrlParam};
    use serde_json::json;

    use super::get_updated_url;

    fn url_join(url: &str, rest: &str) -> String {
        get_updated_url(
            &ServiceConfig {
                url: url.to_owned(),
                method: mars_config::Method::ANY,
                query_params: vec![
                    UrlParam {
                        key: "added".to_string(),
                        value: "value".to_string(),
                        action: mars_config::Action::Add,
                    },
                    UrlParam {
                        key: "rajesh".to_string(),
                        value: "".to_string(),
                        action: mars_config::Action::Discard,
                    },
                    UrlParam {
                        key: "haha".to_string(),
                        value: "".to_string(),
                        action: mars_config::Action::Discard,
                    },
                ],
                headers: vec![],
                auth: MarsAuth::new(json!({}), mars_config::AuthType::NoAuth),
                params: GeneralParams::new(json!({})),
            },
            rest,
        )
        .unwrap()
        .to_string()
    }
    #[test]
    fn test_updated_url() {
        assert_eq!(
            "https://httpbin.org/first/second/third/?ranga=ramu&added=value",
            url_join(
                "https://httpbin.org/first/second?rajesh=ram",
                "third/?ranga=ramu&haha=ere"
            )
        );
        assert_eq!(
            "https://httpbin.org/first/second/third/?ranga=ramu&added=value",
            url_join(
                "https://httpbin.org/first/second/?rajesh=ram",
                "third/?ranga=ramu&haha=ere"
            )
        );
        assert_eq!(
            "https://httpbin.org/first/second/third/?ranga=ramu&added=value",
            url_join(
                "https://httpbin.org/first/second/?rajesh=ram",
                "/third/?ranga=ramu&haha=ere"
            )
        );
        assert_eq!(
            "https://httpbin.org/first/second/third/?ranga=ramu&added=value",
            url_join(
                "https://httpbin.org/first/second/?rajesh=ram",
                "/third/?ranga=ramu&haha=ere"
            )
        );
    }
}

#[derive(Clone)]
pub struct CommonUpdateQueryNHeaderLayer {
    service_config: ServiceConfig,
}

impl CommonUpdateQueryNHeaderLayer {
    pub fn new(service_config: ServiceConfig) -> Self {
        Self { service_config }
    }
}

impl<S> Layer<S> for CommonUpdateQueryNHeaderLayer {
    type Service = CommonUpdateQueryNHeaders<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CommonUpdateQueryNHeaders {
            service_config: self.service_config.clone(),
            inner,
        }
    }
}

type ResBody = hyper::Body;
impl<ReqBody, S> Service<Request<ReqBody>> for CommonUpdateQueryNHeaders<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: 'static,
    <S as Service<Request<ReqBody>>>::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let url: Option<String> = req.extensions().get::<ProxyUrlPath>().map(|x| x.0.clone());
        match self.get_updated_request(&url.expect("impossible to fail"), &mut req) {
            Ok(_) => {
                let fut = self.inner.call(req);
                Box::pin(async move { fut.await })
            }
            Err(error) => {
                let message = format!("unable to transform request err: `{:?}`", error);
                Box::pin(async move {
                    Ok(response_from_status_message(500, message).expect("impossible to fail"))
                })
            }
        }
    }
}
