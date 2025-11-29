use crate::DEBUG;
use actix_files::file_extension_to_mime;
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, Responder};
use include_dir::{include_dir, Dir};
use vite_actix::vite_app_factory::ViteAppFactory;

// Static directory including all files under `target/wwwroot`.
//
// This static directory is used to embed files into the binary at compile time.
// The `WWWROOT` directory will be used to serve static files such as `index.html`.
static WWWROOT: Dir = include_dir!("target/wwwroot");


// Handles the request for the index.html file.
//
// This function serves the `index.html` file from the embedded directory
// if it exists, and returns an internal server error if the file is not found.
//
// # Arguments
//
// * `_req` - The HTTP request object.
//
// # Returns
//
// An `impl Responder` which can either be a successful HTTP response containing
// the `index.html` file, or an internal server error.
pub async fn index(_req: HttpRequest) -> anyhow::Result<impl Responder, Error> {
	if let Some(file) = WWWROOT.get_file("index.html") {
		let body = file.contents();
		return Ok(HttpResponse::Ok().content_type("text/html").body(body));
	}
	Err(ErrorInternalServerError("Failed to find index.html"))
}

#[get("")]
async fn assets(file: web::Path<String>) -> impl Responder {
	if let Some(file) = WWWROOT.get_file(format!("assets/{}", file.as_str())) {
		let body = file.contents();
		return Ok(HttpResponse::Ok()
			.content_type(file_extension_to_mime(
				file.path().extension().unwrap().to_str().unwrap(),
			))
			.body(body));
	}
	Err(ErrorInternalServerError(format!("Failed to find {}", file)))
}

pub trait AssetsAppConfig {
	fn configure_frontend_routes(self) -> Self;
}

impl<T> AssetsAppConfig for App<T>
where
	T: actix_web::dev::ServiceFactory<
		actix_web::dev::ServiceRequest,
		Config = (),
		Error = Error,
		InitError = (),
	>,
{
	fn configure_frontend_routes(self) -> Self {
		if !DEBUG {
			self.default_service(web::route().to(index))
			    .service(web::scope("/assets/{file:.*}").service(assets))
		} else {
			self.configure_vite()
		}
	}
}
