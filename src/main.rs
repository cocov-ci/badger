mod api;
mod helpers;
mod formatters;

use std::{env, io};
use actix_web::{get, web, App, HttpResponse, HttpServer, Result, error};
use actix_web::http::{header, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers, Logger};
use env_logger::Env;
use ab_glyph::{FontArc};
use derive_more::{Display, Error};

struct State {
    api: Box<dyn api::Api>,
    font: FontArc,
}

#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "not found")]
    NotFound,
}

impl error::ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .body(self.to_string())
    }
}

impl From<reqwest::Error> for MyError {
    fn from(_: reqwest::Error) -> Self {
        MyError::InternalError
    }
}

impl From<io::Error> for MyError {
    fn from(_: io::Error) -> Self {
        MyError::InternalError
    }
}

#[get("/{repository}/coverage")]
async fn coverage(data: web::Data<State>, path: web::Path<String>) -> Result<HttpResponse, MyError> {
    let repo = path.into_inner();
    let value = data.api.coverage_for(repo).await?;
    let (color, label) = formatters::format_coverage(value);

    Ok(helpers::response_for_svg(helpers::badge_for(&data.font, "coverage".into(), label, color)))
}

#[get("/{repository}/issues")]
async fn issues(data: web::Data<State>, path: web::Path<String>) -> Result<HttpResponse, MyError> {
    let repo = path.into_inner();
    let value = data.api.issues_for(repo).await?;
    let (color, label) = formatters::format_issues(value);

    Ok(helpers::response_for_svg(helpers::badge_for(&data.font, "issues".into(), label, color)))
}

#[get("/health")]
async fn health() -> Result<HttpResponse, MyError> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("Ok"))
}

fn add_error_header<B>(mut res: actix_web::dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );

    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = env::var("COCOV_BADGER_BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1".into());

    let port: u16 = env::var("COCOV_BADGER_BIND_PORT")
        .unwrap_or_else(|_| "4000".into())
        .parse()
        .unwrap_or(4000);

    let api_url: String = env::var("COCOV_BADGER_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".into());

    let api_token: String = env::var("COCOV_BADGER_API_SERVICE_TOKEN")
        .unwrap_or_else(|_| "".into());

    let font_arc = FontArc::try_from_slice(include_bytes!("../resources/DejaVuSans.ttf"))
        .expect("failed parsing bundled font data");

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let state = State {
        api: Box::new(api::RealApi::new(api_url, api_token)),
        font: font_arc,
    };

    let server_state = web::Data::new(state);

    HttpServer::new(move || {
        App::new()
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, add_error_header),
            )
            .wrap(Logger::default())
            .app_data(server_state.clone())
            .service(coverage)
            .service(issues)
            .service(health)
    })
        .bind((host, port))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use actix_web::web::Data;

    fn make_state() -> Data<State> {
        let font_arc = FontArc::try_from_slice(include_bytes!("../resources/DejaVuSans.ttf"))
            .expect("failed parsing bundled font data");

        let state = State {
            api: Box::new(api::FakeApi {}),
            font: font_arc,
        };

        Data::new(state)
    }

    #[actix_web::test]
    async fn test_coverage_get() {
        let app = test::init_service(App::new().app_data(make_state()).service(coverage)).await;
        let req = test::TestRequest::default()
            .uri("/u/coverage")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_coverage_not_found() {
        let app = test::init_service(App::new().app_data(make_state()).service(issues)).await;
        let req = test::TestRequest::default()
            .uri("/404/coverage")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }


    #[actix_web::test]
    async fn test_problem_get() {
        let app = test::init_service(App::new().app_data(make_state()).service(issues)).await;
        let req = test::TestRequest::default()
            .uri("/u/issues")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_problem_not_found() {
        let app = test::init_service(App::new().app_data(make_state()).service(issues)).await;
        let req = test::TestRequest::default()
            .uri("/404/issues")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }
}
