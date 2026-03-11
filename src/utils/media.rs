use std::path::Path;

use tokio::process::Command;

use super::fs::{get_file_ext /* get_parent_dir */};
use crate::{AppResult, Error};

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageInfo {
    pub format: String,
    pub codec_name: String,
    pub codec_long_name: String,
    pub codec_type: String,
    pub codec_tag: String,
    pub width: u64,
    pub height: u64,
    #[serde(default)]
    pub byte_size: u64,
}
#[derive(Serialize, Deserialize, Debug)]
struct ImageStream {
    codec_name: String,
    codec_long_name: String,
    codec_type: String,
    codec_tag: String,
    width: u64,
    height: u64,
}
impl ImageStream {
    fn into_info(self, format: String) -> ImageInfo {
        let ImageStream {
            codec_name,
            codec_long_name,
            codec_type,
            codec_tag,
            width,
            height,
            ..
        } = self;
        ImageInfo {
            format,
            codec_name,
            codec_long_name,
            codec_type,
            codec_tag,
            width,
            height,
            byte_size: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FileFormat {
    size: String,
    #[serde(default)]
    duration: String,
}
pub async fn get_image_info<P: AsRef<Path>>(file: P) -> AppResult<ImageInfo> {
    #[derive(Serialize, Deserialize, Debug)]
    struct FfprobeOutput {
        streams: Vec<ImageStream>,
        format: FileFormat,
    }
    let mut cmd = Command::new("ffprobe");
    cmd.kill_on_drop(true)
        .args("-v quiet -print_format json -show_format -show_streams".split(' '))
        .arg(file.as_ref());
    tracing::info!("get image info: {:?}", cmd);

    let output = cmd.output().await?;
    let mut output = serde_json::from_slice::<FfprobeOutput>(&output.stdout)?;
    if !output.streams.is_empty() {
        let mut image_info: ImageInfo = output.streams.remove(0).into_info(get_file_ext(file));
        image_info.byte_size = output.format.size.parse::<u64>().unwrap_or_default();
        Ok(image_info)
    } else {
        Err(crate::Error::Public("get image info failed".into()))
    }
}

pub async fn resize_image<S, P>(
    width: Option<u64>,
    height: Option<u64>,
    origin_file: S,
    dest_file: P,
) -> AppResult<()>
where
    S: AsRef<Path>,
    P: AsRef<Path>,
{
    let info = match get_image_info(&origin_file).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!(origin_file = ?origin_file.as_ref(), "resize_image: get_image_info failed");
            return Err(e);
        }
    };
    let mut cmd = Command::new("ffmpeg");
    cmd.kill_on_drop(true)
        .arg("-y")
        .arg("-i")
        .arg(origin_file.as_ref());
    let filter = if let (Some(width), Some(height)) = (width, height) {
        if (width as f64 / height as f64) >= (info.width as f64 / info.height as f64) {
            Some(format!(
                "scale={}:-2,crop={}:{}:exact=1",
                width, width, height
            ))
        } else {
            Some(format!(
                "scale=-2:{},crop={}:{}:exact=1",
                height, width, height
            ))
        }
    } else if let Some(width) = width {
        if width > info.width {
            return Err(Error::Public(format!(
                "resized width {} is larger than image width {}",
                width, info.width
            )));
        }
        Some(format!("scale={}:-2", width))
    } else if let Some(height) = height {
        if height > info.height {
            return Err(Error::Public(format!(
                "resized height {} is larger than image height {}",
                height, info.height
            )));
        }
        Some(format!("scale=-2:{}", height))
    } else {
        None
    };
    if let Some(filter) = filter {
        cmd.arg("-filter:v").arg(filter);
    }
    cmd.arg(dest_file.as_ref());

    println!("resize image, cmd: ${:?}", cmd);
    let mut child = cmd.spawn()?;
    if !child.wait().await?.success() {
        return Err(Error::Public("resize image failed 5".into()));
    }
    Ok(())
}
