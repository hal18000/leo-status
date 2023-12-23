mod gpsdo;

use std::time::Duration;

use gpsdo::UsbInterface;
use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};

use crate::gpsdo::GpsdoDevice;

use clap::Parser;

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
}

const VID_LEO_BONDAR: u16 = 0x1dd2;
const PID_LEO_BODNAR_GPSDO: u16 = 0x2210;
const PID_LEO_BODNAR_MINI_GPSDO: u16 = 0x2211;

fn is_target_device(descriptor: &DeviceInfo) -> bool {
    let product_id = descriptor.product_id();

    descriptor.vendor_id() == VID_LEO_BONDAR
        && (product_id == PID_LEO_BODNAR_GPSDO || product_id == PID_LEO_BODNAR_MINI_GPSDO)
}

struct GpsdoHidApiInterface<'a> {
    driver: &'a HidDevice,
}

impl<'a> GpsdoHidApiInterface<'a> {
    fn new(driver: &'a HidDevice) -> Self {
        Self { driver }
    }
}

impl<'a> UsbInterface for GpsdoHidApiInterface<'a> {
    type InterfaceError = HidError;

    fn hid_read(&self, buf: &mut [u8]) -> Result<usize, Self::InterfaceError> {
        self.driver.read(buf)
    }

    fn serial_number(&self) -> Result<Option<String>, Self::InterfaceError> {
        self.driver.get_serial_number_string()
    }

    fn hid_get_feature_report(
        &self,
        report_id: u8,
        buf: &mut [u8],
    ) -> Result<usize, Self::InterfaceError> {
        assert!(!buf.is_empty());
        buf[0] = report_id;

        let size = self.driver.get_feature_report(buf)?;

        // Workaround - Windows hidapi returns the report id in the first byte
        // of the result, so we correct this by moving everything backwards
        #[cfg(target_os = "windows")]
        {
            assert_eq!(buf[0], report_id);
            buf.copy_within(1..size + 1, 0)
        }

        Ok(size)
    }
}

fn main() {
    let args = Args::parse();

    let hid_api = HidApi::new().expect("failed to create hidapi context");

    let device = match args.serial_number {
        // Look for a device that matches the serial number and is from Leo Bodnar
        Some(serial_number) => hid_api.device_list().find(|&descriptor| {
            descriptor.vendor_id() == VID_LEO_BONDAR
                && descriptor
                    .serial_number()
                    .is_some_and(|device_serial| device_serial == serial_number)
        }),

        // Look for any device that is a GPSDO
        None => hid_api
            .device_list()
            .find(|&descriptor| is_target_device(descriptor)),
    }
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

    loop {
        let config = gpsdo.config().expect("failed to get config from gpsdo");
        let status = gpsdo.status().expect("failed to get status from gpsdo");

        println!("config: {:?}, status: {:?}", config, status);

        std::thread::sleep(args.interval);
    }
}
