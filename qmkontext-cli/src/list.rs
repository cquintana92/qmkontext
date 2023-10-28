use qmkontext::hidapi;

pub fn list_hid_devices() -> Result<(), hidapi::HidError> {
    let api = hidapi::HidApi::new()?;
    for device in api.device_list() {
        let product = device.product_string().unwrap_or_default();
        println!(
            "{}: vendor_id: {} | product_id: {} | usage: {} | usage_page: {}",
            product,
            device.vendor_id(),
            device.product_id(),
            device.usage(),
            device.usage_page(),
        );
    }

    Ok(())
}
