use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bpaf::Bpaf;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use notify_debouncer_full::notify::EventKind;
use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Slide {
    #[serde(default)]
    title: Option<String>,

    #[serde(default)]
    body: Option<String>,

    #[serde(default)]
    notes: Option<String>,

    #[serde(default)]
    media: Option<PathBuf>,

    #[serde(default)]
    layout: Option<Layout>,
}
impl Slide {
    fn render(&self) -> Markup {
        let text = render(&self.body, |text| {
            let md = markdown::to_html(text);
            html! { (PreEscaped(md)) }
        });

        let title = render(&self.title, |text| {
            let md = markdown::to_html(text);
            html! { h1 { (PreEscaped(md)) } }
        });

        let media = render(&self.media, |path| {
            let path = path.display().to_string();
            html! {
                img src=(path);
            }
        });
        html! {
            div {
                (title)
                (media)
                (text)
            }
        }
    }

    fn layout(&self) -> Layout {
        self.layout.unwrap_or(Layout::Centered)
    }

    fn class(&self) -> &'static str {
        self.layout().as_str()
    }

    fn notes(&self) -> &str {
        self.notes.as_deref().unwrap_or("(no notes)")
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
enum Layout {
    #[default]
    Centered,
}
impl Layout {
    fn as_str(&self) -> &'static str {
        match self {
            Layout::Centered => "centered",
        }
    }
}

/// Helper to handle rendering `Option`s.
fn render<T>(opt: &Option<T>, render: impl FnOnce(&T) -> Markup) -> Markup {
    opt.as_ref().map(render).unwrap_or_default()
}

fn render_slides(slides: &[Slide]) -> Markup {
    const STYLE: &str = include_str!("style.css");
    const SCRIPT: &str = include_str!("script.js");
    html! {
        (DOCTYPE)
        head {
            style { (PreEscaped(STYLE)) }
        }
        body {
            @for (i, slide) in slides.iter().enumerate() {
                .slide.(slide.class()) #(i) data-notes=(slide.notes()) { (slide.render()) }
            }
        }

        script type="text/javascript" { (PreEscaped(SCRIPT)) }
    }
}

/// Copy referenced media files to the output assets directory,
/// and update paths accordingly.
fn consolidate_assets(slides: &mut Vec<Slide>, asset_dir: &Path) {
    for slide in slides {
        if let Some(media) = &slide.media {
            let name = media.file_name().unwrap();
            let dest = asset_dir.join(name);
            fs_err::copy(media, dest).expect("unable to copy media file");
            slide.media = Some(PathBuf::from("assets").join(name));
        }
    }
}

fn prepare_output_dir() -> io::Result<(PathBuf, PathBuf)> {
    let outdir = PathBuf::from("slides");
    if outdir.exists() {
        fs_err::remove_dir_all(&outdir)?;
    }
    fs_err::create_dir_all(&outdir)?;

    let assets = outdir.join("assets");
    fs_err::create_dir_all(&assets)?;

    Ok((outdir, assets))
}

fn compile_slides_impl(path: &Path) -> io::Result<()> {
    let data = fs_err::read_to_string(path).expect("unable to read file");
    let mut slides: Vec<Slide> = serde_yaml::from_str(&data)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    let (outdir, assets) = prepare_output_dir()?;
    consolidate_assets(&mut slides, &assets);

    let file = outdir.join("index.html");
    let html = render_slides(&slides);
    fs_err::write(file, html.into_string()).expect("unable to write file");
    Ok(())
}

fn compile_slides(path: &Path) {
    match compile_slides_impl(path) {
        Ok(_) => println!("Slides compiled successfully."),
        Err(err) => println!("Error compiling slides:\n  {err}"),
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// Create an HTML slideshow.
struct Args {
    /// Slides YAML definition to compile.
    #[bpaf(positional("PATH"))]
    path: PathBuf,

    #[bpaf(short, long)]
    watch: bool,
}

fn main() {
    let opts = args().run();
    if opts.watch {
        println!("Watching {:?}", opts.path);

        let path = opts.path.clone();
        compile_slides(&path);
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |res: DebounceEventResult| match res {
                Ok(events) => {
                    if let Some(_) = events.iter().find(|ev| {
                        !matches!(ev.kind, EventKind::Access(_))
                            && ev.paths.iter().any(|p| p.ends_with(&path))
                    }) {
                        compile_slides(&path);
                    }
                }
                Err(e) => println!("Error {:?}", e),
            },
        )
        .unwrap();
        debouncer
            .watch(&opts.path.parent().unwrap(), RecursiveMode::Recursive)
            .unwrap();
        loop {}
    } else {
        compile_slides(&opts.path);
    }
}
