pub mod constants;
pub mod error;
pub mod protocol;
pub mod request;
pub mod response;
pub mod token;

#[cfg(test)]
mod test {
    use std::{
        collections::{BTreeMap, HashMap},
        fs::{self, read_dir, File},
        io::Write,
        path::Path,
        vec,
    };

    use anyhow::Result;
    use convert_case::{Case, Casing};
    use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
    use serde_json::{from_value, Value};
    use tera::{Context, Tera};

    fn build_tera() -> Result<Tera> {
        let mut tera = Tera::new("templates/**/*.tera")?;
        tera.build_inheritance_chains()?;

        tera.register_filter("rust_type", |value: &Value, _: &HashMap<String, Value>| {
            let basic_type: BasicType = from_value(value.clone())?;
            Ok(Value::String(basic_type.as_rust_type()))
        });

        tera.register_filter(
            "pascal_case",
            |value: &Value, _: &HashMap<String, Value>| {
                let s: String = from_value(value.clone())?;
                Ok(Value::String(s.to_case(Case::Pascal)))
            },
        );

        tera.register_filter("snake_case", |value: &Value, _: &HashMap<String, Value>| {
            let s: String = from_value(value.clone())?;
            Ok(Value::String(s.to_case(Case::Snake)))
        });

        Ok(tera)
    }

    #[derive(Debug, Serialize, Default, Clone)]
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
        fn as_rust_type(&self) -> String {
            match self {
                BasicType::Unit => "()".to_owned(),
                BasicType::U8 => "u8".to_owned(),
                BasicType::U16 => "u16".to_owned(),
                BasicType::U32 => "u32".to_owned(),
                BasicType::U64 => "u64".to_owned(),
                BasicType::Struct => "Struct".to_owned(),
                BasicType::String => "UTF8String".to_owned(),
                BasicType::Array(ty, len) => format!("[{}; {len}]", ty.as_rust_type()),
            }
        }
    }

    struct BasicTypeVisitor;

    impl<'de> Visitor<'de> for BasicTypeVisitor {
        type Value = BasicType;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter
                .write_str("a string representing a basic type like u8, u16, u32, u64, string, array or struct")
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

    #[derive(Debug, Deserialize, Default, Clone)]
    #[serde(default)]
    struct Route {
        #[serde(skip_deserializing)]
        id: Vec<u8>,
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

    impl Route {
        fn render_id(&self) -> String {
            let mut id = String::new();

            let mut bytes = self.id.iter().peekable();
            while let Some(byte) = bytes.next() {
                id.push_str(&format!("{byte:02x}"));
                if bytes.peek().is_some() {
                    id.push_str(", ");
                }
            }

            id
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

        fn render(&self, tera: &Tera) -> Result<String> {
            let mut rendered = String::new();
            match self.r#type.as_ref().expect("route type is missing") {
                RouteType::Router => {
                    let mut context = Context::new();
                    context.insert(
                        "module_name",
                        &self
                            .name
                            .as_ref()
                            .expect("route name is missing")
                            .to_case(Case::Snake),
                    );

                    let mut routes = String::new();
                    for (_, subroute) in &self.routes {
                        routes.push_str(&subroute.render(tera)?);
                    }
                    context.insert("routes", &routes);
                    rendered.push_str(&tera.render("router.tera", &context)?);
                }
                RouteType::Command => {
                    let mut context = Context::new();
                    context.insert("id", &self.render_id());
                    context.insert("name", &self.name);
                    context.insert(
                        "name_pascal_case",
                        &self
                            .name
                            .as_ref()
                            .expect("route name is missing")
                            .to_case(Case::Pascal),
                    );
                    context.insert(
                        "description",
                        &self
                            .description
                            .as_ref()
                            .map(|desc| desc.replace("\n", "\n/// ")),
                    );

                    context.insert("request_type", &self.request_type.as_rust_type());
                    context.insert("request_struct_members", &self.request_struct_members);

                    context.insert("return_type", &self.return_type.as_rust_type());
                    context.insert("return_struct_members", &self.return_struct_members);

                    rendered.push_str(&tera.render("command.tera", &context)?);

                    for (_, subroute) in &self.routes {
                        rendered.push_str(&subroute.render(tera)?);
                    }
                }
            }

            Ok(rendered)
        }

        fn expand_ids(&mut self, parent_id: &mut Vec<u8>) {
            parent_id.extend_from_slice(&self.id);
            self.id = parent_id.clone();

            for (_, route) in &mut self.routes {
                route.expand_ids(&mut parent_id.clone());
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
        version: String,
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
        }

        fn render(&self, tera: &Tera) -> Result<String> {
            let mut rendered = String::new();
            for (_, route) in self.routes.iter() {
                rendered.push_str(&route.render(&tera)?);
            }

            Ok(prettyplease::unparse(&syn::parse_file(&rendered)?))
        }

        fn from_spec(path: &Path) -> Result<Spec> {
            let content = fs::read_to_string(path)?;

            let mut spec = deser_hjson::from_str::<Spec>(&content)?;

            for (_, route) in &mut spec.routes {
                route.expand_ids(&mut Vec::new());
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

    #[test]
    fn do_it() -> Result<()> {
        // Listen for changes to templates and specs, but not generated files
        // println!("cargo:rerun-if-changed=specs");
        // println!("cargo:rerun-if-changed=templates");

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
            let spec = specs[i - 1].clone();
            specs[i].merge(&spec);
        }

        let tera = build_tera()?;

        for spec in specs {
            File::create(format!(
                "{}/src/xap_{}.rs",
                env!("CARGO_MANIFEST_DIR"),
                spec.version.to_owned().replace(".", "_")
            ))?
            .write_all(spec.render(&tera)?.as_bytes())?;
        }

        Ok(())
    }
}
