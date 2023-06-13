use crate::files::file_metadata::file_name::FileName;
use crate::parser::parsed_file_data::ParsedFileData;
use anyhow::{anyhow, Result};
use redis::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::serde::ulid_as_u128;
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Event {
  FileSaved {
    #[serde(with = "ulid_as_u128")]
    file_id: Ulid,
    file_name: FileName,
  },
  FileParsed {
    #[serde(with = "ulid_as_u128")]
    file_id: Ulid,
    file_name: FileName,
    data: ParsedFileData,
  },
  FileParseFailed {
    #[serde(with = "ulid_as_u128")]
    file_id: Ulid,
    file_name: FileName,
    error: String,
  },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EventPayload {
  pub event: Event,
  pub correlation_id: Option<String>,
  pub metadata: Option<HashMap<String, String>>,
}

impl From<EventPayload> for HashMap<String, String> {
  fn from(val: EventPayload) -> Self {
    let mut result = HashMap::new();
    result.insert(
      "event".to_string(),
      serde_json::to_string(&val.event).unwrap(),
    );
    result.insert(
      "metadata".to_string(),
      serde_json::to_string(&val.metadata.unwrap_or(HashMap::new())).unwrap(),
    );
    if let Some(correlation_id) = val.correlation_id {
      result.insert("correlation_id".to_string(), correlation_id);
    }
    result
  }
}

fn get_value_as_string(value: &redis::Value) -> Result<String> {
  match value {
    redis::Value::Data(raw) => Ok(String::from_utf8(raw.clone())?),
    _ => Err(anyhow::anyhow!("data was not binary")),
  }
}

impl TryFrom<HashMap<String, Value>> for EventPayload {
  type Error = anyhow::Error;

  fn try_from(value: HashMap<String, Value>) -> Result<Self> {
    let event = serde_json::from_str::<Event>(&get_value_as_string(
      value.get("event").ok_or(anyhow!("event not found"))?,
    )?)?;
    let correlation_id = value
      .get("correlation_id")
      .map(|value| get_value_as_string(value).unwrap());
    let metadata = value.get("metadata").map(|value| {
      serde_json::from_str::<HashMap<String, String>>(&get_value_as_string(value).unwrap()).unwrap()
    });
    Ok(EventPayload {
      event,
      correlation_id,
      metadata,
    })
  }
}

pub enum Stream {
  File,
  Parser,
  Lookup,
}

impl Stream {
  pub fn tag(&self) -> String {
    match &self {
      Stream::File => "file".to_string(),
      Stream::Parser => "parser".to_string(),
      Stream::Lookup => "lookup".to_string(),
    }
  }

  pub fn redis_key(&self) -> String {
    format!("event:stream:{}", &self.tag())
  }

  pub fn redis_cursor_key(&self, subscriber_id: &str) -> String {
    format!("event:stream:{}:cursor:{}", &self.tag(), subscriber_id)
  }
}
