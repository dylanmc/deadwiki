use deadwiki::{app, db, sync};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut args = args.iter();
    let mut path = "";
    let mut host = "0.0.0.0";
    let mut port = 8000;
    let mut sync = false;

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-v" | "-version" | "--version" => return print_version(),
            "-h" | "-help" | "--help" => return print_help(),
            "-s" | "-sync" | "--sync" => sync = true,
            "-H" | "-host" | "--host" => {
                if let Some(arg) = args.next() {
                    host = arg;
                } else {
                    return eprintln!("--host needs a value");
                }
            }
            "-p" | "-port" | "--port" => {
                if let Some(arg) = args.next() {
                    port = arg.parse().unwrap();
                } else {
                    return eprintln!("--port needs a value");
                }
            }
            _ => {
                if arg.starts_with('-') {
                    return eprintln!("unknown option: {}", arg);
                }
                path = arg;
            }
        }
    }

    println!("~> deadwiki v{}", env!("CARGO_PKG_VERSION"));

    if path.is_empty() {
        return print_help();
    }

    // ~ -> $HOME
    let path = if path.contains('~') {
        path.replace('~', &std::env::var("HOME").unwrap())
    } else {
        path.into()
    };

    let path = if !path.ends_with('/') {
        format!("{}/", path)
    } else {
        path
    };

    if sync {
        if let Err(e) = sync::start(&path) {
            eprintln!("Sync Error: {}", e);
            return;
        }
    }

    let db = db::DB::new(path);
    vial::use_state!(db);
    if let Err(e) = vial::run_with_banner!("~> started at {}", format!("{}:{}", host, port), app) {
        eprintln!("WebServer Error: {}", e);
    }
}

fn print_version() {
    println!("deadwiki v{}", env!("CARGO_PKG_VERSION"))
}

fn print_help() {
    print!(
        "Usage: dead [options] <PATH TO WIKI>

Options:
    -H, --host     Host to bind to. Default: 0.0.0.0
    -p, --port     Port to bind to. Default: 8000
    -s, --sync     Automatically sync wiki. Must be a git repo.
    -v, --version  Print version.
    -h, --help     Show this message.
",
    );
}
