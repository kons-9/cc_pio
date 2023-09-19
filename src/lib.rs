use cc;
use serde_json::{self, Value};
use std::path::PathBuf;
use std::process::Command;

pub trait PioExtention {
    fn pio(&mut self, pio_path: Option<PathBuf>, cpp: bool) -> &mut Self;
    fn search_pio(pio_path: Option<PathBuf>) -> Result<Command, std::io::Error> {
        let mut command = if let Some(pio_path) = pio_path {
            Command::new(pio_path)
        } else {
            println!("searching pio in default path");
            Command::new("pio")
        };
        command.status()?;
        Ok(command)
    }
}

impl PioExtention for cc::Build {
    fn pio(&mut self, pio_path: Option<PathBuf>, cpp: bool) -> &mut Self {
        // get pio command
        let mut command = Self::search_pio(pio_path).unwrap();

        // get json
        let output = command
            .arg("project")
            .arg("metadata")
            .arg("--json-output")
            .output()
            .expect("failed to execute process")
            .stdout;
        let stdout = String::from_utf8(output).unwrap();
        let std_json: Value = serde_json::from_str(&stdout).unwrap();

        // get computer name
        let micro_com_name = match &std_json {
            Value::Object(obj) => {
                // get first key
                obj.keys().next().unwrap()
            }
            _ => {
                panic!("invalid platformio.ini");
            }
        };
        println!("target computer's name: {}", micro_com_name);

        // get metadata
        let _build_type = &std_json[micro_com_name]["build_type"];
        let _env_name = &std_json[micro_com_name]["env_name"];
        let libsource_dirs = &std_json[micro_com_name]["libsource_dirs"];
        let defines = &std_json[micro_com_name]["defines"];
        let includes = &std_json[micro_com_name]["includes"];
        let cc_flags = &std_json[micro_com_name]["cc_flags"];
        let cxx_flags = &std_json[micro_com_name]["cxx_flags"];
        let cc_path = &std_json[micro_com_name]["cc_path"];
        let cxx_path = &std_json[micro_com_name]["cxx_path"];

        // add libsource_dirs
        for libsource_dir in libsource_dirs.as_array().unwrap() {
            self.flag_if_supported(&("-L".to_string() + libsource_dir.as_str().unwrap()));
        }
        // add defines
        for define in defines.as_array().unwrap() {
            let define = define.as_str().unwrap();
            if define.find("=").is_some() {
                self.define(
                    define.split("=").next().unwrap(),
                    Some(define.split("=").last().unwrap()),
                );
            } else {
                self.define(define, None);
            }
        }
        // add includes
        for include in includes.as_object().unwrap().keys() {
            for include_dir in includes[include].as_array().unwrap() {
                self.include(include_dir.as_str().unwrap());
            }
        }

        if cpp {
            // add cxx_flags
            for cxx_flag in cxx_flags.as_array().unwrap() {
                self.flag(cxx_flag.as_str().unwrap());
            }
            // add cxx_path
            self.compiler(cxx_path.as_str().unwrap());
        } else {
            // add cc_flags
            for cc_flag in cc_flags.as_array().unwrap() {
                self.flag(cc_flag.as_str().unwrap());
            }
            // add cc_path
            self.compiler(cc_path.as_str().unwrap());
        }

        self
    }
}
