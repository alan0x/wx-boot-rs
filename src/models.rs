mod base;

pub mod help_ticket;
pub mod order;

pub use base::*;

pub static ID_NAME_SEARCH_TMPL: &str = "id::varchar(255)='{{data}}' or name ilike E'%{{data}}%'";
pub static ID_SUBJECT_SEARCH_TMPL: &str =
    "id::varchar(255)='{{data}}' or subject ilike E'%{{data}}%'";
