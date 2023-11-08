use anyhow::{anyhow, Result};
use serde::{de::Visitor, Deserialize, Serialize};

use crate::{params::Param, DecodedParams, Value};

/// Contract ABI (Abstract Binary Interface).
///
/// This struct holds defitions for a contracts' ABI.
///
/// ```no_run
/// use ola_lang_abi::Abi;
///
/// let abi_json =  r#"[{
///     "type": "function",
///     "name": "f",
///     "inputs": [{"type": "u32", "name": "x"}]}
/// ]"#;
///
/// let abi: Abi = serde_json::from_str(abi_json).unwrap();
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Abi {
    /// Contract defined functions.
    pub functions: Vec<Function>,
}

impl Abi {
    // Decode function input from slice.
    pub fn decode_input_from_slice<'a>(
        &'a self,
        input: &[u64],
    ) -> Result<(&'a Function, DecodedParams)> {
        let f = self
            .functions
            .iter()
            .find(|f| f.method_id() == input[0])
            .ok_or_else(|| anyhow!("ABI function not found"))?;

        // input = [method_id, param-len, param1, param2, ...]
        let decoded_params = f.decode_input_from_slice(&input[2..])?;

        Ok((f, decoded_params))
    }

    pub fn encode_input_with_signature(
        &self,
        signature: &str,
        params: &[Value],
    ) -> Result<Vec<u64>> {
        let f = self
            .functions
            .iter()
            .find(|f| f.signature() == signature)
            .ok_or_else(|| anyhow!("ABI function not found"))?;

        let mut enc_input = vec![f.method_id()];

        let params = Value::encode(params);
        enc_input.push(params.len() as u64);
        enc_input.extend(params);

        Ok(enc_input)
    }

    pub fn encode_input_values(&self, params: &[Value]) -> Result<Vec<u64>> {
        let mut enc_input = vec![];

        let params = Value::encode(params);
        enc_input.push(params.len() as u64);
        enc_input.extend(params);

        Ok(enc_input)
    }
}

impl Serialize for Abi {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut entries = vec![];

        for f in &self.functions {
            entries.push(AbiEntry {
                type_: String::from("function"),
                name: Some(f.name.clone()),
                inputs: Some(f.inputs.clone()),
                outputs: Some(f.outputs.clone()),
            });
        }
        entries.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Abi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(AbiVisitor)
    }
}

/// Contract function definition.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function {
    /// Function name.
    pub name: String,
    /// Function inputs.
    pub inputs: Vec<Param>,
    /// Function outputs.
    pub outputs: Vec<Param>,
}

impl Function {
    /// Computes the function's method id (function selector).
    pub fn method_id(&self) -> u64 {
        use tiny_keccak::{Hasher, Keccak};

        let mut keccak_out = [0u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(self.signature().as_bytes());
        hasher.finalize(&mut keccak_out);
        u32::from_le_bytes(keccak_out[0..4].try_into().unwrap()) as u64
    }

    /// Returns the function's signature.
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

    // Decode function input from slice.
    pub fn decode_input_from_slice(&self, input: &[u64]) -> Result<DecodedParams> {
        let inputs_types = self
            .inputs
            .iter()
            .map(|f_input| f_input.type_.clone())
            .collect::<Vec<_>>();

        Ok(DecodedParams::from(
            self.inputs
                .iter()
                .cloned()
                .zip(Value::decode_from_slice(input, &inputs_types)?)
                .collect::<Vec<_>>(),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AbiEntry {
    #[serde(rename = "type")]
    type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inputs: Option<Vec<Param>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<Vec<Param>>,
}

struct AbiVisitor;

impl<'de> Visitor<'de> for AbiVisitor {
    type Value = Abi;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "ABI")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut abi = Abi { functions: vec![] };

        loop {
            let entry = seq.next_element::<AbiEntry>()?;

            match entry {
                None => return Ok(abi),

                Some(entry) => match entry.type_.as_str() {
                    "function" => {
                        let inputs = entry.inputs.unwrap_or_default();

                        let outputs = entry.outputs.unwrap_or_default();

                        let name = entry.name.ok_or_else(|| {
                            serde::de::Error::custom("missing function name".to_string())
                        })?;

                        abi.functions.push(Function {
                            name,
                            inputs,
                            outputs,
                        });
                    }

                    _ => {
                        return Err(serde::de::Error::custom(format!(
                            "invalid ABI entry type: {}",
                            entry.type_
                        )))
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::types::Type;

    use super::*;

    const TEST_ABI: &str = r#"[
        {
          "name": "contract_init",
          "type": "function",
          "inputs": [
            {
              "name": "proposalNames_",
              "type": "u32[]",
              "internalType": "u32[]"
            }
          ],
          "outputs": []
        },
        {
          "name": "winningProposal",
          "type": "function",
          "inputs": [],
          "outputs": [
            {
              "name": "winningProposal_",
              "type": "u32",
              "internalType": "u32"
            }
          ]
        },
        {
          "name": "getWinnerName",
          "type": "function",
          "inputs": [],
          "outputs": [
            {
              "name": "",
              "type": "u32",
              "internalType": "u32"
            }
          ]
        },
        {
          "name": "vote_proposal",
          "type": "function",
          "inputs": [
            {
              "name": "proposal_",
              "type": "u32",
              "internalType": "u32"
            }
          ],
          "outputs": []
        },
        {
          "name": "get_caller",
          "type": "function",
          "inputs": [],
          "outputs": [
            {
              "name": "",
              "type": "address",
              "internalType": "address"
            }
          ]
        },
        {
          "name": "vote_test",
          "type": "function",
          "inputs": [],
          "outputs": []
        }
      ]"#;

    fn test_function() -> Function {
        Function {
            name: "funname".to_string(),
            inputs: vec![
                Param {
                    name: "".to_string(),
                    type_: Type::Address,
                },
                Param {
                    name: "x".to_string(),
                    type_: Type::FixedArray(Box::new(Type::U32), 2),
                },
            ],
            outputs: vec![],
        }
    }

    #[test]
    fn function_signature() {
        let fun = test_function();
        assert_eq!(fun.signature(), "funname(address,u32[2])");
    }

    #[test]
    fn function_method_id() {
        let fun = test_function();
        assert_eq!(fun.method_id(), 0xf146ff09);
    }

    #[test]
    fn abi_function_decode_input_from_slice() {
        let addr = [1, 2, 3, 4];
        let uint1 = 37;
        let uint2 = 109;

        let input_values = vec![
            Value::Address(crate::FixedArray4(addr)),
            Value::FixedArray(vec![Value::U32(uint1), Value::U32(uint2)], Type::U32),
        ];

        let fun = test_function();
        let abi = Abi {
            functions: vec![fun],
        };

        let mut enc_input = vec![abi.functions[0].method_id()];

        let params = Value::encode(&input_values);
        enc_input.push(params.len() as u64);
        enc_input.extend(params);
        let dec = abi
            .decode_input_from_slice(&enc_input)
            .expect("decode_input_from_slice failed");

        let expected_decoded_params = DecodedParams::from(
            abi.functions[0]
                .inputs
                .iter()
                .cloned()
                .zip(input_values)
                .collect::<Vec<(Param, Value)>>(),
        );

        assert_eq!(dec, (&abi.functions[0], expected_decoded_params));
    }

    #[test]
    fn abi_json_work() {
        let v = serde_json::json!([
            {
                "inputs": [
                    {
                        "internalType": "u32",
                        "name": "n",
                        "type": "u32"
                    },
                    {
                        "components": [
                            {
                                "internalType": "u32",
                                "name": "a",
                                "type": "u32"
                            },
                            {
                                "internalType": "string",
                                "name": "b",
                                "type": "string"
                            }
                        ],
                        "internalType": "struct A.X",
                        "name": "x",
                        "type": "tuple"
                    }
                ],
                "name": "f",
                "outputs": [],
                "type": "function"
            }
        ]);

        let abi: Abi = serde_json::from_str(&v.to_string()).unwrap();

        assert_eq!(
            abi,
            Abi {
                functions: vec![Function {
                    name: "f".to_string(),
                    inputs: vec![
                        Param {
                            name: "n".to_string(),
                            type_: Type::U32,
                        },
                        Param {
                            name: "x".to_string(),
                            type_: Type::Tuple(vec![
                                ("a".to_string(), Type::U32),
                                ("b".to_string(), Type::String)
                            ]),
                        }
                    ],
                    outputs: vec![],
                }],
            }
        );
    }

    #[test]
    fn test_serde() {
        let abi: Abi = serde_json::from_str(TEST_ABI).unwrap();

        let ser_abi = serde_json::to_string(&abi).expect("serialized abi");
        let de_abi: Abi = serde_json::from_str(&ser_abi).expect("deserialized abi");

        assert_eq!(abi, de_abi);
    }
}
