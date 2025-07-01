use std::path::{Path, PathBuf};
use std::time::Duration;

use bpaf::Bpaf;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use serde::{Deserialize, Serialize};

// TODO: Maybe `Slide` itself should be the enum
// with each variant representing a different layout?
#[derive(Debug, Serialize, Deserialize)]
struct Slide {
    #[serde(default)]
    text: Option<String>,

    #[serde(default)]
    notes: Option<String>,

    #[serde(default)]
    media: Option<PathBuf>,
}
impl Slide {
    fn render(&self) -> Markup {
        let text = render(&self.text, |text| {
            let md = markdown::to_html(text);
            html! { (PreEscaped(md)) }
        });
        let media = render(&self.media, |path| {
            let path = path.display().to_string();
            html! {
                img src=(path);
            }
        });
        html! {
            (text)
            (media)
        }
    }

    fn layout(&self) -> Layout {
        if self.media.is_some() {
            Layout::Centered
        } else {
            Layout::Simple
        }
    }

    fn class(&self) -> &'static str {
        self.layout().as_str()
    }

    fn notes(&self) -> &str {
        self.notes.as_deref().unwrap_or("(no notes)")
    }
}

enum Layout {
    Simple,
    Centered,
}
impl Layout {
    fn as_str(&self) -> &'static str {
        match self {
            Layout::Simple => "simple",
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
            style { (STYLE) }
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

fn prepare_output_dir() -> (PathBuf, PathBuf) {
    let outdir = PathBuf::from("slides");
    if outdir.exists() {
        fs_err::remove_dir_all(&outdir).expect("couldn't clean output dir");
    }
    fs_err::create_dir_all(&outdir).expect("couldn't create output dir");

    let assets = outdir.join("assets");
    fs_err::create_dir_all(&assets).expect("couldn't create assets dir");

    (outdir, assets)
}

fn compile_slides(path: &Path) {
    let data = fs_err::read_to_string(path).expect("unable to read file");
    let mut slides: Vec<Slide> = serde_yaml::from_str(&data).unwrap();

    let (outdir, assets) = prepare_output_dir();
    consolidate_assets(&mut slides, &assets);

    let file = outdir.join("index.html");
    let html = render_slides(&slides);
    fs_err::write(file, html.into_string()).expect("unable to write file");
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
        let mut debouncer = new_debouncer(
            Duration::from_secs(1),
            move |res: DebounceEventResult| match res {
                Ok(_) => compile_slides(&path),
                Err(e) => println!("Error {:?}", e),
            },
        )
        .unwrap();
        debouncer
            .watcher()
            .watch(&opts.path, RecursiveMode::Recursive)
            .unwrap();
        loop {}
    } else {
        compile_slides(&opts.path);
    }
}
