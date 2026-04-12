use colored::Colorize;
use std::env;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use minify_html::{Cfg, minify as html_minify};
use minify_js::{Session, TopLevelMode, minify as js_minify};

use crate::{SophiAction, SophiBase, util::loading::LoadingBar, util::timer::SophiTimer};

pub struct SophiBuild {
    pub command: SophiBase,
}

#[derive(PartialEq)]
enum BuildEnv {
    DEV,
    TEST,
    PROD,
}

fn build_env_display(build_env: &BuildEnv) -> &str {
    if build_env == &BuildEnv::PROD {
        return "Production";
    } else if build_env == &BuildEnv::TEST {
        return "Test";
    } else {
        return "Development";
    };
}

fn format_bytes_display(bytes: u64) -> String {
    let kilobytes: f64 = bytes as f64 / 1024.00;
    let megabytes: f64 = kilobytes / 1024.00;
    let gigabytes: f64 = megabytes / 1024.00;

    if bytes <= 1024 {
        format!("{} bytes", bytes)
    } else if bytes <= 1024 * 1024 {
        format!("{:.2} KB", kilobytes)
    } else if bytes <= 1024 * 1024 * 1024 {
        format!("{:.2} MB", megabytes)
    } else {
        format!("{:.2} GB", gigabytes)
    }
}

impl SophiBuild {
    pub fn valid_args() -> Vec<String> {
        vec![
            "-c".to_string(),
            "--client".to_string(),
            "-s".to_string(),
            "--server".to_string(),
            "-a".to_string(),
            "--all".to_string(),
            "-d".to_string(),
            "--develop".to_string(),
            "-t".to_string(),
            "--test".to_string(),
            "-p".to_string(),
            "--production".to_string(),

            "-m".to_string(),
            "--minify".to_string()
        ]
    }

    fn count_dir_files(path: &str, sub_dirs: bool) -> usize {
        let mut count: usize = 0;
        if let Ok(dir) = fs::read_dir(path) {
            for entry in dir {
                let val: DirEntry = entry.unwrap();
                let route = val.path();

                if val.metadata().unwrap().is_dir() && sub_dirs {
                    count += SophiBuild::count_dir_files(route.to_str().unwrap(), sub_dirs);
                } else {
                    count += 1;
                }
            }
        }

        return count;
    }

    fn read_dir_file_content<F>(
        path: &str,
        sub_dirs: bool,
        ldr: &mut LoadingBar,
        wrapper: &F,
    ) -> String
    where
        F: Fn(&str, &str) -> String,
    {
        let dir = fs::read_dir(path);
        let mut content: String = String::new();

        match dir {
            Ok(paths) => {
                for entry in paths {
                    let val: DirEntry = entry.unwrap();
                    let route = val.path();

                    if val.metadata().unwrap().is_dir() {
                        if sub_dirs {
                            content.push_str(&SophiBuild::read_dir_file_content(
                                route.to_str().unwrap(),
                                sub_dirs,
                                ldr,
                                wrapper,
                            ));
                            content.push_str("\n");
                        }
                    } else {
                        ldr.load(
                            None,
                            &format!(
                                "Processing: {} ({} bytes)",
                                route.display().to_string().bright_black(),
                                val.metadata().unwrap().len()
                            ),
                        );
                        content.push_str(&wrapper(
                            route.file_stem().unwrap().to_str().unwrap(),
                            &fs::read_to_string(&route).unwrap(),
                        ));
                        content.push_str("\n");
                    }
                }

                return content;
            }
            Err(_) => String::new(),
        }
    }

    fn client_build(build_env: &BuildEnv, is_verbose: bool, is_minify: bool) {
        let has_client = Path::new("./client").exists();
        if !has_client {
            println!("{}", "No client folder to build.".yellow());
            return;
        }

        let mut timer = SophiTimer::new();

        let is_prod = build_env == &BuildEnv::PROD;

        const BUILD: &str = "./build/index.html";

        const MAIN: &str = "./client/main/index.html";
        const APP: &str = "./client/main/app.js";
        const INDEX: &str = "./client/main/index.js";
        const JS_DIR: &str = "./client/js";
        const CSS_DIR: &str = "./client/css";
        const COMPONENTS_DIR: &str = "./client/components";
        const WORKERS_DIR: &str = "./client/workers";
        const SHARED_DIR: &str = "./shared";

        let mut total_files: usize = [JS_DIR, CSS_DIR, COMPONENTS_DIR, WORKERS_DIR, SHARED_DIR]
            .map(|d| SophiBuild::count_dir_files(&d, true))
            .iter()
            .sum();
        total_files += 2;

        let mut loading = LoadingBar::new();
        loading.reset(total_files);
        loading.verbose = is_verbose;

        let mut dist: String =
            fs::read_to_string(MAIN).expect("There must be a ./client/main/index.html file");

        // Get the Vue source code files
        dist = dist.replace("{{# vue }}", if is_prod {
        "<script src=\"https://unpkg.com/vue@3.5.9/dist/vue.global.prod.js\"></script>\n<script src=\"https://unpkg.com/vue-router@4.4.5/dist/vue-router.global.prod.js\"></script>"
    } else {
        "<script src=\"https://unpkg.com/vue@3\"></script>\n<script src=\"https://unpkg.com/vue-router@4\"></script>"
    });

        let app_file = fs::metadata(APP);
        if app_file.is_err() {
            loading.load(None, &format!("No file: {}", APP.bright_black()));
            dist = dist.replace("{{# app }}", "");
        } else {
            loading.load(
                None,
                &format!(
                    "Processing: {} ({} bytes)",
                    APP.bright_black(),
                    app_file.unwrap().len()
                ),
            );
            let app: String = fs::read_to_string(APP).unwrap();
            dist = dist.replace("{{# app }}", &format!("<script>\n{}\n</script>", app));
        }

        let index_file = fs::metadata(INDEX);

        if index_file.is_err() {
            loading.load(None, &format!("No file: {}", INDEX.bright_black()));
            dist = dist.replace("{{# index }}", "");
        } else {
            loading.load(
                None,
                &format!(
                    "Processing: {} ({} bytes)",
                    INDEX.bright_black(),
                    index_file.unwrap().len()
                ),
            );
            let index: String = fs::read_to_string(INDEX).unwrap();
            dist = dist.replace("{{# index }}", &format!("<script>\n{}\n</script>", index));
        }

        let shared: String =
            SophiBuild::read_dir_file_content(SHARED_DIR, true, &mut loading, &|_, content| {
                content.to_string()
            });
        let js: String =
            SophiBuild::read_dir_file_content(JS_DIR, true, &mut loading, &|_, content| {
                content.to_string()
            });
        dist = dist.replace(
            "{{# js }}",
            &format!("<script>\n{}\n{}\n</script>", shared, js),
        );

        let css: String =
            SophiBuild::read_dir_file_content(CSS_DIR, true, &mut loading, &|_, content| {
                content.to_string()
            });
        dist = dist.replace("{{# css }}", &format!("<style>\n{}\n</style>", css));

        let components: String = SophiBuild::read_dir_file_content(
            COMPONENTS_DIR,
            true,
            &mut loading,
            &|file, content| {
                content.replace(
                    "export default {",
                    &format!(
                        "const {} = {{",
                        Path::new(file).file_stem().unwrap().to_str().unwrap()
                    ),
                )
            },
        );
        dist = dist.replace("{{# components }}", &components);

        let workers: String = SophiBuild::read_dir_file_content(
            WORKERS_DIR,
            true,
            &mut loading,
            &|name, content| -> String {
                format!(
                    "<script type=\"text/js-worker\" id=\"worker-{}\">\n{}\n</script>",
                    name, content
                )
            },
        );
        dist = dist.replace("{{# workers }}", &workers);

        if is_minify {
            // minify html
            let mut cfg = Cfg::new();
            cfg.preserve_brace_template_syntax = true;
            cfg.minify_js = true;
            cfg.minify_css = true;
            let minified = html_minify(dist.as_bytes(), &cfg);
            dist = String::from_utf8(minified).expect("Failed to get string from minified html");
        }

        let _ = fs::write(BUILD, dist);

        let build_file = fs::metadata(BUILD);
        loading.complete(&format!(
            "Built {} files into {} ({})",
            total_files,
            BUILD,
            format_bytes_display(build_file.unwrap().len())
        ));

        timer.stop().print_line();
        SophiBuild::report_build_finished("Client", BUILD, &build_env);
    }

    fn server_build(build_env: &BuildEnv, is_verbose: bool, is_minify: bool) {
        let has_server = Path::new("./server").exists();
        if !has_server {
            println!("{}", "No server folder to build.".yellow());
            return;
        }

        let mut timer = SophiTimer::new();

        const BUILD: &str = "./build/index.js";

        const CONFIG: &str = "./server/ConfigEnv.js";
        const SERVER: &str = "./server/server.js";

        const SHARED_DIR: &str = "./shared";
        const UTILS_DIR: &str = "./server/utils";
        const LIB_DIR: &str = "./server/lib";
        const INTEGRATIONS_DIR: &str = "./server/integrations";
        const SERVICES_DIR: &str = "./server/services";

        let mut total_files: usize = [
            SHARED_DIR,
            UTILS_DIR,
            LIB_DIR,
            INTEGRATIONS_DIR,
            SERVICES_DIR,
        ]
        .map(|d| SophiBuild::count_dir_files(&d, true))
        .iter()
        .sum();

        let has_config = Path::new(CONFIG).exists();
        let has_server = Path::new(SERVER).exists();
        total_files += has_config as usize + has_server as usize;

        let mut loading = LoadingBar::new();
        loading.reset(total_files);
        loading.verbose = is_verbose;

        let mut dist: String = String::new();

        if has_config {
            let config_file = fs::metadata(CONFIG);
            loading.load(
                None,
                &format!(
                    "Processing: {} ({} bytes)",
                    CONFIG.bright_black(),
                    config_file.unwrap().len()
                ),
            );
            let config = &fs::read_to_string(CONFIG).unwrap().replace(
                "// {{# configurationTemplate }}",
                &format!(
                    "return ConfigurationFactory.{}Config();",
                    build_env_display(&build_env).to_lowercase()
                ),
            );
            dist.push_str(config);
        }

        let shared =
            SophiBuild::read_dir_file_content(SHARED_DIR, true, &mut loading, &|_, content| {
                content.to_string()
            });
        let utils =
            SophiBuild::read_dir_file_content(UTILS_DIR, true, &mut loading, &|_, content| {
                content.to_string()
            });
        let lib = SophiBuild::read_dir_file_content(LIB_DIR, true, &mut loading, &|_, content| {
            content.to_string()
        });
        let integrations = SophiBuild::read_dir_file_content(
            INTEGRATIONS_DIR,
            true,
            &mut loading,
            &|_, content| content.to_string(),
        );
        let services =
            SophiBuild::read_dir_file_content(SERVICES_DIR, true, &mut loading, &|_, content| {
                content.to_string()
            });

        dist.push_str("\n");
        dist.push_str(&shared);
        dist.push_str("\n");
        dist.push_str(&utils);
        dist.push_str("\n");
        dist.push_str(&lib);
        dist.push_str("\n");
        dist.push_str(&integrations);
        dist.push_str("\n");
        dist.push_str(&services);
        dist.push_str("\n");

        if has_server {
            let server_file = fs::metadata(SERVER);
            loading.load(
                None,
                &format!(
                    "Processing: {} ({} bytes)",
                    SERVER.bright_black(),
                    server_file.unwrap().len()
                ),
            );
            let server = fs::read_to_string(SERVER).unwrap();
            dist.push_str(&server);
        }

        if is_minify {
            // minify js
            let session = Session::new();
            let mut minified = Vec::new();
            js_minify(&session, TopLevelMode::Global, dist.as_bytes(), &mut minified).expect("Failed to minimize js!");
            dist = String::from_utf8(minified).expect("Failed to get string from minified js");
        }

        let _ = fs::write(BUILD, dist);

        let build_file = fs::metadata(BUILD);
        loading.complete(&format!(
            "Built {} files into {} ({})",
            total_files,
            BUILD,
            format_bytes_display(build_file.unwrap().len())
        ));

        timer.stop().print_line();
        SophiBuild::report_build_finished("Server", BUILD, &build_env);
    }

    fn report_build_finished(side: &str, loc: &str, build_env: &BuildEnv) {
        let color: &'static str = if build_env == &BuildEnv::PROD {
            "red"
        } else if build_env == &BuildEnv::TEST {
            "yellow"
        } else {
            "green"
        };

        let line: String = format!(
            "[{}] {} built to {}",
            side,
            build_env_display(&build_env),
            loc
        );

        println!("{}", line.color(color));
    }
}

impl SophiAction for SophiBuild {
    fn action(&self) {
        println!(
            "Building directory: {}",
            env::current_dir().unwrap().display().to_string()
        );

        let args = &self.command.args;

        let is_client: bool = args.iter().any(|arg| arg == "--client" || arg == "-c");
        let is_server: bool = args.iter().any(|arg| arg == "--server" || arg == "-s");
        let is_all: bool = args.iter().any(|arg| arg == "--all" || arg == "-a");
        let is_verbose: bool = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
        let is_minify: bool = args.iter().any(|arg| arg == "--minify" || arg == "-m");

        let build_env: BuildEnv = if args.iter().any(|arg| arg == "--test" || arg == "-t") {
            BuildEnv::TEST
        } else if args.iter().any(|arg| arg == "--prod" || arg == "-p") {
            BuildEnv::PROD
        } else {
            BuildEnv::DEV
        };

        if is_client || is_all {
            SophiBuild::client_build(&build_env, is_verbose, is_minify);
        }

        if is_server || is_all {
            SophiBuild::server_build(&build_env, is_verbose, is_minify);
        }
    }
}
