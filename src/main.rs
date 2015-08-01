extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::fs;
use std::path::Path;
use rustc_serialize::json;
use std::io::prelude::*;
use std::fs::File;

  //naval_fate.py ship <name> move <x> <y> [--speed=<kn>]
  //naval_fate.py ship shoot <x> <y>
  //naval_fate.py mine (set|remove) <x> <y> [--moored | --drifting]
  //naval_fate.py (-h | --help)
  //naval_fate.py --version


static VERSION: &'static str = "0.0.1";
static USAGE: &'static str = "
bootstrap.

Usage:
  bootstrap env new <name> (--ip=<ip> --ssh-user=<ssh_user> --key-file=<key_file>)
  bootstrap env list
  bootstrap ssh <name>
  bootstrap ((-h | --help))
  bootstrap --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_ip: String,
    flag_ssh_user: String,
    flag_key_file: String,
    arg_name: String,
    cmd_env: bool,
    cmd_ssh: bool
}

#[derive(RustcDecodable, RustcEncodable)]
struct Environment {
    name: String,
    ip: String,
    sshuser: String,
    key: String
}

// Creates a new directory in the local storage
fn create_dir_in_store(dir: &str) {
    let store_dir = Path::new(".bootstrap");
    let new_dir = store_dir.join(dir);
    let done = fs::create_dir_all(new_dir);
    if done.is_err() {
        println!("Failed to create directory {}", dir);
    }
}

fn get_store_path(dir: &str) -> std::path::PathBuf {
    let path = Path::new(".bootstrap");
    let dir_path = path.join(dir);
    return dir_path
}

fn get_env_file_path(name: &str) -> std::path::PathBuf {
    let mut env_store_path = get_store_path("environments");
    env_store_path.push(name);
    return env_store_path;
}

fn read_file(file_name: &str) -> String {
    let mut f = File::open(file_name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s);
    return s;
}

fn get_all_environments() -> Vec<Environment> {
    let env_dir = get_store_path("environments");
    let paths = fs::read_dir(env_dir.as_path()).unwrap();
    let mut environments: Vec<Environment> = Vec::new();
    for path in paths {
        let file_contents = read_file(path.unwrap().path().to_str().unwrap());
        let decoded: Environment = json::decode(&file_contents).unwrap();
        environments.push(decoded);
    }
    return environments;
}

fn get_environment(name: &str) -> Option<Environment> {
    let file_name = format!("{}.json", name);
    let env_file = get_env_file_path(&file_name);
    let file_contents = read_file(env_file.to_str().unwrap());
    let decoded: Environment = json::decode(&file_contents).unwrap();
    Some(decoded)
}

fn process_env_command(args: &Args) {
    create_dir_in_store("environments");
    if args.arg_name.is_empty() {
        // just listing the env
        let environments = get_all_environments();
        println!("{}\t{}\t{}", "NAME", "IP", "USER");
        println!("--------------------");
        for env in environments {
            println!("{}\t{}\t{}", env.name, env.ip, env.sshuser);
        }
    } else {
        let e = Environment {
            name: args.arg_name.clone(),
            ip: args.flag_ip.clone(),
            sshuser: args.flag_ssh_user.clone(),
            key: read_file(&args.flag_key_file)
        };
        let encoded = json::encode(&e).unwrap();

        //writing to the File
        let file_name = format!("{}.json", e.name);
        let file_path = get_env_file_path(&file_name);
        let mut f = File::create(file_path.to_str().unwrap()).unwrap();
        f.write_all(&(encoded.into_bytes())[..]);
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.cmd_env {
        process_env_command(&args);
    } else if args.cmd_ssh {
        let name = args.arg_name;
        let env = get_environment(&name);
        match env {
            Some(e) => println!("valid env"),
            None => println!("invalid env"),
        }
        println!("will ssh");
    } else {
        println!("{}", VERSION);
    }
}
