use anyhow::{anyhow, Result};

use crate::types::Type;

type AddressValue = [usize; 4];

type HashValue = [usize; 4];

/// ABI decoded value.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    /// Unsigned int value (uint<M>).
    U32(usize),
    /// Signed int value (int<M>).
    Field(usize),
    /// Address value (address).
    Address(AddressValue),
    /// Hash value(hash).
    Hash(HashValue),
    /// Bool value (bool).
    Bool(bool),

    /// Fixed size array value (T\[k\]).
    FixedArray(Vec<Value>, Type),
    /// UTF-8 string value (string).
    String(String),
    /// Dynamic size field value.
    Fields(Vec<usize>),
    /// Dynamic size array value (T[]).
    Array(Vec<Value>, Type),
    /// Tuple value (tuple(T1, T2, ..., Tn)).
    ///
    /// This variant's vector items have the form (name, value).
    Tuple(Vec<(String, Value)>),
}

impl Value {
    /// Decodes values from bytes using the given type hint.
    pub fn decode_from_slice(bs: &[usize], tys: &[Type]) -> Result<Vec<Value>> {
        tys.iter()
            .try_fold((vec![], 0), |(mut values, at), ty| {
                let (value, consumed) = Self::decode(bs, ty, 0, at)?;
                values.push(value);

                Ok((values, at + consumed))
            })
            .map(|(values, _)| values)
    }

    /// Encodes values into bytes.
    pub fn encode(values: &[Self]) -> Vec<usize> {
        let mut buf = vec![];
        for value in values {
            match value {
                Value::U32(i) => {
                    let start = buf.len();
                    buf.resize(start + 1, *i);
                }

                Value::Field(i) => {
                    let start = buf.len();
                    buf.resize(start + 1, *i);
                }

                Value::Address(addr) => {
                    let start = buf.len();
                    buf.resize(start + 4, 0);

                    // big-endian, as if it were a uint160.
                    buf[start..(start + 4)].copy_from_slice(addr);
                }

                Value::Hash(hash) => {
                    let start = buf.len();
                    buf.resize(start + 4, 0);

                    // big-endian, as if it were a uint160.
                    buf[start..(start + 4)].copy_from_slice(hash);
                }

                Value::Bool(b) => {
                    let start = buf.len();
                    buf.resize(start + 1, 0);

                    if *b {
                        buf[start] = 1;
                    }
                }

                Value::FixedArray(values, _) => {
                    // write array values
                    let bytes = Self::encode(values);
                    buf.extend(bytes);
                }

                Value::Tuple(values) => {
                    let values: Vec<_> = values.iter().cloned().map(|(_, value)| value).collect();

                    let bytes = Self::encode(&values);
                    buf.extend(bytes);
                }

                Value::String(value) => {
                    let start = buf.len();
                    let value_len = value.as_bytes().len();
                    let new_len = start + value_len + 1;
                    buf.resize(new_len, value_len);

                    // TODO Currently, Ola can only encode strings into arrays based on fields
                    // and does not support encoding into u8 type arrays.
                    // write bytes
                    buf[start + 1..(new_len)].copy_from_slice(
                        value
                            .as_bytes()
                            .into_iter()
                            .map(|x| *x as usize)
                            .collect::<Vec<usize>>()
                            .as_slice(),
                    );
                }

                Value::Fields(value) => {
                    let start = buf.len();
                    let value_len = value.len();
                    let new_len = start + value_len + 1;
                    buf.resize(new_len, value_len);

                    // write bytes
                    buf[start + 1..new_len].copy_from_slice(value);
                }

                Value::Array(values, _) => {
                    let start = buf.len();
                    buf.resize(start + 1, values.len());
                    // write array values
                    let bytes = Self::encode(values);
                    buf.extend(bytes);
                }
            };
        }

        buf
    }

    /// Returns the type of the given value.
    pub fn type_of(&self) -> Type {
        match self {
            Value::U32(_) => Type::U32,
            Value::Field(_) => Type::Field,
            Value::Address(_) => Type::Address,
            Value::Hash(_) => Type::Hash,
            Value::Bool(_) => Type::Bool,
            Value::FixedArray(values, ty) => Type::FixedArray(Box::new(ty.clone()), values.len()),
            Value::String(_) => Type::String,
            Value::Fields(_) => Type::Fields,
            Value::Array(_, ty) => Type::Array(Box::new(ty.clone())),
            Value::Tuple(values) => Type::Tuple(
                values
                    .iter()
                    .map(|(name, value)| (name.clone(), value.type_of()))
                    .collect(),
            ),
        }
    }

    fn decode(bs: &[usize], ty: &Type, base_addr: usize, at: usize) -> Result<(Value, usize)> {
        match ty {
            Type::U32 => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 1))
                    .ok_or_else(|| anyhow!("reached end of input while decoding {:?}", ty))?;

                let u32_value = slice[0];

                Ok((Value::U32(u32_value), 1))
            }

            Type::Field => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 1))
                    .ok_or_else(|| anyhow!("reached end of input while decoding {:?}", ty))?;

                let field_value = slice[0];

                Ok((Value::Field(field_value), 1))
            }

            Type::Address => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 4))
                    .ok_or_else(|| anyhow!("reached end of input while decoding {:?}", ty))?;

                let mut addr = [0usize; 4];
                addr.copy_from_slice(slice);

                Ok((Value::Address(addr), 4))
            }

            Type::Hash => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 4))
                    .ok_or_else(|| anyhow!("reached end of input while decoding {:?}", ty))?;

                let mut hash = [0usize; 4];
                hash.copy_from_slice(slice);

                Ok((Value::Hash(hash), 4))
            }

            Type::Bool => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 1))
                    .ok_or_else(|| anyhow!("reached end of input while decoding bool"))?;

                let b = slice[0] == 1;

                Ok((Value::Bool(b), 1))
            }
            Type::FixedArray(ty, size) => (0..(*size))
                .try_fold((vec![], 0), |(mut values, total_consumed), _| {
                    let (value, consumed) = Self::decode(bs, ty, base_addr, at + total_consumed)?;

                    values.push(value);

                    Ok((values, total_consumed + consumed))
                })
                .map(|(values, consumed)| (Value::FixedArray(values, *ty.clone()), consumed)),

            Type::String => {
                let (bytes_value, consumed) = Self::decode(bs, &Type::Fields, base_addr, at)?;

                let bytes = if let Value::Fields(bytes) = bytes_value {
                    bytes
                } else {
                    // should always be Value::Bytes
                    unreachable!();
                };

                let s = String::from_utf8(bytes.into_iter().map(|b| b as u8).collect())?;

                Ok((Value::String(s), consumed))
            }

            Type::Fields => {
                let at = base_addr + at;
                let field_len_slice = bs
                    .get(at..(at + 1))
                    .ok_or_else(|| anyhow!("reached end of input while decoding fields length"))?;
                let field_len = field_len_slice[0];

                let at = at + 1;
                let fields_value = bs
                    .get(at..(at + field_len))
                    .ok_or_else(|| anyhow!("reached end of input while decoding bytes"))?
                    .to_vec();

                // consumes only the first 32 bytes, i.e. the offset pointer
                Ok((Value::Fields(fields_value), field_len + 1))
            }

            Type::Array(ty) => {
                let at = base_addr + at;

                let array_len_slice = bs
                    .get(at..(at + 1))
                    .ok_or_else(|| anyhow!("reached end of input while decoding array length"))?;
                let array_len = array_len_slice[0];

                let at = at + 1;

                (0..array_len)
                    .try_fold((vec![], 0), |(mut values, total_consumed), _| {
                        let (value, consumed) = Self::decode(bs, ty, at, total_consumed)?;
                        values.push(value);

                        Ok((values, total_consumed + consumed))
                    })
                    .map(|(values, total_consumed)| {
                        (Value::Array(values, *ty.clone()), total_consumed + 1)
                    })
            }

            Type::Tuple(tys) => tys
                .iter()
                .cloned()
                .try_fold((vec![], 0), |(mut values, total_consumed), (name, ty)| {
                    let (value, consumed) = Self::decode(bs, &ty, base_addr, at + total_consumed)?;

                    values.push((name, value));

                    Ok((values, total_consumed + consumed))
                })
                .map(|(values, total_consumed)| (Value::Tuple(values), total_consumed)),
        }
    }
}

#[cfg(test)]
mod test {

    use std::vec;

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn decode_uint() {
        let bs = vec![100, 200, 300];

        let v = Value::decode_from_slice(&bs, &[Type::U32, Type::U32, Type::U32])
            .expect("decode_from_slice failed");

        assert_eq!(v, vec![Value::U32(100), Value::U32(200), Value::U32(300)]);
    }

    #[test]
    fn decode_field() {
        let bs = vec![100, 200, 300];

        let v = Value::decode_from_slice(&bs, &[Type::Field, Type::Field, Type::Field])
            .expect("decode_from_slice failed");

        assert_eq!(
            v,
            vec![Value::Field(100), Value::Field(200), Value::Field(300)]
        );
    }

    #[test]
    fn decode_address() {
        let bs = [1, 2, 3, 4];

        let v = Value::decode_from_slice(&bs, &[Type::Address]).expect("decode_from_slice failed");

        assert_eq!(v, vec![Value::Address([1, 2, 3, 4])]);
    }

    #[test]
    fn decode_hash() {
        let bs = [1, 2, 3, 4];

        let v = Value::decode_from_slice(&bs, &[Type::Hash]).expect("decode_from_slice failed");

        assert_eq!(v, vec![Value::Hash([1, 2, 3, 4])]);
    }

    #[test]
    fn decode_bool() {
        let bs = [0, 1];
        let v = Value::decode_from_slice(&bs, &[Type::Bool, Type::Bool])
            .expect("decode_from_slice failed");

        assert_eq!(v, vec![Value::Bool(false), Value::Bool(true)]);
    }

    #[test]
    fn decode_fixed_array() {
        // encode some data
        let uint1 = 5;
        let uint2 = 6;
        let uint3 = 7;
        let uint4 = 8;

        let bs = vec![uint1, uint2, uint3, uint4];

        let uint_arr2 = Type::FixedArray(Box::new(Type::U32), 2);

        let v = Value::decode_from_slice(&bs, &[Type::FixedArray(Box::new(uint_arr2.clone()), 2)])
            .expect("decode_from_slice failed");

        assert_eq!(
            v,
            vec![Value::FixedArray(
                vec![
                    Value::FixedArray(vec![Value::U32(uint1), Value::U32(uint2)], Type::U32),
                    Value::FixedArray(vec![Value::U32(uint3), Value::U32(uint4)], Type::U32)
                ],
                uint_arr2
            )]
        );
    }

    #[test]
    fn decode_string() {
        let source = "olavm"
            .as_bytes()
            .into_iter()
            .map(|x| *x as usize)
            .collect::<Vec<usize>>();
        let mut bs = vec![source.len() as usize];
        bs.extend_from_slice(source.as_slice());
        let v = Value::decode_from_slice(&bs, &[Type::String]).expect("decode_from_slice failed");

        let expected_str = "olavm".to_string();
        assert_eq!(v, vec![Value::String(expected_str)]);
    }

    #[test]
    fn decode_fields() {
        let source = "hello,world"
            .as_bytes()
            .into_iter()
            .map(|x| *x as usize)
            .collect::<Vec<usize>>();
        let mut bs = vec![source.len() as usize];
        bs.extend_from_slice(source.as_slice());
        let v = Value::decode_from_slice(&bs, &[Type::Fields]).expect("decode_from_slice failed");
        let expected_fields = vec![104, 101, 108, 108, 111, 44, 119, 111, 114, 108, 100];
        assert_eq!(v, vec![Value::Fields(expected_fields)]);
    }

    #[test]
    fn decode_array() {
        // encode some data
        let uint1 = 5;
        let uint2 = 6;
        let uint3 = 7;
        let uint4 = 8;
        let uint5 = 9;
        let uint6 = 10;
        let bs = vec![2, uint1, uint2, uint3, uint4, uint5, uint6];

        let uint_arr2 = Type::FixedArray(Box::new(Type::U32), 3);

        let v = Value::decode_from_slice(&bs, &[Type::Array(Box::new(uint_arr2.clone()))])
            .expect("decode_from_slice failed");

        assert_eq!(
            v,
            vec![Value::Array(
                vec![
                    Value::FixedArray(
                        vec![Value::U32(uint1), Value::U32(uint2), Value::U32(uint3)],
                        Type::U32
                    ),
                    Value::FixedArray(
                        vec![Value::U32(uint4), Value::U32(uint5), Value::U32(uint6)],
                        Type::U32
                    )
                ],
                uint_arr2
            )]
        );
    }

    #[test]
    fn decode_array2() {
        // [[1, 2, 3], [8, 9]]
        let bs = vec![3, 1, 2, 3, 2, 8, 9];

        let uint_arr2 = Type::FixedArray(Box::new(Type::Array(Box::new(Type::U32))), 2);

        let v = Value::decode_from_slice(&bs, &[uint_arr2]).expect("decode_from_slice failed");

        assert_eq!(
            v,
            vec![Value::FixedArray(
                vec![
                    Value::Array(
                        vec![Value::U32(1), Value::U32(2), Value::U32(3),],
                        Type::U32
                    ),
                    Value::Array(vec![Value::U32(8), Value::U32(9)], Type::U32),
                ],
                Type::Array(Box::new(Type::U32))
            ),],
        );
    }

    #[test]
    fn decode_fixed_tuple() {
        // encode some data
        let uint1 = 5;
        let uint2 = 6;
        let addr = [1, 2, 3, 4];
        let mut bs = vec![uint1, uint2];
        bs.extend_from_slice(&addr);

        let v = Value::decode_from_slice(
            &bs,
            &[Type::Tuple(vec![
                ("a".to_string(), Type::U32),
                ("b".to_string(), Type::U32),
                ("c".to_string(), Type::Address),
            ])],
        )
        .expect("decode_from_slice failed");

        assert_eq!(
            v,
            vec![Value::Tuple(vec![
                ("a".to_string(), Value::U32(uint1)),
                ("b".to_string(), Value::U32(uint2)),
                ("c".to_string(), Value::Address(addr))
            ])]
        );
    }

    #[test]
    fn decode_tuple() {
        // encode some data
        let uint1 = 5;
        let mut bs = vec![uint1];
        let str = "olavm".to_string();
        let source = str
            .as_bytes()
            .into_iter()
            .map(|x| *x as usize)
            .collect::<Vec<usize>>();
        bs.resize(2, 0);
        bs[1] = source.len() as usize;
        bs.extend_from_slice(&source);
        let addr = [1, 2, 3, 4];
        bs.extend_from_slice(&addr);

        let v = Value::decode_from_slice(
            &bs,
            &[Type::Tuple(vec![
                ("a".to_string(), Type::U32),
                ("b".to_string(), Type::String),
                ("c".to_string(), Type::Address),
            ])],
        )
        .expect("decode_from_slice failed");

        assert_eq!(
            v,
            vec![Value::Tuple(vec![
                ("a".to_string(), Value::U32(uint1)),
                ("b".to_string(), Value::String(str)),
                ("c".to_string(), Value::Address(addr))
            ])]
        );
    }

    #[test]
    fn decode_many() {
        // fn f(string x, u32 y, u32[][2]  z)
        let tys = vec![
            Type::String,
            Type::U32,
            Type::FixedArray(Box::new(Type::Array(Box::new(Type::U32))), 2),
        ];

        // f("olavm", 12, [[1, 2], [3]])
        let bs = vec![5, 111, 108, 97, 118, 109, 12, 2, 1, 2, 1, 3];

        let v = Value::decode_from_slice(&bs, &tys).expect("decode_from_slice failed");

        assert_eq!(
            v,
            vec![
                Value::String("olavm".to_string()),
                Value::U32(12),
                Value::FixedArray(
                    vec![
                        Value::Array(vec![Value::U32(1), Value::U32(2),], Type::U32),
                        Value::Array(vec![Value::U32(3)], Type::U32),
                    ],
                    Type::Array(Box::new(Type::U32))
                ),
            ],
        );
    }

    #[test]
    fn encode_uint() {
        let value = Value::U32(12);

        let expected_bytes = vec![12];

        assert_eq!(Value::encode(&[value]), expected_bytes);
    }

    #[test]
    fn encode_address() {
        let addr = [1, 2, 3, 4];
        let value = Value::Address(addr);

        let expected_bytes = vec![1, 2, 3, 4];

        assert_eq!(Value::encode(&[value]), expected_bytes);
    }

    #[test]
    fn encode_hash() {
        let addr = [1, 2, 3, 4];
        let value = Value::Address(addr);

        let expected_bytes = vec![1, 2, 3, 4];

        assert_eq!(Value::encode(&[value]), expected_bytes);
    }

    #[test]
    fn encode_bool() {
        let true_vec = vec![1];

        let false_vec = vec![0];

        assert_eq!(Value::encode(&[Value::Bool(true)]), true_vec);
        assert_eq!(Value::encode(&[Value::Bool(false)]), false_vec);
    }

    #[test]
    fn encode_fixed_array() {
        let uint1 = 57;
        let uint2 = 108;

        let value = Value::FixedArray(vec![Value::U32(uint1), Value::U32(uint2)], Type::U32);

        let expected_bytes = [57, 108];

        assert_eq!(Value::encode(&[value]), expected_bytes);
    }

    #[test]
    fn encode_string_and_fields() {
        // Bytes and strings are encoded in the same way.

        let expected_bytes = [5, 111, 108, 97, 118, 109];
        assert_eq!(
            Value::encode(&[Value::String("olavm".to_string())]),
            expected_bytes
        );
    }

    #[test]
    fn encode_array() {
        let addr1 = [1, 2, 3, 4];
        let addr2 = [5, 6, 7, 8];

        let value = Value::Array(
            vec![Value::Address(addr1), Value::Address(addr2)],
            Type::Address,
        );

        let expected_bytes = [2, 1, 2, 3, 4, 5, 6, 7, 8];

        assert_eq!(Value::encode(&[value]), expected_bytes);
    }

    #[test]
    fn encode_fixed_tuple() {
        let addr = [1, 2, 3, 4];

        let value = Value::Tuple(vec![
            ("a".to_string(), Value::Address(addr)),
            ("b".to_string(), Value::U32(99)),
        ]);

        let expected_bytes = [1, 2, 3, 4, 99];

        assert_eq!(Value::encode(&[value]), expected_bytes);
    }

    #[test]
    fn encode_tuple() {
        let s = "olavm".to_string();

        let value = Value::Tuple(vec![
            ("a".to_string(), Value::String(s.clone())),
            ("b".to_string(), Value::U32(99)),
        ]);

        let expected_bytes = [5, 111, 108, 97, 118, 109, 99];

        assert_eq!(Value::encode(&[value]), expected_bytes);
    }

    #[test]
    fn encode_many() {
        let values = vec![
            Value::String("olavm".to_string()),
            Value::U32(99),
            Value::FixedArray(
                vec![
                    Value::Array(vec![Value::U32(1), Value::U32(2)], Type::U32),
                    Value::Array(vec![Value::U32(3)], Type::U32),
                ],
                Type::Array(Box::new(Type::U32)),
            ),
        ];

        let expected = [5, 111, 108, 97, 118, 109, 99, 2, 1, 2, 1, 3];
        assert_eq!(Value::encode(&values), expected);
    }
}
