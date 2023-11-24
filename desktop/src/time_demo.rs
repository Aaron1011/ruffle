use crate::cli::Opt;
use anyhow::{anyhow, Context, Error};
use reqwest::blocking::ClientBuilder;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use std::time::Instant;
use url::Url;

fn load_movie(url: &Url, opt: &Opt) -> Result<SwfMovie, Error> {
    let mut movie = if url.scheme() == "file" {
        SwfMovie::from_path(
            url.to_file_path()
                .map_err(|_| anyhow!("Invalid swf path"))?,
            None,
        )
        .map_err(|e| anyhow!(e.to_string()))
        .context("Couldn't load swf")?
    } else {
        let mut builder = ClientBuilder::new();
        if let Some(proxy) = opt.proxy.clone() {
            builder = builder.proxy(reqwest::Proxy::all(proxy)?);
        }
        let client = builder.build().context("Couldn't create HTTP client")?;
        let response = client
            .get(url.to_string())
            .send()
            .with_context(|| format!("Couldn't load URL {url}"))?;
        let bytes = response
            .bytes()
            .context("Couldn't read response from server")?;

        SwfMovie::from_data(&bytes, url.to_string(), None)
            .map_err(|e| anyhow!(e.to_string()))
            .context("Couldn't load swf")?
    };

    movie.append_parameters(opt.parameters());

    Ok(movie)
}

pub fn run_timedemo(mut opt: Opt) -> Result<(), Error> {
    let movie_url = opt
        .movie_url
        .take()
        .ok_or_else(|| anyhow!("Input file necessary for timedemo"))?;
    let movie = load_movie(&movie_url, &opt).context("Couldn't load movie")?;
    let movie_frames = Some(movie.num_frames());

    let viewport_width = 1920;
    let viewport_height = 1080;
    let viewport_scale_factor = 1.0;

    let renderer = WgpuRenderBackend::for_offscreen(
        (viewport_width, viewport_height),
        opt.graphics.into(),
        opt.power.into(),
        opt.trace_path(),
    )
    .map_err(|e| anyhow!(e.to_string()))
    .context("Couldn't create wgpu rendering backend")?;

    let mut builder = PlayerBuilder::new();

    if cfg!(feature = "software_video") {
        builder = builder.with_video(ruffle_video_software::backend::SoftwareVideoBackend::new());
    }

    let player = builder
        .with_renderer(renderer)
        .with_movie(movie)
        .with_viewport_dimensions(viewport_width, viewport_height, viewport_scale_factor)
        .with_autoplay(true)
        .build();

    let mut player_lock = player.lock().expect("Cannot reenter");

    println!("Running {}...", movie_url);

    let start = Instant::now();
    let mut num_frames = 0;
    const MAX_FRAMES: u32 = 5000;
    while num_frames < MAX_FRAMES && player_lock.current_frame() < movie_frames {
        player_lock.run_frame();
        player_lock.render();
        num_frames += 1;
    }
    let end = Instant::now();
    let duration = end.duration_since(start);

    println!("Ran {num_frames} frames in {}s.", duration.as_secs_f32());

    Ok(())
}
