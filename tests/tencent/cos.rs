use std::env;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use http::header::AUTHORIZATION;
use http::Request;
use http::StatusCode;
use log::debug;
use log::warn;
use percent_encoding::utf8_percent_encode;
use percent_encoding::NON_ALPHANUMERIC;
use reqsign::TencentCosConfig;
use reqsign::TencentCosCredentialLoader;
use reqsign::TencentCosSigner;
use reqwest::Client;

fn init_signer() -> Option<(TencentCosCredentialLoader, TencentCosSigner)> {
    let _ = env_logger::builder().is_test(true).try_init();

    dotenv::from_filename(".env").ok();
    if env::var("REQSIGN_TENCENT_COS_TEST").is_err()
        || env::var("REQSIGN_TENCENT_COS_TEST").unwrap() != "on"
    {
        return None;
    }

    let config = TencentCosConfig {
        access_key_id: Some(
            env::var("REQSIGN_TENCENT_COS_ACCESS_KEY")
                .expect("env REQSIGN_TENCENT_COS_ACCESS_KEY must set"),
        ),
        secret_access_key: Some(
            env::var("REQSIGN_TENCENT_COS_SECRET_KEY")
                .expect("env REQSIGN_TENCENT_COS_SECRET_KEY must set"),
        ),
        ..Default::default()
    };
    let loader = TencentCosCredentialLoader::new(config);

    Some((loader, TencentCosSigner::new()))
}

#[tokio::test]
async fn test_get_object() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_TENCENT_COS_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();
    let cred = loader.load().await?.unwrap();

    let url = &env::var("REQSIGN_TENCENT_COS_URL").expect("env REQSIGN_TENCENT_COS_URL must set");

    let mut req = Request::new("");
    *req.method_mut() = http::Method::GET;
    *req.uri_mut() = http::Uri::from_str(&format!("{}/{}", url, "not_exist_file"))?;

    signer
        .sign(&mut req, &cred)
        .expect("sign request must success");

    debug!("signed request: {:?}", req.headers().get(AUTHORIZATION));

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must succeed");

    let status = resp.status();
    debug!("got response: {:?}", resp);
    debug!("got response content: {}", resp.text().await?);
    assert_eq!(StatusCode::NOT_FOUND, status);
    Ok(())
}

#[tokio::test]
async fn test_delete_objects() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_TENCENT_COS_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();
    let cred = loader.load().await?.unwrap();

    let url = &env::var("REQSIGN_TENCENT_COS_URL").expect("env REQSIGN_TENCENT_COS_URL must set");

    let mut req = Request::new(
        r#"<Delete>
<Object>
 <Key>sample1.txt</Key>
 </Object>
 <Object>
   <Key>sample2.txt</Key>
 </Object>
 </Delete>"#,
    );
    *req.method_mut() = http::Method::POST;
    *req.uri_mut() = http::Uri::from_str(&format!("{}/?delete", url))?;
    req.headers_mut()
        .insert("CONTENT-MD5", "WOctCY1SS662e7ziElh4cw==".parse().unwrap());

    signer
        .sign(&mut req, &cred)
        .expect("sign request must success");

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must succeed");

    let status = resp.status();
    debug!("got response: {:?}", resp);
    debug!("got response content: {}", resp.text().await?);
    assert_eq!(StatusCode::OK, status);
    Ok(())
}

#[tokio::test]
async fn test_get_object_with_query_sign() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_TENCENT_COS_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();
    let cred = loader.load().await?.unwrap();

    let url = &env::var("REQSIGN_TENCENT_COS_URL").expect("env REQSIGN_TENCENT_COS_URL must set");

    let mut req = Request::new("");
    *req.method_mut() = http::Method::GET;
    *req.uri_mut() = http::Uri::from_str(&format!("{}/{}", url, "not_exist_file"))?;

    signer
        .sign_query(&mut req, Duration::from_secs(3600), &cred)
        .expect("sign request must success");

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must succeed");

    let status = resp.status();
    debug!("got response: {:?}", resp);
    debug!("got response content: {}", resp.text().await?);
    assert_eq!(StatusCode::NOT_FOUND, status);
    Ok(())
}

#[tokio::test]
async fn test_head_object_with_special_characters() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_TENCENT_COS_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();
    let cred = loader.load().await?.unwrap();

    let url = &env::var("REQSIGN_TENCENT_COS_URL").expect("env REQSIGN_TENCENT_COS_URL must set");

    let mut req = Request::new("");
    *req.method_mut() = http::Method::HEAD;
    *req.uri_mut() = http::Uri::from_str(&format!(
        "{}/{}",
        url,
        utf8_percent_encode("not-exist-!@#$%^&*()_+-=;:'><,/?.txt", NON_ALPHANUMERIC)
    ))?;

    signer
        .sign(&mut req, &cred)
        .expect("sign request must success");

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    debug!("got response: {:?}", resp);
    assert_eq!(StatusCode::NOT_FOUND, resp.status());
    Ok(())
}

#[tokio::test]
async fn test_put_object_with_special_characters() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_TENCENT_COS_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();
    let cred = loader.load().await?.unwrap();

    let url = &env::var("REQSIGN_TENCENT_COS_URL").expect("env REQSIGN_TENCENT_COS_URL must set");

    let mut req = Request::new("");
    *req.method_mut() = http::Method::PUT;
    *req.uri_mut() = http::Uri::from_str(&format!(
        "{}/{}",
        url,
        utf8_percent_encode("put-!@#$%^&*()_+-=;:'><,/?.txt", NON_ALPHANUMERIC)
    ))?;

    signer
        .sign(&mut req, &cred)
        .expect("sign request must success");

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    let status = resp.status();
    debug!("got response: {:?}", resp);
    debug!("got response content: {:?}", resp.text().await?);
    assert_eq!(StatusCode::OK, status);
    Ok(())
}

#[tokio::test]
async fn test_list_bucket() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_TENCENT_COS_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();
    let cred = loader.load().await?.unwrap();

    let url = &env::var("REQSIGN_TENCENT_COS_URL").expect("env REQSIGN_TENCENT_COS_URL must set");

    let mut req = Request::new("");
    *req.method_mut() = http::Method::GET;
    *req.uri_mut() =
        http::Uri::from_str(&format!("{url}?list-type=2&delimiter=/&encoding-type=url"))?;

    signer
        .sign(&mut req, &cred)
        .expect("sign request must success");

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    let status = resp.status();
    debug!("got response: {:?}", resp);
    debug!("got response content: {}", resp.text().await?);
    assert_eq!(StatusCode::OK, status);
    Ok(())
}

#[tokio::test]
async fn test_list_bucket_with_invalid_token() -> Result<()> {
    let signer = init_signer();
    if signer.is_none() {
        warn!("REQSIGN_TENCENT_COS_TEST is not set, skipped");
        return Ok(());
    }
    let (loader, signer) = signer.unwrap();
    let cred = loader.load().await?.unwrap();

    let url = &env::var("REQSIGN_TENCENT_COS_URL").expect("env REQSIGN_TENCENT_COS_URL must set");

    let mut req = Request::new("");
    *req.method_mut() = http::Method::GET;
    *req.uri_mut() = http::Uri::from_str(&format!(
        "{}?list-type=2&delimiter=/&encoding-type=url&continuation-token={}",
        url,
        utf8_percent_encode("hello.txt", NON_ALPHANUMERIC)
    ))?;

    signer
        .sign(&mut req, &cred)
        .expect("sign request must success");

    debug!("signed request: {:?}", req);

    let client = Client::new();
    let resp = client
        .execute(req.try_into()?)
        .await
        .expect("request must success");

    let status = resp.status();
    debug!("got response: {:?}", resp);
    debug!("got response content: {}", resp.text().await?);
    assert_eq!(StatusCode::BAD_REQUEST, status);
    Ok(())
}
