extern crate rustc_serialize;
extern crate docopt;
extern crate uuid;

use docopt::Docopt;
use std::fs;
use std::path::Path;
use rustc_serialize::json;
use std::io::prelude::*;
use std::fs::File;
use std::io::Result;
use std::process::exit;
use std::env;
use uuid::Uuid;
use std::process::Command;

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

fn read_file(file_name: &str) -> Result<String> {
    let mut f = try!(File::open(file_name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    return Ok(s);
}

fn write_file<P: AsRef<Path>>(file_path: &P, contents: &[u8]) -> Result<()> {
    let mut f = try!(File::create(file_path));
    try!(f.write_all(contents));
    Ok(())
}

fn get_all_environments() -> Vec<Environment> {
    let env_dir = get_store_path("environments");
    let paths = fs::read_dir(env_dir.as_path()).unwrap();
    let mut environments: Vec<Environment> = Vec::new();
    for path in paths {
        let file_contents = read_file(path.unwrap().path().to_str().unwrap()).unwrap();
        let decoded: Environment = json::decode(&file_contents).unwrap();
        environments.push(decoded);
    }
    return environments;
}

fn get_environment(name: &str) -> Option<Environment> {
    let file_name = format!("{}.json", name);
    let env_file = get_env_file_path(&file_name);
    let file_contents = match read_file(env_file.to_str().unwrap()) {
        Ok(c) => {c},
        Err(e) => {return None}
    };
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
            key: read_file(&args.flag_key_file).unwrap()
        };
        let encoded = json::encode(&e).unwrap();

        //writing to the File
        let file_name = format!("{}.json", e.name);
        let file_path = get_env_file_path(&file_name);
        let mut f = File::create(file_path.to_str().unwrap()).unwrap();
        f.write_all(&(encoded.into_bytes())[..]);
    }
}

fn ssh_into(env: &Environment) {
    // storing the key in a temporary file so that can be passed to SSH command
    let mut pem_file = env::temp_dir();
    pem_file.push(format!("{}.pem", Uuid::new_v4().to_string()));
    match write_file(&pem_file, &(env.key.clone().into_bytes()[..])) {
        Ok(_) => {},
        Err(e) => { println!("Failed to write pem file"); exit(2); }
    }

    println!("ssh -i {} {}@{}", pem_file.to_str().unwrap(), env.sshuser, env.ip);
    let mut ssh = Command::new("ssh")
                        .arg("-i")
                        .arg(pem_file)
                        .arg(format!("{}@{}", env.sshuser, env.ip))
                        .spawn()
                        .unwrap_or_else(|e| { panic!("failed to execute ssh: {}", e) });
    let ecode = ssh.wait().unwrap_or_else(|e| { panic!("failed to wait on child: {}", e) });
    if !ecode.success() {
        println!("SSH exited with a non-zero exit code: {}", ecode);
        exit(2);
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
            Some(e) => ssh_into(&e),
            None => {println!("{} is not a valid environment", name); exit(2);},
        }
    } else {
        println!("{}", VERSION);
    }
}
