use std::process::Stdio;

use log::*;
use tokio::process::{Child, Command};

use crate::utils::{
    config::PlayoutConfig,
    logging::{fmt_cmd, Target},
};
use crate::vec_strings;
use crate::{
    player::{
        controller::ProcessUnit::*,
        utils::{prepare_output_cmd, Media},
    },
    utils::errors::ServiceError,
};

/// Streaming Output
///
/// Prepare the ffmpeg command for streaming output
pub async fn output(config: &PlayoutConfig, log_format: &str) -> Result<Child, ServiceError> {
    let id = config.general.channel_id;
<<<<<<< HEAD
<<<<<<< HEAD
    media.unit = Encoder;
    media.add_filter(config, &None).await;

    let mut enc_prefix = vec_strings!["-hide_banner", "-nostats", "-threads", "4", "-thread_queue_size", "1024", "-hwaccel_device", "0", "-hwaccel", "cuda", "-fix_sub_duration", "-drop_second_field", "true", "-hwaccel_output_format", "cuda","-resize","1280x720", "-v", log_format];
=======
    let mut enc_prefix = vec_strings!["-hide_banner", "-nostats", "-v", log_format];
=======
    //let mut enc_prefix = vec_strings!["-hide_banner", "-nostats", "-v", log_format];
    let mut enc_prefix = vec_strings!["-hide_banner", "-nostats", "-threads", "6", "-thread_queue_size", "1024", "-hwaccel_device", "0", "-hwaccel", "cuda", "-fix_sub_duration", "-drop_second_field", "true", "-hwaccel_output_format", "cuda","-resize","1280x720", "-v", log_format];
>>>>>>> 6f4c6c9c (modificado 27 janeiro)
    let mut media = Media {
        unit: Encoder,
        ..Default::default()
    };
    media.add_filter(config, &None).await;
>>>>>>> c3e971ae (remove is_terminated to use only is_alive, cleanup/shorten coder, longer control throttle)

    if let Some(input_cmd) = &config.advanced.encoder.input_cmd {
        enc_prefix.append(&mut input_cmd.clone());
    }

    enc_prefix.append(&mut vec_strings!["-re", "-i", "pipe:0"]);

    let enc_cmd = prepare_output_cmd(config, enc_prefix, &media.filter);

    debug!(target: Target::file_mail(), channel = id;
        "Encoder CMD: <bright-blue>ffmpeg {}</>",
        fmt_cmd(&enc_cmd)
    );

    let child = Command::new("ffmpeg")
        .args(enc_cmd)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(child)
}
