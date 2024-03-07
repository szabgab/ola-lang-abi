use anyhow::{anyhow, Result};
use mini_goldilocks::poseidon::unsafe_poseidon_bytes_auto_padded;
use std::collections::VecDeque;

use crate::{DecodedParams, FixedArray4, Param, Type, Value};

/// Contract Error Definition
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Error {
    /// Error name.
    pub name: String,
    /// Error inputs.
    pub inputs: Vec<Param>,
}

/// Contract event definition.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Event {
    /// Event name.
    pub name: String,
    /// Event inputs.
    pub inputs: Vec<Param>,
    /// Whether the event is anonymous or not.
    pub anonymous: bool,
}

impl Event {
    /// Returns the event's signature.
    pub fn signature(&self) -> String {
        format!(
            "{}({})",
            self.name,
            self.inputs
                .iter()
                .map(|param| param.type_.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    /// Compute the event's topic hash

    pub fn topic(&self) -> FixedArray4 {
        FixedArray4(unsafe_poseidon_bytes_auto_padded(
            &self.signature().as_bytes(),
        ))
    }

    /// Decode event params from a log's topics and data.
    pub fn decode_data_from_slice(
        &self,
        mut topics: &[FixedArray4],
        data: &[u64],
    ) -> Result<DecodedParams> {
        // strip event topic from the topics array
        // so that we end up with only the values we
        // need to decode
        if !self.anonymous {
            topics = topics
                .get(1..)
                .ok_or_else(|| anyhow!("missing event topic"))?;
        }

        let mut topics_values = VecDeque::from(topics.to_vec());

        let mut data_values = VecDeque::from(Value::decode_from_slice(
            data,
            &self
                .inputs
                .iter()
                .filter(|input| !input.indexed.unwrap_or(false))
                .map(|input| input.type_.clone())
                .collect::<Vec<_>>(),
        )?);

        let mut decoded = vec![];
        for input in self.inputs.iter().cloned() {
            let decoded_value = if input.indexed.unwrap_or(false) {
                let val = topics_values
                    .pop_front()
                    .ok_or_else(|| anyhow!("insufficient topics entries"))?;

                if Self::is_encoded_to_hash(&input.type_) {
                    Ok(Value::Hash(val))
                } else if input.type_ == Type::U32
                    || input.type_ == Type::Bool
                    || input.type_ == Type::Field
                {
                    // decode value from topics entry, using the input type
                    //  If the input type is hash or address, take the value directly.
                    //  If the input type is u32, bool, field, take the last value (big-endian).

                    Value::decode_from_slice(
                        &[val.0.get(3).unwrap().clone()],
                        &[input.type_.clone()],
                    )?
                    .first()
                    .ok_or_else(|| anyhow!("no value decoded from topics entry"))
                    .map(Clone::clone)
                } else {
                    Value::decode_from_slice(&val.0, &[input.type_.clone()])?
                        .first()
                        .ok_or_else(|| anyhow!("no value decoded from topics entry"))
                        .map(Clone::clone)
                }
            } else {
                data_values
                    .pop_front()
                    .ok_or_else(|| anyhow!("insufficient data values"))
            };

            decoded.push((input, decoded_value?));
        }

        Ok(DecodedParams::from(decoded))
    }

    fn is_encoded_to_hash(ty: &Type) -> bool {
        matches!(
            ty,
            Type::FixedArray(_, _)
                | Type::U256
                | Type::Array(_)
                | Type::Fields
                | Type::String
                | Type::Tuple(_)
        )
    }
}

#[cfg(test)]
mod test {

    use crate::{Abi, DecodedParams, Type};

    use super::*;

    use pretty_assertions::assert_eq;

    fn test_event() -> Event {
        Event {
            name: "Approve".to_string(),
            inputs: vec![
                Param {
                    name: "x".to_string(),
                    type_: Type::U32,
                    indexed: Some(true),
                },
                Param {
                    name: "y".to_string(),
                    type_: Type::String,
                    indexed: Some(true),
                },
            ],
            anonymous: false,
        }
    }

    #[test]
    fn test_poseidon_hash() {
        let result = unsafe_poseidon_bytes_auto_padded("world".as_bytes());
        assert_eq!(
            result,
            [
                1298737262017568572,
                12445360621592034485,
                13004999764278192581,
                3441866816748036873
            ]
        );
    }

    #[test]
    fn test_signature() {
        let evt = test_event();

        assert_eq!(evt.signature(), "Approve(u32,string)");
    }

    #[test]
    fn test_topic() {
        let evt = test_event();
        assert_eq!(
            evt.topic(),
            FixedArray4::from("0xF9C165D12ACC9776822FF3684D676F567781B3609185E4A01ED1EA5138EAF215")
        );
    }

    #[test]
    fn test_decode_data_from_slice() {
        let topics: Vec<_> = vec![
            FixedArray4([
                13964306673005018703,
                10894260269595496822,
                17848333703059337299,
                3412739309839435658,
            ]),
            FixedArray4([0, 0, 0, 10]),
            FixedArray4([0, 0, 0, 11]),
        ];

        let data = vec![1, 2, 3, 97, 98, 99];

        let x = Param {
            name: "x".to_string(),
            type_: Type::U32,
            indexed: None,
        };
        let y = Param {
            name: "y".to_string(),
            type_: Type::U32,
            indexed: Some(true),
        };
        let x1 = Param {
            name: "x1".to_string(),
            type_: Type::U32,
            indexed: None,
        };
        let y1 = Param {
            name: "y1".to_string(),
            type_: Type::U32,
            indexed: Some(true),
        };
        let s = Param {
            name: "s".to_string(),
            type_: Type::String,
            indexed: None,
        };

        let evt = Event {
            name: "Test".to_string(),
            inputs: vec![x.clone(), y.clone(), x1.clone(), y1.clone(), s.clone()],
            anonymous: false,
        };

        let abi = Abi {
            functions: vec![],
            events: vec![evt],
        };

        assert_eq!(
            abi.decode_log_from_slice(&topics, &data)
                .expect("decode_log_from_slice failed"),
            (
                &abi.events[0],
                DecodedParams::from(vec![
                    (x, Value::U32(1)),
                    (y, Value::U32(10)),
                    (x1, Value::U32(2)),
                    (y1, Value::U32(11)),
                    (s, Value::String("abc".to_string()))
                ])
            )
        );
    }
}
