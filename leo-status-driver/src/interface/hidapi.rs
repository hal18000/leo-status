use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};

use crate::{
    consts::{PID_LEO_BODNAR_GPSDO, PID_LEO_BODNAR_MINI_GPSDO, VID_LEO_BONDAR},
    UsbInterface,
};

pub struct GpsdoHidApiInterface<'a> {
    driver: &'a HidDevice,
}

impl<'a> GpsdoHidApiInterface<'a> {
    pub fn new(driver: &'a HidDevice) -> Self {
        Self { driver }
    }

    pub fn is_supported_vid_pid(descriptor: &DeviceInfo) -> bool {
        let product_id = descriptor.product_id();

        descriptor.vendor_id() == VID_LEO_BONDAR
            && (product_id == PID_LEO_BODNAR_GPSDO || product_id == PID_LEO_BODNAR_MINI_GPSDO)
    }

    pub fn find_gpsdo<'b>(
        hid_api: &'b HidApi,
        serial_number: Option<String>,
    ) -> Option<&'b DeviceInfo> {
        match serial_number {
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
                .find(|&descriptor| Self::is_supported_vid_pid(descriptor)),
        }
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
