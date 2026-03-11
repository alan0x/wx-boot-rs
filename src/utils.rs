pub mod fs;
pub mod media;
pub mod password;
pub mod validator;

use regex::Regex;
use std::borrow::Cow;
use std::fmt::Write;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use once_cell::sync::Lazy;
use salvo::http::header::CONTENT_DISPOSITION;
use salvo::http::{HeaderValue, Request, Response};
use uuid::Uuid;

use crate::{AppResult, Error};

pub fn clean_string(s: &str) -> String {
    let re = Regex::new(r"[^\p{L}\p{N}]").unwrap();
    re.replace_all(s, "").to_string()
}

pub fn hash_file_md5(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
    let mut file = File::open(path.as_ref())?;
    hash_reader_md5(&mut file)
}
pub fn hash_reader_md5<R: Read>(reader: &mut R) -> Result<String, std::io::Error> {
    let mut ctx = md5::Context::new();
    io::copy(reader, &mut ctx)?;
    Ok(hash_string(&*ctx.compute()))
}
pub fn hash_string(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(result, "{:02X}", b).unwrap();
    }
    result
}

static TURE_VALUES: Lazy<Vec<&'static str>> =
    Lazy::new(|| vec!["true", "1", "yes", "on", "t", "y", "\u{2713}"]);
pub fn str_to_bool(v: &str) -> bool {
    TURE_VALUES.contains(&v)
}

pub fn uuid_string() -> String {
    Uuid::new_v4()
        .as_simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}

pub fn add_serve_file_content_disposition(
    res: &mut Response,
    file_path: impl AsRef<Path>,
    disposition_type: Option<&str>,
    attached_name: Option<&str>,
) -> AppResult<()> {
    let content_type = mime_guess::from_path(file_path.as_ref()).first_or_octet_stream();
    let disposition_type = disposition_type.unwrap_or_else(|| {
        if attached_name.is_some() {
            "attachment"
        } else {
            match (content_type.type_(), content_type.subtype()) {
                (mime::IMAGE | mime::TEXT | mime::VIDEO | mime::AUDIO, _)
                | (_, mime::JAVASCRIPT | mime::JSON) => "inline",
                _ => "attachment",
            }
        }
    });
    let content_disposition = if disposition_type == "attachment" {
        let attached_name = match attached_name {
            Some(attached_name) => Cow::Borrowed(attached_name),
            None => file_path
                .as_ref()
                .file_name()
                .map(|file_name| file_name.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".into())
                .into(),
        };
        format!("attachment; filename={}", attached_name)
            .parse::<HeaderValue>()
            .map_err(|_| Error::Internal("failed to parse http header value".into()))?
    } else {
        disposition_type
            .parse::<HeaderValue>()
            .map_err(|_| Error::Internal("failed to parse http header value".into()))?
    };
    res.headers_mut()
        .insert(CONTENT_DISPOSITION, content_disposition);
    Ok(())
}

pub fn fallbacks_in_query(req: &mut Request) -> Vec<String> {
    if let Some(fbs) = req.queries().get_vec("fb") {
        fbs.iter()
            .filter(|fb| !fb.contains('/') && !fb.contains('\\') && !fb.starts_with('.'))
            .map(|s| s.to_owned())
            .collect()
    } else {
        vec![]
    }
}
