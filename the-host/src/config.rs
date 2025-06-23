use std::collections::HashMap;
use bevy::asset::{AssetLoader, Assets, AsyncReadExt, LoadContext};
use bevy::prelude::{Asset, Component, Handle, Res, TypePath, warn, Resource};
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::hash::Hash;
use std::ops::{Deref};
use bevy::asset::io::Reader;
use bevy::tasks::ConditionalSendFuture;
use thiserror::Error;

#[derive(Component)]
pub struct ConfigValueMap<T>(pub HashMap<T, ConfigValue>);

impl<T> Deref for ConfigValueMap<T> {
    type Target = HashMap<T, ConfigValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: PartialEq + Eq + Hash> ConfigValueMap<K> {
    pub fn cv(&self, key: K) -> &ConfigValue {
        self.0.get(&key).unwrap()
    }
}

// macro to make RacingConfig
// macro_rules! make_config_enum {
//     ($name:ident, $($cfg_ident:ident, $json_key:expr, $expected_type:expr),* ) => {
//          
//      }
// }

pub struct ConfigValue {
    path: ConfigPath,
    config: Handle<Config>,
}

impl ConfigValue {
    pub fn new(value_path: ConfigPath, config: Handle<Config>) -> ConfigValue {
        ConfigValue {
            path: value_path,
            config
        }
    }
}

#[derive(TypePath, Asset, Deserialize)]
pub struct Config(pub Value);

#[derive(Resource)]
pub struct ConfigAccessor {
    pub handle: Handle<Config>
}

#[derive(Default)]
pub struct ConfigLoader;
    
pub fn load_config_from_path(path: &str) -> Option<Config> {
    let file = File::open(path).ok()?;
    serde_json::from_reader(file).ok()
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConfigAssetLoaderError {
    // /// An [IO](std::io) Error
    // #[error("Could not load asset: {0}")]
    // Io(#[from] std::io::Error),
    // /// A [RON](ron) Error
    // #[error("Could not parse RON: {0}")]
    // RonSpannedError(#[from] ron::error::SpannedError),
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parsing error
    #[error("Bad JSON file: {0}")]
    JsonParseError(#[from] serde_json::Error)
}

impl AssetLoader for ConfigLoader {
    type Asset = Config;
    type Settings = ();
    type Error = ConfigAssetLoaderError;

    async fn load(
            &self,
            reader: &mut dyn Reader,
            _settings: &Self::Settings,
            _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let json = serde_json::from_reader(&bytes[..])?;
        Ok(Config(json))
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}


impl Deref for Config {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub enum ConfigType {
    Int,   // usize
    Float, // f32
    String, // String
           // don't care about other stufef right now
}

pub struct ConfigPath(Vec<ConfigPathElem>, ConfigType);
pub enum ConfigPathElem {
    Key(String),
    Index(usize),
}

struct JsonValueWithExpectedType(Value, ConfigType);

impl ConfigPath {
    pub fn new(path: Vec<ConfigPathElem>, expected_type: ConfigType) -> ConfigPath {
        ConfigPath(path, expected_type)
    }
    pub fn key(key: &str, expected_type: ConfigType) -> ConfigPath {
        ConfigPath(vec![ConfigPathElem::Key(key.to_string())], expected_type)
    }
    pub fn get<T: From<JsonValueWithExpectedType>>(&self, config: &Config) -> T {
        // follow path
        // clone and .into()
        let mut value = &config.0;
        for path_elem in self.0.iter() {
            value = match path_elem {
                ConfigPathElem::Key(s) => &value[s],
                ConfigPathElem::Index(i) => &value[i],
            };
        }
        JsonValueWithExpectedType(value.clone(), self.1).into()
    }
}

pub trait GetConfig {
    fn get_config_value<T: From<JsonValueWithExpectedType> + Default>(&self, cv: &ConfigValue) -> T;
}

impl GetConfig for Res<'_, Assets<Config>> {
    fn get_config_value<T: From<JsonValueWithExpectedType> + Default>(&self, cv: &ConfigValue) -> T {
        self.get(cv.config.id())
            .map(|config| cv.path.get(config))
            .unwrap_or_default()
    }
}

macro_rules! impl_from_jvwet {
    ($utype:ty) => {
        impl From<JsonValueWithExpectedType> for $utype {
            fn from(value: JsonValueWithExpectedType) -> Self {
                let json = value.0;
                let expected_type = value.1;
                match expected_type {
                    ConfigType::Int => json.as_u64().unwrap() as $utype,
                    ConfigType::Float => {
                        let f = json.as_f64().unwrap();
                        warn!("Float config value {f} being reinterpreted as unsigned integer");
                        f as $utype
                    }
                    ConfigType::String => {
                        let s = json.as_str().unwrap();
                        warn!("Got string config value {s}, expected unsigned integer type. Substituting 0.");
                        0
                    }
                }
            }
        }
    }
}

impl_from_jvwet!(usize);
impl_from_jvwet!(u64);
impl_from_jvwet!(u32);
impl_from_jvwet!(u16);
impl_from_jvwet!(u8);

macro_rules! impl_from_jvwet_signed_int {
    ($itype:ty) => {
        impl From<JsonValueWithExpectedType> for $itype {
            fn from(value: JsonValueWithExpectedType) -> Self {
                let json = value.0;
                let expected_type = value.1;
                match expected_type {
                    ConfigType::Int => json.as_i64().unwrap() as $itype,
                    ConfigType::Float => {
                        let f = json.as_f64().unwrap();
                        warn!("Float config value {f} being reinterpreted as signed integer");
                        f as $itype
                    }
                    ConfigType::String => {
                        let s = json.as_str().unwrap();
                        warn!(
                            "Got string config value {s}, expected signed integer. Substituting 0."
                        );
                        0
                    }
                }
            }
        }
    };
}

impl_from_jvwet_signed_int!(isize);
impl_from_jvwet_signed_int!(i64);
impl_from_jvwet_signed_int!(i32);
impl_from_jvwet_signed_int!(i16);
impl_from_jvwet_signed_int!(i8);

macro_rules! impl_from_jvwet_float {
    ($ftype:ty) => {
        impl From<JsonValueWithExpectedType> for $ftype {
            fn from(value: JsonValueWithExpectedType) -> Self {
                let json = value.0;
                let expected_type = value.1;
                match expected_type {
                    ConfigType::Int => json.as_i64().unwrap() as $ftype,
                    ConfigType::Float => {
                        let f = json.as_f64().unwrap();
                        f as $ftype
                    }
                    ConfigType::String => {
                        let s = json.as_str().unwrap();
                        warn!("Got string config value {s}, expected float. Substituting 0.");
                        0.
                    }
                }
            }
        }
    };
}

impl_from_jvwet_float!(f64);
impl_from_jvwet_float!(f32);

impl From<JsonValueWithExpectedType> for String {
    fn from(value: JsonValueWithExpectedType) -> Self {
        match value.1 {
            ConfigType::Int => {
                warn!("Reinterpreting Config Integer as empty string.");
                "".to_string()
            }
            ConfigType::Float => {
                warn!("Reinterpreting Config Float as an empty string.");
                "".to_string()
            }
            ConfigType::String => value.0.as_str().unwrap().to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::{ConfigPath, ConfigPathElem, ConfigType, load_config_from_path};

    #[test]
    fn test_1() {
        let config = load_config_from_path("config/racing.json").unwrap();
        println!("Car size: {}", config["car-size"]);
    }

    #[test]
    fn test_2() {
        let config = load_config_from_path("config/racing.json").unwrap();
        let value = ConfigPath(
            vec![ConfigPathElem::Key("car-size".to_string())],
            ConfigType::Float,
        );
        let car_size: f32 = value.get(&config);
    }
}
