use std::error::Error;

pub mod cli;
mod document_storage;
mod http;
pub mod ledger_storage;
mod logger;
mod notification;

pub trait InputAdapter {
    fn run(&mut self) -> Result<(), Box<dyn Error>>;
    fn get_response(&self) -> String;
}
