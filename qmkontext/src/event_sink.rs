use crate::{Error, Result};
use hidapi::{HidApi, HidDevice};

#[derive(Clone, Debug)]
pub struct SendData {
    pub command_id: u8,
    pub data: u8,
}

pub trait EventSink {
    fn send(&self, data: &SendData) -> Result<()>;
}

pub struct HidEventSink {
    hid_device: HidDevice,
}

impl HidEventSink {
    pub fn new(vid: u16, pid: u16, usage: u16, usage_page: u16) -> Result<Self> {
        let api = HidApi::new()?;
        let device = Self::get_device(&api, vid, pid, usage, usage_page)?;
        Ok(Self { hid_device: device })
    }

    fn get_device(
        api: &HidApi,
        vid: u16,
        pid: u16,
        usage: u16,
        usage_page: u16,
    ) -> Result<HidDevice> {
        for device in api.device_list() {
            if device.vendor_id() == vid
                && device.product_id() == pid
                && usage == device.usage()
                && device.usage_page() == usage_page
            {
                let open_device = device.open_device(api)?;
                return Ok(open_device);
            }
        }
        Err(Error::HidError(format!(
            "Cannot find device vid={vid} pid={pid} usage={usage} usage_page={usage_page}"
        )))
    }
}

impl EventSink for HidEventSink {
    fn send(&self, data: &SendData) -> Result<()> {
        let mut buff = [0u8; 33];
        buff[1] = data.command_id;
        buff[2] = data.data;

        debug!(
            "Sending command_id={} | data={}",
            data.command_id, data.data
        );
        self.hid_device.write(&buff)?;
        Ok(())
    }
}

pub struct CliSink;

impl EventSink for CliSink {
    fn send(&self, data: &SendData) -> Result<()> {
        info!("Data: {:?}", data);
        Ok(())
    }
}
