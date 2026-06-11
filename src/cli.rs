use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Options {
    pub recursive: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub preserve: bool,
    pub follow_symlinks: bool,
    pub no_clobber: bool,
    pub force: bool,
    pub parents: bool,
    pub progress: bool,
    pub update: bool,
    pub dry_run: bool,
    pub interactive: bool,
    pub backup: bool,
    pub hard_link: bool,
    pub no_target_dir: bool,
    pub exclude: Vec<String>,
}

pub struct ParsedArgs {
    pub sources: Vec<PathBuf>,
    pub destination: PathBuf,
    pub options: Options,
}

pub fn parse_args() -> ParsedArgs {
    let args: Vec<String> = std::env::args().collect();
    let prog = args.first().map(|s| s.as_str()).unwrap_or("cprust");

    let mut opts = Options::default();
    let mut sources: Vec<PathBuf> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            match arg.splitn(2, '=').collect::<Vec<_>>()[0] {
                "--parents" => opts.parents = true,
                "--progress" => opts.progress = true,
                "--dry-run" => opts.dry_run = true,
                "--version" => {
                    println!("cprust {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "--help" => {
                    print_usage(prog);
                    std::process::exit(0);
                }
                "--exclude" => {
                    if arg.len() > 10 {
                        opts.exclude.push(arg[10..].to_string());
                    } else {
                        eprintln!("{}: --exclude requires a pattern", prog);
                        std::process::exit(1);
                    }
                }
                _ => {
                    eprintln!("{}: unknown option '{}'", prog, arg);
                    print_usage(prog);
                    std::process::exit(1);
                }
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            for ch in arg[1..].chars() {
                match ch {
                    'r' | 'R' => opts.recursive = true,
                    'v' => opts.verbose = true,
                    'q' => opts.quiet = true,
                    'p' => opts.preserve = true,
                    'L' => opts.follow_symlinks = true,
                    'P' => opts.follow_symlinks = false,
                    'n' => opts.no_clobber = true,
                    'f' => opts.force = true,
                    'u' => opts.update = true,
                    'i' => opts.interactive = true,
                    'b' => opts.backup = true,
                    'l' => opts.hard_link = true,
                    'T' => opts.no_target_dir = true,
                    'h' => {
                        print_usage(prog);
                        std::process::exit(0);
                    }
                    _ => {
                        eprintln!("{}: unknown option '-{}'", prog, ch);
                        print_usage(prog);
                        std::process::exit(1);
                    }
                }
            }
        } else {
            sources.push(arg.into());
        }

        i += 1;
    }

    if sources.len() < 2 {
        eprintln!("{}: missing file operand", prog);
        eprintln!();
        print_usage(prog);
        std::process::exit(1);
    }

    let destination = sources.pop().unwrap();
    let sources = sources;

    ParsedArgs {
        sources,
        destination,
        options: opts,
    }
}

fn print_usage(prog: &str) {
    eprintln!("Usage: {} [OPTION]... SOURCE... DESTINATION", prog);
    eprintln!();
    eprintln!("Copy SOURCE(s) to DESTINATION.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -r, -R          copy directories recursively");
    eprintln!("  -v              verbose mode, print each copied file");
    eprintln!("  -q              quiet mode, suppress output");
    eprintln!("  -p              preserve file metadata (timestamps, permissions)");
    eprintln!("  -L              follow symbolic links (copy target, not link)");
    eprintln!("  -P              do not follow symbolic links (copy as symlink, default)");
    eprintln!("  -n              no-clobber, do not overwrite existing files");
    eprintln!("  -f              force, overwrite existing files");
    eprintln!("  -u              update, only copy when source is newer than dest");
    eprintln!("  -i              interactive, prompt before overwrite");
    eprintln!("  -b              backup, create .bak file before overwrite");
    eprintln!("  -l              create hard link instead of copying");
    eprintln!("  -T              no-target-directory, treat destination as a file");
    eprintln!("  --parents       recreate full directory structure");
    eprintln!("  --progress      show progress bar for large files");
    eprintln!("  --dry-run       show what would be copied without copying");
    eprintln!("  --exclude=PAT   exclude files/dirs matching pattern (glob)");
    eprintln!("  --version       show version");
    eprintln!("  -h, --help      show this help message");
}
