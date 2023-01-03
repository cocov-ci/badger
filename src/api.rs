use std::io;
use async_trait::async_trait;
use reqwest::header::AUTHORIZATION;
use crate::MyError;

#[async_trait]
pub(crate) trait Api: Sync + Send {
    async fn coverage_for(&self, repo: String) -> Result<String, MyError>;
    async fn issues_for(&self, repo: String) -> Result<String, MyError>;
}

pub(crate) struct RealApi {
    pub(crate) url: String,
    pub(crate) token: String,
}

impl RealApi {
    pub(crate) fn new(url: String, token: String) -> RealApi {
        let mut url = url;
        if url.ends_with('/') {
            url = url.trim_end_matches('/').to_string();
        }

        RealApi {
            token,
            url,
        }
    }
}

impl RealApi {
    async fn get(&self, repo: String, kind: &str) -> Result<String, MyError> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/v1/repositories/{}/badges/{}", self.url, repo, kind))
            .header(AUTHORIZATION, format!("bearer {}", self.token))
            .send()
            .await?;
        if resp.status() == 404 {
            return Err(MyError::NotFound);
        } else if !resp.status().is_success() {
            return Err(MyError::InternalError);
        }
        Ok(resp.text()
            .await?)
    }
}

#[async_trait]
impl Api for RealApi {
    async fn coverage_for(&self, repo: String) -> Result<String, MyError> {
        self.get(repo, "coverage").await
    }

    async fn issues_for(&self, repo: String) -> Result<String, MyError> {
        self.get(repo, "issues").await
    }
}

pub(crate) struct FakeApi();

impl FakeApi {
    fn common_behaviour(&self, repo: String) -> Result<String, MyError> {
        if repo == "u" {
            Ok("unknown".into())
        } else if repo == "404" {
            Err(MyError::NotFound)
        } else {
            Err(io::Error::from(io::ErrorKind::UnexpectedEof).into())
        }
    }
}

#[async_trait]
impl Api for FakeApi {
    async fn coverage_for(&self, repo: String) -> Result<String, MyError> {
        if repo == "90" {
            Ok("90".into())
        } else {
            self.common_behaviour(repo)
        }
    }

    async fn issues_for(&self, repo: String) -> Result<String, MyError> {
        if repo == "1" {
            Ok("1".into())
        } else {
            self.common_behaviour(repo)
        }
    }
}
