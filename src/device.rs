//! @file device.rs
//! @brief Explicit wired-device selection and checked HID feature exchanges.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

use crate::protocol::{FeatureTransport, Packet, transmit};
use anyhow::{Context, Result, ensure};
use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::{ffi::CString, thread, time::Duration};

#[derive(Clone, Debug, PartialEq)]
pub struct Keyboard {
    pub path: CString,
    pub manufacturer: String,
    pub product: String,
    pub serial: String,
}

fn matches(info: &DeviceInfo) -> bool {
    info.vendor_id() == 0x05ac
        && info.product_id() == 0x024f
        && info.interface_number() == 0
        && is_wired(
            info.manufacturer_string().unwrap_or(""),
            info.product_string().unwrap_or(""),
        )
}

pub fn is_wired(manufacturer: &str, product: &str) -> bool {
    manufacturer.eq_ignore_ascii_case("hfd.cn") && product.eq_ignore_ascii_case("USB DEVICE")
}

pub fn scan() -> Result<Vec<Keyboard>> {
    let api = HidApi::new().context("HID 初始化失败")?;
    Ok(api
        .device_list()
        .filter(|i| matches(i))
        .map(|i| Keyboard {
            path: i.path().to_owned(),
            manufacturer: i.manufacturer_string().unwrap_or("").into(),
            product: i.product_string().unwrap_or("").into(),
            serial: i.serial_number().unwrap_or("").into(),
        })
        .collect())
}

struct Transport(HidDevice);

impl FeatureTransport for Transport {
    fn exchange(&mut self, report: &[u8; 65]) -> Result<()> {
        self.0
            .send_feature_report(report)
            .context("发送 HID feature report 失败")?;
        thread::sleep(Duration::from_millis(5));
        let mut response = [0u8; 65];
        let count = self
            .0
            .get_feature_report(&mut response)
            .context("读取 HID feature report 失败")?;
        ensure!(count > 0, "设备返回空报文");
        // Reference code does not document an ACK signature. A successful HID
        // exchange is not a verification of the keyboard's stored settings.
        Ok(())
    }
}

pub fn write(target: &Keyboard, packets: &[Packet]) -> Result<usize> {
    let api = HidApi::new().context("HID 初始化失败")?;
    let info = api
        .device_list()
        .find(|i| i.path() == target.path.as_c_str())
        .context("设备已断开，请刷新设备列表")?;
    ensure!(matches(info), "设备身份已变化，已停止写入");
    ensure!(
        info.serial_number().unwrap_or("") == target.serial,
        "设备序列号已变化，已停止写入"
    );
    let device = api
        .open_path(&target.path)
        .context("打开键盘失败；请检查 hidraw 的当前用户访问权限")?;
    transmit(&mut Transport(device), packets, |ms| {
        thread::sleep(Duration::from_millis(ms))
    })
}
