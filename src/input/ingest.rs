use std::{
    io::{Error, Read},
    path::Path,
    process::{Command, Stdio},
    sync::{mpsc::Sender, Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use process_control::{ChildExt, Terminator};
use simplelog::*;
use tokio::runtime::Handle;

use crate::utils::{stderr_reader, GlobalConfig};

fn overlay(config: &GlobalConfig) -> String {
    let mut logo_chain = String::new();

    if config.processing.add_logo && Path::new(&config.processing.logo).is_file() {
        let opacity = format!(
            "format=rgba,colorchannelmixer=aa={}",
            config.processing.logo_opacity
        );
        let logo_loop = "loop=loop=-1:size=1:start=0";
        logo_chain = format!("[v];movie={},{logo_loop},{opacity}", config.processing.logo);

        logo_chain
            .push_str(format!("[l];[v][l]{}:shortest=1", config.processing.logo_filter).as_str());
    }

    logo_chain
}

pub async fn ingest_server(
    log_format: String,
    ingest_sender: Sender<[u8; 65424]>,
    rt_handle: Handle,
    proc_terminator: Arc<Mutex<Option<Terminator>>>,
    is_terminated: Arc<Mutex<bool>>,
) -> Result<(), Error> {
    let config = GlobalConfig::global();
    let mut buffer: [u8; 65424] = [0; 65424];
    let mut filter = format!(
        "[0:v]fps={},scale={}:{},setdar=dar={}",
        config.processing.fps,
        config.processing.width,
        config.processing.height,
        config.processing.aspect
    );

    filter.push_str(&overlay(&config));
    filter.push_str("[vout1]");
    let mut filter_list = vec!["-filter_complex", &filter, "-map", "[vout1]", "-map", "0:a"];

    let mut server_cmd = vec!["-hide_banner", "-nostats", "-v", log_format.as_str()];
    let stream_input = config.ingest.stream_input.clone();
    let stream_settings = config.processing.settings.clone().unwrap();

    server_cmd.append(&mut stream_input.iter().map(String::as_str).collect());
    server_cmd.append(&mut filter_list);
    server_cmd.append(&mut stream_settings.iter().map(String::as_str).collect());

    info!(
        "Start ingest server, listening on: <b><magenta>{}</></b>",
        stream_input.last().unwrap()
    );

    debug!("Server CMD: <bright-blue>{:?}</>", server_cmd);

    loop {
        if *is_terminated.lock().unwrap() {
            break
        }
        let mut server_proc = match Command::new("ffmpeg")
            .args(server_cmd.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Err(e) => {
                error!("couldn't spawn ingest server: {}", e);
                panic!("couldn't spawn ingest server: {}", e)
            }
            Ok(proc) => proc,
        };

        let serv_terminator = server_proc.terminator()?;
        *proc_terminator.lock().unwrap() = Some(serv_terminator);

        rt_handle.spawn(stderr_reader(
            server_proc.stderr.take().unwrap(),
            "Server".to_string(),
            proc_terminator.clone(),
            is_terminated.clone(),
        ));

        let ingest_reader = server_proc.stdout.as_mut().unwrap();

        loop {
            if let Err(e) = ingest_reader.read_exact(&mut buffer[..]) {
                debug!("Ingest server read {:?}", e);
                break;
            };

            if let Err(e) = ingest_sender.send(buffer) {
                error!("Ingest server write error: {:?}", e);

                *is_terminated.lock().unwrap() = true;
                server_proc.kill().expect("Ingest server could not killed");

                break;
            }
        }

        sleep(Duration::from_secs(1));

        if let Err(e) = server_proc.wait() {
            panic!("Ingest server {:?}", e)
        };
    }

    Ok(())
}
