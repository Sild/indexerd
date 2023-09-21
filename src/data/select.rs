use crate::data::updater::Updater;
use mysql::*;
use std::error::Error;
use std::sync::{Arc, RwLock};

pub fn init(_updater: &mut Updater) -> Result<(), Box<dyn Error>> {
    Ok(())
}
