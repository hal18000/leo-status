mod dto;

use leo_status_driver::{interface::GpsdoHidApiInterface, GpsdoDevice};
use tiny_http::{Header, Response, Server};

use std::{
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use hidapi::HidApi;

use clap::Parser;

use crate::dto::{ConfigResponse, LockStatusResponse};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long, value_parser = humantime::parse_duration, help = "Interval to poll the GPSDO for status")]
    interval: Duration,

    #[arg(
        long,
        help = "Serial number of the Leo Bodnar GPSDO device to use, if not specified any Leo Bodnar GPSDO connected will be used"
    )]
    serial_number: Option<String>,

    #[arg(long, help = "Print status of GPSDO to the console in JSON format")]
    stdout: bool,

    #[arg(long, help = "HTTP host to listen on")]
    http_host: SocketAddr,
}

fn main() {
    let args = Args::parse();

    let hid_api = HidApi::new().expect("failed to create hidapi context");

    let device = GpsdoHidApiInterface::find_gpsdo(&hid_api, args.serial_number)
        .expect("could not find leo bodnar gpsdo");

    let conn = device
        .open_device(&hid_api)
        .expect("could not open leo bodnar gpsdo usb");

    let hid_interface = GpsdoHidApiInterface::new(&conn);

    let gpsdo = GpsdoDevice::new(&hid_interface);

    let serial_number = gpsdo.serial_number().expect("could not get serial number");

    let config = gpsdo.config().unwrap();
    eprintln!(
        "device configuration: {:?}, f3 {}, fout1 {}, fout2 {}",
        config,
        config.f3(),
        config.fout1(),
        config.fout2()
    );
    eprintln!(
        "Using device with serial number {}",
        serial_number.unwrap_or_else(|| "unknown".to_owned())
    );

    let config_mutex: Arc<RwLock<Option<ConfigResponse>>> = Arc::new(RwLock::new(Option::None));
    let status_mutex: Arc<RwLock<Option<LockStatusResponse>>> = Arc::new(RwLock::new(Option::None));

    let http_host = args.http_host;
    let http_config_mutex = config_mutex.clone();
    let http_status_mutex = status_mutex.clone();
    std::thread::spawn(move || {
        let header_json_content_type = Header::from_str("Content-Type: application/json").unwrap();
        let server = Server::http(http_host).unwrap();

        for request in server.incoming_requests() {
            let response: Response<_> = match request.url() {
                "/config" | "/config/" => {
                    match http_config_mutex
                        .read()
                        .expect("failed to get config mutex")
                        .as_ref()
                    {
                        Some(value) => Response::from_data(
                            serde_json::to_vec(value).expect("failed to serialize config"),
                        )
                        .with_header(header_json_content_type.clone()),

                        None => Response::from_string("Service Unavailable - data not ready yet")
                            .with_status_code(503),
                    }
                }
                "/status" | "/status/" => {
                    match http_status_mutex
                        .read()
                        .expect("failed to get status mutex")
                        .as_ref()
                    {
                        Some(value) => Response::from_data(
                            serde_json::to_vec(value).expect("failed to serialize status"),
                        )
                        .with_header(header_json_content_type.clone()),

                        None => Response::from_string("Service Unavailable - data not ready yet")
                            .with_status_code(503),
                    }
                }

                _ => Response::from_string("Not Found").with_status_code(404),
            };

            if let Err(error) = request.respond(response) {
                eprintln!("failed to respond to http request: {}", error);
            }
        }
    });

    loop {
        let config = gpsdo.config().expect("failed to get config from gpsdo");
        let status = gpsdo.status().expect("failed to get status from gpsdo");

        *config_mutex.write().unwrap() = Some(config.into());
        *status_mutex.write().unwrap() = Some(status.into());

        std::thread::sleep(args.interval);
    }
}
