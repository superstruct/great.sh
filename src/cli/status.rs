use anyhow::Result;
use clap::Args as ClapArgs;

use crate::platform;

#[derive(ClapArgs)]
pub struct Args {
    /// Show detailed status information
    #[arg(long, short)]
    pub verbose: bool,

    /// Output status as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let info = platform::detect_platform_info();

    if args.json {
        println!(
            r#"{{"platform": "{}", "arch": "{:?}", "shell": "{}"}}"#,
            info.platform,
            info.platform.arch(),
            info.shell
        );
    } else {
        println!("great status");
        println!("  platform: {}", info.platform.display_detailed());
        if args.verbose {
            println!("  capabilities:");
            let caps = &info.capabilities;
            if caps.has_homebrew {
                println!("    homebrew: installed");
            }
            if caps.has_apt {
                println!("    apt: installed");
            }
            if caps.has_dnf {
                println!("    dnf: installed");
            }
            if caps.has_snap {
                println!("    snap: installed");
            }
            if caps.has_systemd {
                println!("    systemd: active");
            }
            if caps.has_docker {
                println!("    docker: installed");
            }
            println!("  shell: {}", info.shell);
            println!("  root: {}", info.is_root);
        }
    }
    Ok(())
}
