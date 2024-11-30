use hidapi::{HidDevice, HidError};

use crate::UsbInterface;

pub struct GpsdoHidApiInterface<'a> {
    driver: &'a HidDevice,
}

impl<'a> GpsdoHidApiInterface<'a> {
    pub fn new(driver: &'a HidDevice) -> Self {
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
