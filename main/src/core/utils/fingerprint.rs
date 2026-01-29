use std::env;
use std::os::windows::process::CommandExt;
use std::process::Command;

use base64::prelude::*;

pub struct Fingerprint {
    crypto_id: String,
    pc_name: String,
    user_name: String,
    cpu_model: String,
    processor_iden: String,
    gpu_model: String,
    product_id: String,
    mac_address: String,
    os_name: String,
    os_version: String,
    os_build: String,
    disk_id: String,
    processor_id: String,
    local_ip: String
}

impl Fingerprint {
    pub fn new() -> Self {
        Self {
            crypto_id: get_powershell("(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Cryptography').MachineGuid"),
            pc_name: env::var("COMPUTERNAME").unwrap_or_default(),
            user_name: env::var("USERNAME").unwrap_or_default(),
            cpu_model: get_powershell("(Get-CimInstance -ClassName Win32_Processor).Name"),
            processor_iden: env::var("PROCESSOR_IDENTIFIER").unwrap_or_default(),
            gpu_model: get_powershell("(Get-CimInstance -Class Win32_VideoController).Caption"),
            product_id: get_powershell("(Get-CimInstance -Class Win32_ComputerSystemProduct).UUID"),
            mac_address: get_powershell("(Get-CimInstance Win32_NetworkAdapterConfiguration | Where-Object IPEnabled).MACAddress -join ' '"),
            os_name: get_powershell("(Get-CimInstance -Class Win32_OperatingSystem).Caption"),
            os_version: get_powershell("(Get-CimInstance -Class Win32_OperatingSystem).Version"),
            os_build: get_powershell("(Get-CimInstance -Class Win32_OperatingSystem).BuildNumber"),
            disk_id: get_powershell("(Get-CimInstance -Class Win32_DiskDrive).SerialNumber"),
            processor_id: get_powershell("(Get-CimInstance -Class Win32_Processor).ProcessorId"),
            local_ip: get_powershell("(Get-NetIPAddress -AddressFamily IPv4 -InterfaceAlias Ethernet).IPAddress")
        }
    }

    pub fn get_hash(&self) -> String {
        let fingerprint = format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            self.crypto_id,
            self.pc_name,
            self.user_name,
            self.cpu_model,
            self.processor_iden,
            self.gpu_model,
            self.product_id,
            self.mac_address,
            self.os_name,
            self.os_version,
            self.os_build,
            self.disk_id,
            self.processor_id,
            self.local_ip
        );

        let hash = lib::hash::sha224_short(fingerprint.as_bytes());

        BASE64_STANDARD
            .encode(hash)
            .replace("=", "")
            .to_uppercase()
    }

    pub fn get_readable(&self) -> String {
        [
            "crypto_id: ", &self.crypto_id, "\n",
            "pc_name: ", &self.pc_name, "\n",
            "user_name: ", &self.user_name, "\n",
            "cpu_model: ", &self.cpu_model, "\n",
            "processor_iden: ", &self.processor_iden, "\n",
            "gpu_model: ", &self.gpu_model, "\n",
            "product_id: ", &self.product_id, "\n",
            "mac_address: ", &self.mac_address, "\n",
            "os_name: ", &self.os_name, "\n",
            "os_version: ", &self.os_version, "\n",
            "os_build: ", &self.os_build, "\n",
            "disk_id: ", &self.disk_id, "\n",
            "processor_id: ", &self.processor_id, "\n",
            "local_ip: ", &self.local_ip
        ].concat()
    }
}

fn get_powershell(arg: &str) -> String {
    let output = Command::new("powershell.exe")
        .arg(arg)
        .creation_flags(0x00000008)
        .output()
        .ok()
        .map(|out| out.stdout)
        .unwrap_or_default();

    String::from_utf8_lossy(&output)
        .trim()
        .to_string()
}