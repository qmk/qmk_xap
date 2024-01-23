use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    fs::{self, read_dir, File},
    io::Write,
    path::Path,
    vec,
};

use anyhow::Result;
use convert_case::{Case, Casing};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct XAPVersion(u32);

impl TryFrom<u32> for XAPVersion {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x01 | 0x0100 | 0x0200 | 0x0300 => Ok(Self(value)),
            _ => Err(anyhow::anyhow!(format!(
                "{value:06X} is not a valid BCD encoded XAP version"
            ))),
        }
    }
}

impl Display for XAPVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for digit in self.0.to_be_bytes() {
            write!(f, "{digit:02X}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
enum BasicType {
    #[default]
    Unit,
    U8,
    U16,
    U32,
    U64,
    Struct,
    String,
    Array(Box<BasicType>, usize),
}

impl BasicType {
    fn as_type(&self) -> String {
        match self {
            BasicType::Unit => "()".to_owned(),
            BasicType::U8 => "u8".to_owned(),
            BasicType::U16 => "u16".to_owned(),
            BasicType::U32 => "u32".to_owned(),
            BasicType::U64 => "u64".to_owned(),
            BasicType::Struct => "Struct".to_owned(),
            BasicType::String => "UTF8String".to_owned(),
            BasicType::Array(ty, len) => format!("[{}; {len}]", ty.as_type()),
        }
    }
}

struct BasicTypeVisitor;

impl<'de> Visitor<'de> for BasicTypeVisitor {
    type Value = BasicType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a string representing a basic type like u8, u16, u32, u64, string, array or struct",
        )
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some((ty, len)) = value.split_once('[') {
            let len = len
                .strip_suffix(']')
                .ok_or_else(|| E::custom("array type missing closing bracket"))?;
            let len = len
                .parse::<usize>()
                .map_err(|_| E::custom("array type length is not a number"))?;

            return Ok(BasicType::Array(Box::new(self.visit_str(ty)?), len));
        }

        match value {
            "u8" => Ok(BasicType::U8),
            "u16" => Ok(BasicType::U16),
            "u32" => Ok(BasicType::U32),
            "u64" => Ok(BasicType::U64),
            "struct" => Ok(BasicType::Struct),
            "string" => Ok(BasicType::String),
            _ => Err(E::custom(format!("unknown basic type: {value}"))),
        }
    }
}

impl<'de> Deserialize<'de> for BasicType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(BasicTypeVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct StructMember {
    name: String,
    r#type: BasicType,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "lowercase")]
enum RouteType {
    #[default]
    Router,
    Command,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
struct Route {
    #[serde(skip_deserializing)]
    id: Vec<u8>,
    #[serde(skip_deserializing)]
    xap_version: Option<XAPVersion>,
    name: Option<String>,
    description: Option<String>,
    r#type: Option<RouteType>,
    #[serde(deserialize_with = "deserialize_routes")]
    routes: BTreeMap<u8, Route>,
    return_type: BasicType,
    return_struct_members: Vec<StructMember>,
    request_type: BasicType,
    request_struct_members: Vec<StructMember>,
}

use std::fmt::Write as fmtWrite;

impl Route {
    fn render_id(&self) -> String {
        let mut rendered = String::new();

        let mut bytes = self.id.iter().peekable();
        while let Some(byte) = bytes.next() {
            rendered.push_str(&format!("{byte:02x}"));

            if bytes.peek().is_some() {
                rendered.push_str(", ");
            }
        }

        rendered
    }

    fn merge(&mut self, other: &Route) {
        if other.name.is_some() {
            self.name = other.name.clone();
        }
        if other.description.is_some() {
            self.description = other.description.clone();
        }
        if other.r#type.is_some() {
            self.r#type = other.r#type.clone();
        }
        for (key, other_route) in &other.routes {
            if let Some(route) = self.routes.get_mut(key) {
                route.merge(other_route);
            } else {
                self.routes.insert(*key, other_route.clone());
            }
        }
    }

    fn render_request_type(&self, ctx: &mut Context) -> Result<(String, String)> {
        let name = format!("{}{}", ctx.module_name, self.name.as_ref().unwrap());

        let (request_type_name, request_type) = match self.request_type {
            BasicType::Struct => {
                let request_struct_name = format!("{}RequestArg", name.to_case(Case::Pascal));

                let mut request_struct = format!(
                    r#"
                    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
                    pub struct {request_struct_name} {{
                    "#
                );

                for member in &self.request_struct_members {
                    writeln!(
                        &mut request_struct,
                        "    pub {}: {},",
                        member.name.to_case(Case::Snake),
                        member.r#type.as_type()
                    )?;
                }

                writeln!(&mut request_struct, "}}\n")?;

                (request_struct_name, request_struct)
            }
            _ => (self.request_type.as_type(), String::new()),
        };

        Ok((request_type_name, request_type))
    }

    fn render_return_type(&self, ctx: &mut Context) -> Result<(String, String)> {
        let name = format!("{}{}", ctx.module_name, self.name.as_ref().unwrap());

        let (return_type_name, return_type) = match self.return_type {
            BasicType::Unit => ("()".to_owned(), String::new()),
            _ => {
                let response_struct_name = format!("{}Response", name.to_case(Case::Pascal));
                let mut response_struct = String::new();

                if self.return_type == BasicType::Struct {
                    writeln!(
                        &mut response_struct,
                        r#"
                        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
                        pub struct {response_struct_name} {{
                        "#
                    )?;
                    for member in &self.return_struct_members {
                        writeln!(
                            &mut response_struct,
                            "    pub {}: {},",
                            member.name.to_case(Case::Snake),
                            member.r#type.as_type()
                        )?;
                    }
                    writeln!(&mut response_struct, "}}\n")?;
                } else {
                    writeln!(
                        &mut response_struct,
                        r#"
                        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
                        pub struct {response_struct_name}(pub {});
                        "#,
                        self.return_type.as_type()
                    )?;
                }

                (response_struct_name, response_struct)
            }
        };

        Ok((return_type_name, return_type))
    }

    fn render_command(&self, ctx: &mut Context) -> Result<()> {
        let command_name = self.name.as_ref().unwrap();
        let name = format!("{}{}", ctx.module_name, command_name);
        let name_pascal = name.to_case(Case::Pascal);
        let name_snake = name.to_case(Case::Snake);
        let description = self.description.as_ref().unwrap().replace("\n", "\n/// ");
        let (request_type_name, request_type) = self.render_request_type(ctx)?;
        let (response_type_name, response_type) = self.render_return_type(ctx)?;
        let id = self.render_id();
        let xap_version = self.xap_version.as_ref().unwrap();

        write!(
            &mut ctx.spec,
            r#"
            /// ======================================================================
            /// {command_name}
            ///
            /// {description}
            /// ======================================================================
            #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
            pub struct {name_pascal}Request(pub {request_type_name});

            {request_type}

            impl XAPRequest for {name_pascal}Request {{
                type Response = {response_type_name};

                fn id() -> &'static [u8] {{
                    &[{id}]
                }}

                fn xap_version() -> u32 {{
                    0x{xap_version}
                }}
            }}

            {response_type}"#
        )?;

        match self.request_type {
            BasicType::Unit => {
                writeln!(
                    &mut ctx.spec,
                    r#"
                    #[tauri::command]
                    #[specta::specta]
                    pub fn {name_snake}(
                        id: Uuid,
                        state: State<'_, Arc<Mutex<XAPClient>>>,
                    ) -> ClientResult<{response_type_name}> {{
                        state
                            .lock()
                            .query(id, {name_pascal}Request(()))
                    }}
                    "#
                )?;
            }
            _ => {
                writeln!(
                    &mut ctx.spec,
                    r#"
                    #[tauri::command]
                    #[specta::specta]
                    pub fn {name_snake}(
                        id: Uuid,
                        arg: {request_type_name},
                        state: State<'_, Arc<Mutex<XAPClient>>>,
                    ) -> ClientResult<{response_type_name}> {{
                        state
                            .lock()
                            .query(id, {name_pascal}Request(arg))
                    }}
                    "#
                )?;
            }
        }

        Ok(())
    }

    fn render_route(&self, ctx: &mut Context) -> Result<()> {
        let module_name = self
            .name
            .as_ref()
            .expect("route name is missing")
            .to_case(Case::Snake);

        ctx.module_name = module_name.clone();

        writeln!(
            &mut ctx.spec,
            r#"
            #[allow(dead_code)]
            #[allow(unused_imports)]
            pub mod {module_name}_routes {{
                use std::sync::Arc;

                use binrw::{{BinRead, BinWrite}};
                use parking_lot::Mutex;
                use serde::{{Serialize, Deserialize}};
                use specta::Type;
                use tauri::State;
                use uuid::Uuid;

                use xap_specs::request::XAPRequest;
                use xap_specs::response::UTF8String;
                use crate::xap::hid::XAPClient;
                use crate::xap::ClientResult;
            "#
        )?;

        for (_, subroute) in &self.routes {
            subroute.render(ctx)?;
        }

        writeln!(
            &mut ctx.spec,
            r#"
            }}
            "#
        )?;

        Ok(())
    }

    fn render(&self, ctx: &mut Context) -> Result<()> {
        match self.r#type.as_ref().expect("route type is missing") {
            RouteType::Router => {
                self.render_route(ctx)?;
            }
            RouteType::Command => {
                self.render_command(ctx)?;
                for (_, subroute) in &self.routes {
                    subroute.render(ctx)?;
                }
            }
        }
        Ok(())
    }

    fn expand_ids(&mut self, parent_id: &mut Vec<u8>) {
        parent_id.extend_from_slice(&self.id);
        self.id = parent_id.clone();

        for (_, route) in &mut self.routes {
            route.expand_ids(&mut parent_id.clone());
        }
    }

    fn expand_xap_specs(&mut self, xap_version: XAPVersion) {
        if self.xap_version.is_none() || self.xap_version > Some(xap_version) {
            self.xap_version.replace(xap_version);
        }

        for (_, route) in &mut self.routes {
            route.expand_xap_specs(xap_version);
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Message {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize, Clone)]
struct BroadcastMessages {
    #[serde(default)]
    messages: HashMap<String, Message>,
}

#[derive(Debug, Deserialize, Clone)]
struct Spec {
    #[serde(deserialize_with = "deserialize_xap_version")]
    // #[serde(skip_deserializing)]
    version: XAPVersion,
    #[serde(default, deserialize_with = "deserialize_routes")]
    routes: BTreeMap<u8, Route>,
    broadcast_messages: Option<BroadcastMessages>,
}

impl Spec {
    fn merge(&mut self, other: &Spec) {
        for (id, route) in &other.routes {
            match self.routes.get_mut(&id) {
                Some(existing_route) => {
                    existing_route.merge(route);
                }
                None => {
                    self.routes.insert(*id, route.clone());
                }
            }
        }

        for (_, route) in &mut self.routes {
            route.expand_xap_specs(self.version)
        }
    }

    fn render(&self, ctx: &mut Context) -> Result<()> {
        for (_, route) in self.routes.iter() {
            route.render(ctx)?;
        }

        Ok(())
    }

    fn from_spec(path: &Path) -> Result<Spec> {
        let content = fs::read_to_string(path)?;

        let mut spec = deser_hjson::from_str::<Spec>(&content)?;

        for (_, route) in &mut spec.routes {
            route.expand_ids(&mut Vec::new());
            route.expand_xap_specs(spec.version)
        }

        Ok(spec)
    }
}

fn deserialize_routes<'de, D>(d: D) -> Result<BTreeMap<u8, Route>, D::Error>
where
    D: Deserializer<'de>,
{
    let routes: BTreeMap<String, Route> = Deserialize::deserialize(d)?;

    routes
        .into_iter()
        .map(|(id, mut route)| {
            let id = u8::from_str_radix(&id.trim_start_matches("0x"), 16)
                .map_err(serde::de::Error::custom)?;
            route.id = vec![id];
            Ok((id, route))
        })
        .collect::<Result<BTreeMap<u8, Route>, D::Error>>()
}

fn deserialize_xap_version<'de, D>(d: D) -> Result<XAPVersion, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: String = Deserialize::deserialize(d)?;

    let mut version = [0_u8; 4];

    for (i, digit) in raw.split('.').enumerate().take(3) {
        version[version.len() - (i + 2)] = digit.parse::<u8>().map_err(|e| {
            serde::de::Error::custom(format!(
                "failed to parse digit {digit} in version string {raw} with error {e}"
            ))
        })?;
    }

    Ok(
        XAPVersion::try_from(u32::from_le_bytes(version)).map_err(|e| {
            serde::de::Error::custom(format!(
                "failed to deserialize XAP version from {raw} with error {e}"
            ))
        })?,
    )
}

struct Context<'a> {
    spec: &'a mut dyn Write,
    module_name: String,
}

fn main() -> Result<()> {
    let mut specs = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/specs/xap"))?
        .filter_map(|spec| {
            let spec = spec.ok()?;
            let spec_name = spec.file_name().to_str().map(|name| name.to_owned())?;
            if spec_name.ends_with(".hjson") {
                Some(spec_name)
            } else {
                None
            }
        })
        .map(|spec_name| {
            eprintln!("Reading spec {}", spec_name);
            Spec::from_spec(Path::new(&format!(
                "{}/specs/xap/{}",
                env!("CARGO_MANIFEST_DIR"),
                spec_name
            )))
        })
        .collect::<Result<Vec<Spec>>>()?;

    // Make sure we process specs in ascending order, as they build upon eachother
    specs.sort_by(|lhs, rhs| lhs.version.cmp(&rhs.version));

    for i in 1..specs.len() {
        let spec_lower_version = specs[i - 1].clone();
        specs[i].merge(&spec_lower_version);
    }

    // Only render the latest spec as it contains all previous iterations
    if let Some(spec) = specs.last() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        let mut context = Context {
            spec: &mut File::create(format!("{manifest_dir}/../src-tauri/src/xap_spec.rs"))?,
            module_name: String::new(),
        };

        spec.render(&mut context)?;
    }

    Ok(())
}
