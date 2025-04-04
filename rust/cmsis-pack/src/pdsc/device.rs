use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use crate::utils::prelude::*;
use anyhow::{format_err, Error};
use roxmltree::Node;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Core {
    Any,
    CortexM0,
    CortexM0Plus,
    CortexM1,
    CortexM3,
    CortexM4,
    CortexM7,
    CortexM23,
    CortexM33,
    CortexM35P,
    CortexM55,
    CortexM85,
    StarMC1,
    SC000,
    SC300,
    ARMV8MBL,
    ARMV8MML,
    ARMV81MML,
    CortexR4,
    CortexR5,
    CortexR7,
    CortexR8,
    CortexA5,
    CortexA7,
    CortexA8,
    CortexA9,
    CortexA15,
    CortexA17,
    CortexA32,
    CortexA35,
    CortexA53,
    CortexA57,
    CortexA72,
    CortexA73,
}

impl FromStr for Core {
    type Err = Error;
    fn from_str(from: &str) -> Result<Self, Error> {
        match from {
            "Cortex-M0" => Ok(Core::CortexM0),
            "Cortex-M0+" => Ok(Core::CortexM0Plus),
            "Cortex-M1" => Ok(Core::CortexM1),
            "Cortex-M3" => Ok(Core::CortexM3),
            "Cortex-M4" => Ok(Core::CortexM4),
            "Cortex-M7" => Ok(Core::CortexM7),
            "Cortex-M23" => Ok(Core::CortexM23),
            "Cortex-M33" => Ok(Core::CortexM33),
            "Cortex-M35P" => Ok(Core::CortexM35P),
            "Cortex-M55" => Ok(Core::CortexM55),
            "Cortex-M85" => Ok(Core::CortexM85),
            "Star-MC1" => Ok(Core::StarMC1),
            "SC000" => Ok(Core::SC000),
            "SC300" => Ok(Core::SC300),
            "ARMV8MBL" => Ok(Core::ARMV8MBL),
            "ARMV8MML" => Ok(Core::ARMV8MML),
            "Cortex-R4" => Ok(Core::CortexR4),
            "Cortex-R5" => Ok(Core::CortexR5),
            "Cortex-R7" => Ok(Core::CortexR7),
            "Cortex-R8" => Ok(Core::CortexR8),
            "Cortex-A5" => Ok(Core::CortexA5),
            "Cortex-A7" => Ok(Core::CortexA7),
            "Cortex-A8" => Ok(Core::CortexA8),
            "Cortex-A9" => Ok(Core::CortexA9),
            "Cortex-A15" => Ok(Core::CortexA15),
            "Cortex-A17" => Ok(Core::CortexA17),
            "Cortex-A32" => Ok(Core::CortexA32),
            "Cortex-A35" => Ok(Core::CortexA35),
            "Cortex-A53" => Ok(Core::CortexA53),
            "Cortex-A57" => Ok(Core::CortexA57),
            "Cortex-A72" => Ok(Core::CortexA72),
            "Cortex-A73" => Ok(Core::CortexA73),
            "*" => Ok(Core::Any),
            unknown => Err(format_err!("Unknown core {}", unknown)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FPU {
    None,
    SinglePrecision,
    DoublePrecision,
}

impl FromStr for FPU {
    type Err = Error;
    fn from_str(from: &str) -> Result<Self, Error> {
        match from {
            "FPU" => Ok(FPU::SinglePrecision),
            "SP_FPU" => Ok(FPU::SinglePrecision),
            "1" => Ok(FPU::SinglePrecision),
            "None" => Ok(FPU::None),
            "0" => Ok(FPU::None),
            "DP_FPU" => Ok(FPU::DoublePrecision),
            "2" => Ok(FPU::DoublePrecision),
            unknown => Err(format_err!("Unknown fpu {}", unknown)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MPU {
    NotPresent,
    Present,
}

impl FromStr for MPU {
    type Err = Error;
    fn from_str(from: &str) -> Result<Self, Error> {
        match from {
            "MPU" => Ok(MPU::Present),
            "1" => Ok(MPU::Present),
            "None" => Ok(MPU::NotPresent),
            "0" => Ok(MPU::NotPresent),
            unknown => Err(format_err!("Unknown fpu {}", unknown)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Processor {
    pub core: Core,
    pub fpu: FPU,
    pub mpu: MPU,
    pub ap: AccessPort,
    pub dp: u8,
    pub address: Option<u32>,
    pub svd: Option<String>,
    pub name: Option<String>,
    pub unit: usize,
    pub default_reset_sequence: Option<String>,
}

#[derive(Debug, Clone)]
struct ProcessorBuilder {
    core: Option<Core>,
    units: Option<usize>,
    name: Option<String>,
    fpu: Option<FPU>,
    mpu: Option<MPU>,
}

impl ProcessorBuilder {
    fn merge(&mut self, other: &Self) {
        self.core = self.core.clone().or(other.core.clone());
        self.units = self.units.or(other.units);
        self.name = self.name.clone().or(other.name.clone());
        self.fpu = self.fpu.clone().or(other.fpu.clone());
        self.mpu = self.mpu.clone().or(other.mpu.clone());
    }
    fn build(self, debugs: &[Debug]) -> Result<Vec<Processor>, Error> {
        let units = self.units.unwrap_or(1);
        let name = self.name.clone();

        (0..units)
            .map(|unit| {
                // The attributes we're interested in may be spread across multiple debug
                // attributes defined in the family, subfamily, or device; and which may or may not
                // be specific to a given Pname or Punit.
                //
                // We'll prioritize the first element with the attribute we're interested in, since
                // family and subfamily debug elements are appended after device debug elements.
                let debugs_iterator = debugs.iter().filter(|debug| {
                    // If Pname or Punit are present on the <debug> element, they must match.
                    debug
                        .name
                        .as_ref()
                        .map_or(true, |n| Some(n) == name.as_ref())
                        && debug.unit.map_or(true, |u| u == unit)
                });

                Ok(Processor {
                    core: self
                        .core
                        .clone()
                        .ok_or_else(|| format_err!("No Core found!"))?,
                    fpu: self.fpu.clone().unwrap_or(FPU::None),
                    mpu: self.mpu.clone().unwrap_or(MPU::NotPresent),
                    dp: debugs_iterator
                        .clone()
                        .find_map(|d| d.dp)
                        .unwrap_or_default(),
                    ap: debugs_iterator
                        .clone()
                        .find_map(|d| d.ap)
                        .unwrap_or_default(),
                    address: debugs_iterator.clone().find_map(|d| d.address),
                    svd: debugs_iterator.clone().find_map(|d| d.svd.clone()),
                    name: name.clone(),
                    unit,
                    default_reset_sequence: debugs_iterator
                        .clone()
                        .find_map(|d| d.default_reset_sequence.clone()),
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

impl FromElem for ProcessorBuilder {
    fn from_elem(e: &Node) -> Result<Self, Error> {
        Ok(ProcessorBuilder {
            core: attr_parse(e, "Dcore").ok(),
            units: attr_parse(e, "Punits").ok(),
            fpu: attr_parse(e, "Dfpu").ok(),
            mpu: attr_parse(e, "Dmpu").ok(),
            name: attr_parse(e, "Pname").ok(),
        })
    }
}

#[derive(Debug, Clone)]
struct ProcessorsBuilder(Vec<ProcessorBuilder>);

impl ProcessorsBuilder {
    fn merge(self, parent: &Option<Self>) -> Result<Self, Error> {
        if let Some(parent) = parent {
            let mut current = self
                .0
                .into_iter()
                .map(|p| (p.name.clone(), p))
                .collect::<HashMap<Option<String>, ProcessorBuilder>>();

            for parent in parent.0.iter() {
                let current = current
                    .entry(parent.name.clone())
                    .or_insert_with(|| parent.clone());
                current.merge(parent);
            }

            let result = current.into_values().collect();
            Ok(Self(result))
        } else {
            Ok(self)
        }
    }

    fn merge_into(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    fn build(self, debugs: Vec<Debug>) -> Result<Vec<Processor>, Error> {
        let mut vec = vec![];
        for processor in self.0.into_iter() {
            vec.extend(processor.build(&debugs)?);
        }
        Ok(vec)
    }
}

impl FromElem for ProcessorsBuilder {
    fn from_elem(e: &Node) -> Result<Self, Error> {
        Ok(ProcessorsBuilder(vec![ProcessorBuilder::from_elem(e)?]))
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum AccessPort {
    Index(u8),
    Address(u64),
}
impl Default for AccessPort {
    fn default() -> Self {
        Self::Index(0)
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Debug {
    pub dp: Option<u8>,
    pub ap: Option<AccessPort>,
    pub address: Option<u32>,
    pub svd: Option<String>,
    pub name: Option<String>,
    pub unit: Option<usize>,
    pub default_reset_sequence: Option<String>,
}

#[derive(Debug, Clone)]
struct DebugBuilder {
    dp: Option<u8>,
    ap: Option<AccessPort>,
    address: Option<u32>,
    svd: Option<String>,
    name: Option<String>,
    unit: Option<usize>,
    default_reset_sequence: Option<String>,
}

impl DebugBuilder {
    fn build(self) -> Debug {
        Debug {
            dp: self.dp,
            ap: self.ap,
            address: self.address,
            svd: self.svd,
            name: self.name,
            unit: self.unit,
            default_reset_sequence: self.default_reset_sequence,
        }
    }
}

impl DebugBuilder {
    fn from_elem_and_parent(e: &Node, p: &Node) -> Result<Self, Error> {
        let c = p
            .children()
            .map(|n| n.tag_name().name())
            .collect::<Vec<_>>();
        let (dp, ap) = if c.contains(&"accessportV1") || c.contains(&"accessportV2") {
            let __apid: u32 = attr_parse(e, "__apid")?;
            let ap = p
                .children()
                .find(|c| {
                    c.tag_name().name().starts_with("accessportV")
                        && attr_parse(c, "__apid")
                            .map(|apid: u32| apid == __apid)
                            .unwrap_or(false)
                })
                .ok_or_else(|| anyhow::anyhow!("Unable do find Access Port with id {__apid:?}."))?;
            match ap.tag_name().name() {
                "accessportV1" => (
                    attr_parse(&ap, "__dp").ok(),
                    attr_parse(&ap, "index").ok().map(AccessPort::Index),
                ),
                "accessportV2" => (
                    attr_parse(&ap, "__dp").ok(),
                    attr_parse_hex(&ap, "address").ok().map(AccessPort::Address),
                ),
                _ => unreachable!(),
            }
        } else {
            (
                attr_parse(e, "__dp").ok(),
                attr_parse(e, "__ap").ok().map(AccessPort::Index),
            )
        };

        Ok(DebugBuilder {
            dp,
            ap,
            address: attr_parse(e, "address").ok(),
            svd: attr_parse(e, "svd").ok(),
            name: attr_parse(e, "Pname").ok(),
            unit: attr_parse(e, "Punit").ok(),
            default_reset_sequence: attr_parse(e, "defaultResetSequence").ok(),
        })
    }
}

#[derive(Debug)]
struct DebugsBuilder(Vec<DebugBuilder>);

impl DebugsBuilder {
    fn from_elem_and_parent(e: &Node, p: &Node) -> Result<Self, Error> {
        Ok(DebugsBuilder(vec![DebugBuilder::from_elem_and_parent(
            e, p,
        )?]))
    }
}

impl DebugsBuilder {
    fn merge(mut self, parent: &Self) -> Self {
        self.0.extend(parent.0.iter().cloned());
        self
    }

    fn merge_into(&mut self, other: Self) {
        self.0.extend(other.0)
    }

    fn build(self) -> Vec<Debug> {
        self.0.into_iter().map(|debug| debug.build()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub peripheral: bool,
    pub secure: bool,
    pub non_secure: bool,
    pub non_secure_callable: bool,
}

impl MemoryPermissions {
    fn from_str(input: &str) -> Self {
        let mut ret = MemoryPermissions {
            read: false,
            write: false,
            execute: false,
            peripheral: false,
            secure: false,
            non_secure: false,
            non_secure_callable: false,
        };
        for c in input.chars() {
            match c {
                'r' => ret.read = true,
                'w' => ret.write = true,
                'x' => ret.execute = true,
                'p' => ret.peripheral = true,
                's' => ret.secure = true,
                'n' => ret.non_secure = true,
                'c' => ret.non_secure_callable = true,
                _ => (),
            }
        }
        ret
    }
}

enum NumberBool {
    False,
    True,
}

impl From<NumberBool> for bool {
    fn from(val: NumberBool) -> Self {
        match val {
            NumberBool::True => true,
            NumberBool::False => false,
        }
    }
}

impl FromStr for NumberBool {
    type Err = Error;
    fn from_str(from: &str) -> Result<Self, Error> {
        match from {
            "true" => Ok(NumberBool::True),
            "1" => Ok(NumberBool::True),
            "false" => Ok(NumberBool::False),
            "0" => Ok(NumberBool::False),
            unknown => Err(format_err!(
                "unkown boolean found in merory startup {}",
                unknown
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub p_name: Option<String>,
    pub access: MemoryPermissions,
    pub start: u64,
    pub size: u64,
    pub startup: bool,
    pub default: bool,
}

struct MemElem(String, Memory);

impl FromElem for MemElem {
    fn from_elem(e: &Node) -> Result<Self, Error> {
        let access = MemoryPermissions::from_str(e.attribute("access").unwrap_or_else(|| {
            let memtype = e.attribute("id").unwrap_or_default();
            if memtype.contains("ROM") {
                "rx"
            } else if memtype.contains("RAM") {
                "rw"
            } else {
                ""
            }
        }));
        let name = e
            .attribute("id")
            .or_else(|| e.attribute("name"))
            .map(|s| s.to_string())
            .ok_or_else(|| format_err!("No name found for memory"))?;
        let p_name = e.attribute("Pname").map(|s| s.to_string());
        let start = attr_parse_hex(e, "start")?;
        let size = attr_parse_hex(e, "size")?;
        let startup = attr_parse(e, "startup")
            .map(|nb: NumberBool| nb.into())
            .unwrap_or_default();
        let default = attr_parse(e, "default")
            .map(|nb: NumberBool| nb.into())
            .unwrap_or_default();
        Ok(MemElem(
            name,
            Memory {
                p_name,
                access,
                start,
                size,
                startup,
                default,
            },
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memories(pub HashMap<String, Memory>);

fn merge_memories(lhs: Memories, rhs: &Memories) -> Memories {
    let rhs: Vec<_> = rhs
        .0
        .iter()
        .filter_map(|(k, v)| {
            if lhs.0.contains_key(k) {
                None
            } else {
                Some((k.clone(), v.clone()))
            }
        })
        .collect();
    let mut lhs = lhs;
    lhs.0.extend(rhs);
    lhs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmStyle {
    Keil,
    IAR,
    CMSIS,
}

impl FromStr for AlgorithmStyle {
    type Err = Error;
    fn from_str(from: &str) -> Result<Self, Error> {
        match from {
            "Keil" => Ok(AlgorithmStyle::Keil),
            "IAR" => Ok(AlgorithmStyle::IAR),
            "CMSIS" => Ok(AlgorithmStyle::CMSIS),
            unknown => Err(format_err!("Unknown algorithm style {}", unknown)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Algorithm {
    pub file_name: PathBuf,
    pub start: u64,
    pub size: u64,
    pub default: bool,
    pub ram_start: Option<u64>,
    pub ram_size: Option<u64>,
    pub style: AlgorithmStyle,
}

impl FromElem for Algorithm {
    fn from_elem(e: &Node) -> Result<Self, Error> {
        let default = attr_parse(e, "default")
            .map(|nb: NumberBool| nb.into())
            .unwrap_or_default();

        let file_name: &str = attr_map(e, "name")?;
        let style = attr_parse(e, "style").ok().unwrap_or(AlgorithmStyle::Keil);
        Ok(Self {
            file_name: file_name.replace('\\', "/").into(),
            start: attr_parse_hex(e, "start")?,
            size: attr_parse_hex(e, "size")?,
            ram_start: attr_parse_hex(e, "RAMstart").ok(),
            ram_size: attr_parse_hex(e, "RAMsize").ok(),
            default,
            style,
        })
    }
}

#[derive(Debug)]
struct DeviceBuilder {
    name: Option<String>,
    algorithms: Vec<Algorithm>,
    memories: Memories,
    processor: Option<ProcessorsBuilder>,
    debugs: DebugsBuilder,
    vendor: Option<String>,
    family: Option<String>,
    sub_family: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Device {
    pub name: String,
    pub memories: Memories,
    pub algorithms: Vec<Algorithm>,
    pub processors: Vec<Processor>,
    pub vendor: Option<String>,
    pub family: String,
    pub sub_family: Option<String>,
}

impl DeviceBuilder {
    fn from_elem(e: &Node) -> Self {
        let memories = Memories(HashMap::new());
        let mut family = None;
        let mut sub_family = None;
        if e.tag_name().name() == "family" {
            family = e.attribute("Dfamily").map(|f| f.to_string());
        }
        if e.tag_name().name() == "subFamily" {
            sub_family = e.attribute("DsubFamily").map(|f| f.to_string());
        }

        DeviceBuilder {
            name: e
                .attribute("Dname")
                .or_else(|| e.attribute("Dvariant"))
                .map(|f| f.to_string()),
            vendor: e.attribute("Dvendor").map(|f| f.to_string()),
            memories,
            algorithms: Vec::new(),
            processor: None,
            debugs: DebugsBuilder(Vec::new()),
            family,
            sub_family,
        }
    }

    fn build(self) -> Result<Device, Error> {
        let name = self
            .name
            .ok_or_else(|| format_err!("Device found without a name"))?;
        let family = self
            .family
            .ok_or_else(|| format_err!("Device found without a family"))?;

        let debugs = self.debugs.build();

        let processors = match self.processor {
            Some(pb) => pb.build(debugs)?,
            None => return Err(format_err!("Device found without a processor {}", name)),
        };

        Ok(Device {
            processors,
            name,
            memories: self.memories,
            algorithms: self.algorithms,
            vendor: self.vendor,
            family,
            sub_family: self.sub_family,
        })
    }

    fn add_parent(mut self, parent: &Self) -> Result<Self, Error> {
        self.algorithms.extend_from_slice(&parent.algorithms);
        Ok(Self {
            name: self.name.or(parent.name.clone()),
            algorithms: self.algorithms,
            memories: merge_memories(self.memories, &parent.memories),
            processor: match self.processor {
                Some(old_proc) => Some(old_proc.merge(&parent.processor)?),
                None => parent.processor.clone(),
            },
            debugs: self.debugs.merge(&parent.debugs),
            vendor: self.vendor.or(parent.vendor.clone()),
            family: self.family.or(parent.family.clone()),
            sub_family: self.sub_family.or(parent.sub_family.clone()),
        })
    }

    fn add_processor(&mut self, processor: ProcessorsBuilder) -> &mut Self {
        match self.processor {
            None => self.processor = Some(processor),
            Some(ref mut origin) => origin.merge_into(processor),
        };
        self
    }

    fn add_debug(&mut self, debug: DebugsBuilder) -> &mut Self {
        self.debugs.merge_into(debug);
        self
    }

    fn add_memory(&mut self, MemElem(name, mem): MemElem) -> &mut Self {
        self.memories.0.insert(name, mem);
        self
    }

    fn add_algorithm(&mut self, alg: Algorithm) -> &mut Self {
        self.algorithms.push(alg);
        self
    }
}

fn parse_device(e: &Node) -> Vec<DeviceBuilder> {
    let mut device = DeviceBuilder::from_elem(e);
    let variants: Vec<DeviceBuilder> = e
        .children()
        .filter_map(|child| match child.tag_name().name() {
            "variant" => Some(DeviceBuilder::from_elem(&child)),
            "memory" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|mem| device.add_memory(mem));
                None
            }
            "algorithm" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|alg| device.add_algorithm(alg));
                None
            }
            "processor" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|prc| device.add_processor(prc));
                None
            }
            "debug" => {
                DebugsBuilder::from_elem_and_parent(&child, e)
                    .ok_warn()
                    .map(|debug| device.add_debug(debug));
                None
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if variants.is_empty() {
        vec![device]
    } else {
        variants
            .into_iter()
            .flat_map(|bld| bld.add_parent(&device).ok_warn())
            .collect()
    }
}

fn parse_sub_family(e: &Node) -> Vec<DeviceBuilder> {
    let mut sub_family_device = DeviceBuilder::from_elem(e);
    let mut devices: Vec<DeviceBuilder> = Vec::new();

    for child in e.children() {
        match child.tag_name().name() {
            "device" => {
                devices.extend(parse_device(&child));
            }
            "memory" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|mem| sub_family_device.add_memory(mem));
            }
            "algorithm" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|alg| sub_family_device.add_algorithm(alg));
            }
            "processor" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|prc| sub_family_device.add_processor(prc));
            }
            "debug" => {
                DebugsBuilder::from_elem_and_parent(&child, e)
                    .ok_warn()
                    .map(|debug| sub_family_device.add_debug(debug));
            }
            _ => continue,
        }
    }
    devices
        .into_iter()
        .flat_map(|bldr| bldr.add_parent(&sub_family_device).ok_warn())
        .collect()
}

fn parse_family(e: &Node) -> Result<Vec<Device>, Error> {
    let mut family_device = DeviceBuilder::from_elem(e);
    let all_devices: Vec<DeviceBuilder> = e
        .children()
        .flat_map(|child| match child.tag_name().name() {
            "subFamily" => parse_sub_family(&child),
            "device" => parse_device(&child),
            "memory" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|mem| family_device.add_memory(mem));
                Vec::new()
            }
            "algorithm" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|alg| family_device.add_algorithm(alg));
                Vec::new()
            }
            "processor" => {
                FromElem::from_elem(&child)
                    .ok_warn()
                    .map(|prc| family_device.add_processor(prc));
                Vec::new()
            }
            "debug" => {
                DebugsBuilder::from_elem_and_parent(&child, e)
                    .ok_warn()
                    .map(|debug| family_device.add_debug(debug));
                Vec::new()
            }
            _ => Vec::new(),
        })
        .collect::<Vec<_>>();
    all_devices
        .into_iter()
        .map(|bldr| bldr.add_parent(&family_device).and_then(|dev| dev.build()))
        .collect()
}

#[derive(Default, Serialize)]
pub struct Devices(pub HashMap<String, Device>);

impl FromElem for Devices {
    fn from_elem(e: &Node) -> Result<Self, Error> {
        e.children()
            .try_fold(HashMap::new(), |mut res, c| {
                let add_this = parse_family(&c)?;
                res.extend(add_this.into_iter().map(|dev| (dev.name.clone(), dev)));
                Ok(res)
            })
            .map(Devices)
    }
}
