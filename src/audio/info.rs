use std::collections::HashMap;

use cpal::traits::{DeviceTrait, HostTrait};
use tracing::info;

#[derive(Debug)]
pub struct OutHosts {
    pub hosts: HashMap<String, OutDevices>,
    pub default: String,
}

#[derive(Debug)]
pub struct OutDevices {
    pub devices: Vec<OutDevice>,
    pub default: Option<String>,
}

#[derive(Debug)]
pub struct OutDevice {
    pub name: String,
    pub configs: Vec<OutDeviceConfig>,
}

#[derive(Debug)]
pub struct OutDeviceConfig {
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub num_channels: u16,
    pub min_buffer_size: u32,
    pub max_buffer_size: u32,
}

pub fn get_available_outputs() -> OutHosts {
    OutHosts {
        hosts: get_available_hosts_struct(cpal::available_hosts()),
        default: cpal::default_host().id().name().to_owned(),
    }
}

pub fn get_default_host_name() -> String {
    String::from(cpal::default_host().id().name())
}

pub fn get_default_host() -> cpal::Host {
    cpal::default_host()
}

pub fn get_default_output_device_name(host: &cpal::Host) -> Option<String> {
    match host.default_output_device() {
        Some(dev) => get_device_name(&dev),
        None => None,
    }
}

pub fn print_info() {
    let hosts = get_available_outputs();
    info!("Available Outputs:");
    info!("    Default Host: {}", hosts.default);
    info!("    Hosts:");
    for (name, devices) in hosts.hosts {
        info!("        <{name}>");
        info!(
            "            Default Device: {}",
            devices.default.unwrap_or("<None>".to_owned())
        );
        info!("            Devices:");
        for device in devices.devices {
            info!("                <{}>", device.name);
            for (cfg_i, cfg) in device.configs.iter().enumerate() {
                info!("                    <{cfg_i}>:");
                info!(
                    "                        Sample Rate: {} - {}:",
                    cfg.min_sample_rate, cfg.max_sample_rate
                );
                info!(
                    "                        Buffer Size: {} - {}:",
                    cfg.min_buffer_size, cfg.max_buffer_size
                );
                info!(
                    "                        Num Channels: {}:",
                    cfg.num_channels
                );
            }
        }
    }
}

fn get_available_hosts_struct(available_hosts: Vec<cpal::HostId>) -> HashMap<String, OutDevices> {
    available_hosts.iter().fold(
        HashMap::with_capacity(available_hosts.len()),
        |mut res_hosts, host_id| {
            push_avail_output_devs(*host_id, &mut res_hosts);
            res_hosts
        },
    )
}

fn push_avail_output_devs(host_id: cpal::HostId, res_hosts: &mut HashMap<String, OutDevices>) {
    if let Ok(host) = cpal::host_from_id(host_id) {
        res_hosts.insert(
            host.id().name().to_owned(),
            OutDevices {
                devices: get_output_devices(&host),
                default: get_default_output_device_name(&host),
            },
        );
    }
}

fn get_output_devices(host: &cpal::Host) -> Vec<OutDevice> {
    match host.output_devices() {
        Ok(devices) => devices.filter_map(|dev| get_output_device(&dev)).collect(),
        Err(_) => vec![],
    }
}

fn get_output_device(dev: &cpal::Device) -> Option<OutDevice> {
    Some(OutDevice {
        name: get_device_name(dev)?,
        configs: get_device_supported_out_configs(dev)?,
    })
}

fn get_device_name(device: &cpal::Device) -> Option<String> {
    device.name().ok().map(|name| name.to_owned())
}

fn get_device_supported_out_configs(device: &cpal::Device) -> Option<Vec<OutDeviceConfig>> {
    let mut result = vec![];
    let cfgs = device.supported_output_configs().ok()?;
    for cfg in cfgs {
        result.push(get_out_device_config_from(cfg));
    }
    Some(result)
}

fn get_out_device_config_from(cfg: cpal::SupportedStreamConfigRange) -> OutDeviceConfig {
    let buffer_size = match cfg.buffer_size() {
        cpal::SupportedBufferSize::Range { min, max } => (*min, *max),
        cpal::SupportedBufferSize::Unknown => (std::u32::MIN, std::u32::MAX),
    };
    OutDeviceConfig {
        min_sample_rate: cfg.min_sample_rate().0,
        max_sample_rate: cfg.max_sample_rate().0,
        num_channels: cfg.channels(),
        min_buffer_size: buffer_size.0,
        max_buffer_size: buffer_size.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info() {
        let hosts = get_available_outputs();
        println!("Available Outputs:");
        println!("    Default Host: {}", hosts.default);
        println!("    Hosts:");
        for (name, devices) in hosts.hosts {
            println!("        <{name}>");
            println!(
                "            Default Device: {}",
                devices.default.unwrap_or("<None>".to_owned())
            );
            println!("            Devices:");
            for device in devices.devices {
                println!("                <{}>", device.name);
                for (cfg_i, cfg) in device.configs.iter().enumerate() {
                    println!("                    <{cfg_i}>:");
                    println!(
                        "                        Sample Rate: {} - {}:",
                        cfg.min_sample_rate, cfg.max_sample_rate
                    );
                    println!(
                        "                        Buffer Size: {} - {}:",
                        cfg.min_buffer_size, cfg.max_buffer_size
                    );
                    println!(
                        "                        Num Channels: {}:",
                        cfg.num_channels
                    );
                }
            }
        }
    }
}
