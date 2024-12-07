mod model;

use actix_multipart::form::MultipartForm;
use actix_web::{get, http::Method, middleware, post, web, HttpResponse, Responder};
use model::{JobOptionsCapabilities, Printer, PrinterJobForm};

use ipp::prelude::*;

use crate::util::AppConfig;
use crate::util::Message;

async fn get_printers(ip_host: &String) -> Result<Vec<Printer>, Message> {
    let uri: Uri = ip_host.parse::<Uri>().or_else(|e| {
        eprintln!("{}", e);
        Err(Message {
            message: format!("Failed parse printer URI: {}", ip_host).to_string(),
        })
    })?;

    let client = AsyncIppClient::new(uri);
    let operation = IppOperationBuilder::cups().get_printers();
    let resp = client.send(operation).await.or_else(|e| {
        eprintln!("Error: {}", e);
        Err(Message {
            message: "Failed to list printers".to_string(),
        })
    })?;
    if resp.header().status_code().is_success() {
        let mut vec = Vec::new();
        for group in resp.attributes().groups_of(DelimiterTag::PrinterAttributes) {
            // uncomment for printer attributes
            // print!("{:#?}", resp.attributes());
            let name = group.attributes()["printer-name"].value();
            let uri: &IppValue = group.attributes()["printer-uri-supported"].value();
            let two_sides = group.attributes()["sides-supported"]
                .value()
                .clone()
                .into_array();
            let state = group.attributes()["printer-state"]
                .value()
                .as_enum()
                .and_then(|v| PrinterState::from_i32(*v))
                .ok_or(Message {
                    message: format!(
                        "Failed read attribute 'printer-state' for printer '{}' - is it invalid?",
                        name
                    ),
                })?;

            vec.push(Printer {
                name: name.to_string(),
                uri: uri.to_string(),
                state: format!("{:?}", state),
                capabilites: JobOptionsCapabilities {
                    two_sided: match two_sides {
                        Ok(v) => v.len() > 0,
                        Err(_) => false,
                    },
                },
            });

            println!("{name}: {uri} {state:?}");
        }
        Ok(vec)
    } else {
        Err(Message {
            message: resp.header().status_code().to_string(),
        })
    }
}

fn get_printer_by_name(printers: Vec<Printer>, name: String) -> Result<Printer, Message> {
    let results: Vec<&Printer> = printers.iter().filter(|p| p.name == name).collect();
    if results.len() == 1 {
        Ok(results[0].clone())
    } else {
        Err(Message {
            message: "Failed to find printer".to_string(),
        })
    }
}

#[get("/")]
async fn find_all(context: web::Data<AppConfig>) -> impl Responder {
    match get_printers(&format!("ipp://{}/printers", context.ipp_uri).to_string()).await {
        Ok(printers) => HttpResponse::Ok().json(printers),
        Err(m) => HttpResponse::InternalServerError().json(m),
    }
}

#[post("/print/")]
async fn print_doc(
    context: web::Data<AppConfig>,
    MultipartForm(form): MultipartForm<PrinterJobForm>,
) -> impl Responder {
    println!(
        "Uploaded file {}, with size: {}",
        form.job_info.printer, form.file.size
    );

    let payload = IppPayload::new(form.file.file);

    let print_uri = get_printers(&format!("ipp://{}/printers", context.ipp_uri).to_string())
        .await
        .and_then(|p| get_printer_by_name(p, form.job_info.printer.clone()))
        .and_then(|p| {
            p.uri
                .parse::<Uri>()
                .or_else(|e| {
                    println!("Error: {}", e);
                    Err(Message {
                        message: format!("Failed to parse URI: {}", context.ipp_uri).to_string(),
                    })
                })
                .and_then(|uri| Ok((p, uri)))
        });

    match print_uri {
        Ok((printer, uri)) => {
            let mut builder = IppOperationBuilder::print_job(uri.clone(), payload)
                .user_name("printweb")
                .job_title(form.file.file_name.unwrap_or("unnamed-job.pdf".to_string()));

            if printer.capabilites.two_sided
                && form
                    .job_info
                    .options
                    .as_ref()
                    .and_then(|j| Some(j.two_sided))
                    .unwrap_or(false)
            {
                builder = builder.attribute(IppAttribute::new(
                    "sides-supported",
                    IppValue::Keyword("two-sided-long-edge".to_string()),
                ));
            }

            let op = builder.build();
            match AsyncIppClient::new(uri).send(op).await {
                Ok(response) => HttpResponse::Ok().json(Message {
                    message: response.header().status_code().to_string(),
                }),
                Err(response) => HttpResponse::InternalServerError().json(Message {
                    message: response.to_string(),
                }),
            }
        }

        Err(e) => HttpResponse::NotFound().json(e),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/printers")
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Always,
            ))
            .service(find_all)
            .service(print_doc)
            .default_service(web::route().method(Method::GET)),
    );
}
