use log::{error, log};
use mysql_cdc::events::row_events::mysql_value::MySqlValue;
use std::ops::Deref;
use std::panic;

pub trait FromSqlValue {
    fn from(value: &MySqlValue) -> Self;
}

pub fn convert<T: FromSqlValue + std::default::Default>(val: &Option<MySqlValue>) -> T {
    let mysql_val = val.as_ref().unwrap();
    match panic::catch_unwind(|| T::from(&mysql_val)) {
        Ok(value) => value,
        Err(e) => {
            log::error!(
                "Fail to convert {:?} to {}: {:?}",
                mysql_val,
                std::any::type_name::<T>(),
                e
            );
            T::default()
        }
    }
}

impl FromSqlValue for i8 {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::TinyInt(v) => *v as i8,
            _ => panic!("Cannot convert MySqlValue to i8"),
        }
    }
}

impl FromSqlValue for i16 {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::SmallInt(v) => *v as i16,
            _ => panic!("Cannot convert MySqlValue to i16"),
        }
    }
}

impl FromSqlValue for i32 {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::Int(v) => *v as i32,
            _ => panic!("Cannot convert MySqlValue to i32"),
        }
    }
}

impl FromSqlValue for i64 {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::Int(v) => *v as i64,
            _ => panic!("Cannot convert MySqlValue to i64"),
        }
    }
}

impl FromSqlValue for u8 {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::TinyInt(v) => *v,
            _ => panic!("Cannot convert MySqlValue to u8"),
        }
    }
}

// Implementations for u16, u32, u64, usize

impl FromSqlValue for f32 {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::Float(v) => *v as f32,
            _ => panic!("Cannot convert MySqlValue to f32"),
        }
    }
}

// Implementation for f64

impl FromSqlValue for bool {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::TinyInt(v) => *v != 0,
            _ => panic!("Cannot convert MySqlValue to bool"),
        }
    }
}

impl FromSqlValue for String {
    fn from(value: &MySqlValue) -> Self {
        match value {
            MySqlValue::String(v) => v.to_string(),
            _ => panic!("Cannot convert MySqlValue to String"),
        }
    }
}
