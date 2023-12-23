use thiserror::Error;

#[derive(Debug, Error)]
pub enum GpsdoError<InterfaceError> {
    #[error("underlying usb interface errored: {0}")]
    UsbInterfaceError(#[from] InterfaceError),

    #[error("received less data than expected from device, expected {expected:?}, received {received:?}")]
    ShortDataError { expected: usize, received: usize },
}

/// The UsbInterface trait allows for use of different USB backends, such as hidapi.
pub trait UsbInterface {
    type InterfaceError;

    /// Read a set of bytes from the device, storing them in the passed buffer. The number of stored bytes should be returned
    fn hid_read(&self, buf: &mut [u8]) -> Result<usize, Self::InterfaceError>;

    /// Get a feature report from the device. The result should be stored in buf, with the zeroth byte being the first data byte.
    /// The report byte should not be included. The caller will provide a buffer which is n+1 in size, where n is the data size.
    fn hid_get_feature_report(
        &self,
        report_id: u8,
        buf: &mut [u8],
    ) -> Result<usize, Self::InterfaceError>;

    /// Get the serial number of the device. If no serial number exists on the device, then `Option::None`
    fn serial_number(&self) -> Result<Option<String>, Self::InterfaceError>;
}

pub struct GpsdoDevice<'a, Interface: UsbInterface> {
    interface: &'a Interface,
}

impl<'a, Interface: UsbInterface> GpsdoDevice<'a, Interface> {
    pub fn new(interface: &'a Interface) -> Self {
        GpsdoDevice { interface }
    }

    pub fn serial_number(&self) -> Result<Option<String>, GpsdoError<Interface::InterfaceError>> {
        Ok(self.interface.serial_number()?)
    }

    pub fn config(&self) -> Result<GpsdoConfig, GpsdoError<Interface::InterfaceError>> {
        let mut buf = [0u8; 61];

        let size = self.interface.hid_get_feature_report(9, &mut buf)?;
        if size < 21 {
            return Err(GpsdoError::ShortDataError {
                expected: 21,
                received: size,
            });
        }

        let output1 = buf[0] & 0x01 != 0;
        let output2 = buf[0] & 0x02 != 0;
        let level = buf[1];
        let fin = u32::from_le_bytes(buf[2..6].try_into().unwrap()) & 0x00FFFFFF;
        let n3 = (u32::from_le_bytes(buf[5..9].try_into().unwrap()) & 0x00FFFFFF) + 1;
        let n2_hs = buf[8] + 4;
        let n2_ls = (u32::from_le_bytes(buf[9..13].try_into().unwrap()) & 0x00FFFFFF) + 1;
        let n1_hs = buf[12] + 4;
        let nc1_ls = (u32::from_le_bytes(buf[13..17].try_into().unwrap()) & 0x00FFFFFF) + 1;
        let nc2_ls = (u32::from_le_bytes(buf[16..20].try_into().unwrap()) & 0x00FFFFFF) + 1;
        let skew = buf[19];
        let bw = buf[20];

        Ok(GpsdoConfig {
            output1,
            output2,
            level,
            fin,
            n3,
            n2_hs,
            n2_ls,
            n1_hs,
            nc1_ls,
            nc2_ls,
            skew,
            bw,
        })
    }

    pub fn status(&self) -> Result<GpsdoStatus, GpsdoError<Interface::InterfaceError>> {
        let mut buf = [0u8; 2];
        let read_count = self.interface.hid_read(&mut buf)?;

        let read_bytes = &buf[..read_count];

        if read_count < 2 {
            return Err(GpsdoError::ShortDataError {
                expected: 2,
                received: read_count,
            });
        }

        let loss_count = read_bytes[0];
        let sat_lock = read_bytes[1] & 0x01 == 0;
        let pll_lock = read_bytes[1] & 0x02 == 0;
        let locked = read_bytes[1] & 0x03 == 0;

        Ok(GpsdoStatus {
            loss_count,
            sat_lock,
            pll_lock,
            locked,
        })
    }
}

#[derive(Debug)]
pub struct GpsdoConfig {
    output1: bool,
    output2: bool,
    level: u8,
    fin: u32,
    n3: u32,
    n2_hs: u8,
    n2_ls: u32,
    n1_hs: u8,
    nc1_ls: u32,
    nc2_ls: u32,
    skew: u8,
    bw: u8,
}

impl GpsdoConfig {
    pub fn output1(&self) -> bool {
        self.output1
    }

    pub fn output2(&self) -> bool {
        self.output2
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn fin(&self) -> u32 {
        self.fin
    }

    pub fn n3(&self) -> u32 {
        self.n3
    }

    pub fn n2_hs(&self) -> u8 {
        self.n2_hs
    }

    pub fn n2_ls(&self) -> u32 {
        self.n2_ls
    }

    pub fn n1_hs(&self) -> u8 {
        self.n1_hs
    }

    pub fn nc1_ls(&self) -> u32 {
        self.nc1_ls
    }

    pub fn nc2_ls(&self) -> u32 {
        self.nc2_ls
    }

    pub fn skew(&self) -> u8 {
        self.skew
    }

    pub fn bw(&self) -> u8 {
        self.bw
    }

    pub fn f3(&self) -> u32 {
        self.fin / self.n3
    }

    pub fn fosc(&self) -> u64 {
        self.fin as u64 * (self.n2_hs as u64 * self.n2_ls as u64) / self.n3 as u64
    }

    pub fn fout1(&self) -> u64 {
        self.fosc() / (self.n1_hs as u64 * self.nc1_ls as u64)
    }

    pub fn fout2(&self) -> u64 {
        self.fosc() / (self.n1_hs as u64 * self.nc2_ls as u64)
    }
}

#[derive(Debug)]
pub struct GpsdoStatus {
    loss_count: u8,
    sat_lock: bool,
    pll_lock: bool,
    locked: bool,
}

impl GpsdoStatus {
    pub fn loss_count(&self) -> u8 {
        self.loss_count
    }

    pub fn sat_locked(&self) -> bool {
        self.sat_lock
    }

    pub fn pll_locked(&self) -> bool {
        self.pll_lock
    }

    pub fn locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
mod test {
    use core::panic;

    use super::{GpsdoDevice, UsbInterface};

    struct TestUsbInterface<'a>(&'a [u8], &'a [u8]);

    impl<'a> UsbInterface for TestUsbInterface<'a> {
        type InterfaceError = std::io::Error;

        fn hid_read(&self, buf: &mut [u8]) -> Result<usize, Self::InterfaceError> {
            buf.copy_from_slice(&self.0);

            Ok(self.0.len())
        }

        fn hid_get_feature_report(
            &self,
            _report_id: u8,
            buf: &mut [u8],
        ) -> Result<usize, Self::InterfaceError> {
            buf.copy_from_slice(&self.1);

            Ok(self.1.len())
        }

        fn serial_number(&self) -> Result<Option<String>, Self::InterfaceError> {
            Ok(Some("AAAA-BBBB".to_string()))
        }
    }

    struct TestUsbErrorInterface;

    impl UsbInterface for TestUsbErrorInterface {
        type InterfaceError = std::io::Error;

        fn hid_read(&self, _buf: &mut [u8]) -> Result<usize, Self::InterfaceError> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "error reading data",
            ))
        }

        fn serial_number(&self) -> Result<Option<String>, Self::InterfaceError> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "error reading serial no",
            ))
        }

        fn hid_get_feature_report(
            &self,
            _report_id: u8,
            _buf: &mut [u8],
        ) -> Result<usize, Self::InterfaceError> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "error getting feature report",
            ))
        }
    }

    #[test]
    fn gpsdo_device_read_returns_correct_data_pll_locked_sat_locked() {
        let test_interface = TestUsbInterface(&[23, 0b000], &[]);

        let device = GpsdoDevice::new(&test_interface);

        let status = device.status().expect("expected success from status");

        assert_eq!(status.loss_count(), 23);
        assert!(status.pll_locked());
        assert!(status.sat_locked());
        assert!(status.locked());
    }

    #[test]
    fn gpsdo_device_read_returns_correct_data_pll_unlocked_sat_unlocked() {
        let test_interface = TestUsbInterface(&[18, 0b111], &[]);

        let device = GpsdoDevice::new(&test_interface);

        let status = device.status().expect("expected success from status");

        assert_eq!(status.loss_count(), 18);
        assert!(!status.pll_locked());
        assert!(!status.sat_locked());
        assert!(!status.locked());
    }

    #[test]
    fn gpsdo_device_serial_number_returns_serial_number_when_serial_number_is_returned_from_interface(
    ) {
        let test_interface = TestUsbInterface(&[], &[]);

        let device = GpsdoDevice::new(&test_interface);

        let result = device.serial_number();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("AAAA-BBBB".into()));
    }

    #[test]
    fn gpsdo_device_read_returns_error_when_interface_returns_error() {
        let test_interface = TestUsbErrorInterface {};

        let device = GpsdoDevice::new(&test_interface);

        let result = device.status();

        match result {
            Ok(_) => panic!("expected error"),
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    "underlying usb interface errored: error reading data"
                );
            }
        }
    }

    #[test]
    fn gpsdo_device_serial_number_returns_error_when_interface_returns_error() {
        let test_interface = TestUsbErrorInterface {};

        let device = GpsdoDevice::new(&test_interface);

        let result = device.serial_number();

        assert!(result.is_err());

        match result {
            Ok(_) => panic!("expected error"),
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    "underlying usb interface errored: error reading serial no"
                );
            }
        }
    }
}
