use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use rand::prelude::*;
use clap::{Command, Arg};
use rand::thread_rng;

const DICKS: &[&str] = &[
    "8=D ", "8=D~ ", "8=D~~ ", "8=D~~~ ", "8==D ", "8==D~ ", "8==D~~ ", "8==D~~~ ", "8===D ", "8===D~ ", "8===D~~ ", "8===D~~~ ", "8====D ", "8====D~ ", "8====D~~ ", "8====D~~~ ", "8=====D ", "8=====D~ ", "8=====D~~ ", "8=====D~~~ ", "8======D ", "8======D~ ", "8======D~~ ", "8======D~~~ ", "8=======D ", "8=======D~ ", "8=======D~~ ", "8=======D~~~ ", "8========D ", "8========D~ ", "8========D~~ ", "8========D~~~ ", "8=========D ", "8=========D~ ", "8=========D~~ ", "8=========D~~~ ", "8==========D ", "8==========D~ ", "8==========D~~ ", "8==========D~~~ ", "8===========D ", "8===========D~ ", "8===========D~~ ", "8===========D~~~ ", "8============D ", "8============D~ ", "8============D~~ ", "8============D~~~ ", "8#=D ", "8#=D~ ", "8#=D~~ ", "8#=D~~~ ", "8#==D ", "8#==D~ ", "8#==D~~ ", "8#==D~~~ ", "8#===D ", "8#===D~ ", "8#===D~~ ", "8#===D~~~ ", "8#====D ", "8#====D~ ", "8#====D~~ ", "8#====D~~~ ", "8#=====D ", "8#=====D~ ", "8#=====D~~ ", "8#=====D~~~ ", "8#======D ", "8#======D~ ", "8#======D~~ ", "8#======D~~~ ", "8#=======D ", "8#=======D~ ", "8#=======D~~ ", "8#=======D~~~ ", "8#========D ", "8#========D~ ", "8#========D~~ ", "8#========D~~~ ", "8#=========D ", "8#=========D~ ", "8#=========D~~ ", "8#=========D~~~ ", "8#==========D ", "8#==========D~ ", "8#==========D~~ ", "8#==========D~~~ ", "8#===========D ", "8#===========D~ ", "8#===========D~~ ", "8#===========D~~~ ", "8#============D ", "8#============D~ ", "8#============D~~ ", "8#============D~~~ ", "𓂺 ",
];

fn generate_dicks() -> Vec<String> {
    let mut dicks = Vec::new();
    for a in 0..2 {
        for b in 1..13 {
            for c in 0..4 {
                let dick = format!("8{}{}D{} ", "#".repeat(a), "=".repeat(b), "~".repeat(c));
                dicks.push(dick);
            }
        }
    }
    dicks
}

fn rand_dick(rng: &mut ThreadRng) -> &'static str {
    let index = rng.gen_range(0..DICKS.len());
    DICKS[index]
}

fn fast_rand_dick<'a>(cache: &'a mut String, count: &mut usize, rng: &mut ThreadRng) -> &'a str {
    if cache.is_empty() || *count == 0 {
        *cache = String::new();
        *count = rng.gen_range(1000..10000);
        for _ in 0..rng.gen_range(150..300) {
            cache.push_str(rand_dick(rng));
        }
    }
    *count -= 1;
    cache
}

fn wipe(dev: &Path, rounds: usize, rng: &mut ThreadRng) -> io::Result<()> {
    let size = fs::metadata(dev).map(|m| m.len()).unwrap_or(0);

    for _ in 0..rounds {
        let mut file = OpenOptions::new().write(true).open(dev)?;

        if size == 0 {
            loop {
                let dick = rand_dick(rng);
                if file.write_all(dick.as_bytes()).is_err() {
                    break;
                }
            }
        } else {
            let mut dlen = 0;
            while dlen < size {
                let dick = rand_dick(rng);
                dlen += dick.len() as u64;
                if file.write_all(dick.as_bytes()).is_err() {
                    break;
                }
            }
        }
    }

    fs::remove_file(dev)?;

    Ok(())
}

fn parse_dir(dir: &Path, recursive: bool) -> io::Result<Vec<PathBuf>> {
    let mut filelist = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?; 
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                filelist.extend(parse_dir(&path, recursive)?);
            }
        } else {
            filelist.push(path);
        }
    }
    Ok(filelist)
}

fn parse_filelist(filelist: &[PathBuf], recursive: bool) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for item in filelist {
        if item.is_dir() {
            if recursive {
                files.extend(parse_dir(item, recursive)?);
            } else {
                eprintln!("WARNING: {:?} is a directory and recursive is off.", item)
            }
        } else if item.exists() {
            files.push(item.to_path_buf());
        }
    }
    Ok(files)
}

fn main() {
    let matches = Command::new("Wipe files/devices with dicks")
        .version("0.0.1")
        .author("vxfemboy")
        .arg(Arg::new("recursive")
            .short('r')
            .long("recursive")
            .help("Recursively wipe directories")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(Arg::new("numrounds")
            .short('n')
            .long("numrounds")
            .help("The number of rounds to wipe the file/device")
            .value_parser(clap::value_parser!(usize))
            .default_value("1")
        )
        .arg(Arg::new("wipefree")
            .short('w')
            .long("wipefree")
            .help("Wipe free space on device")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(Arg::new("slow")
            .short('s')
            .long("slow")
            .help("Use more randomness, tends to be slower")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(Arg::new("files")
            .help("Files or directories to wipe")
            .num_args(1..)
            .required(true)
        )
        .get_matches();

    let recursive = matches.get_flag("recursive");
    let numrounds: usize = *matches.get_one("numrounds").unwrap();
    let slow = matches.get_flag("slow");
    let wipefree = matches.get_flag("wipefree");

    let initial_file_list: Vec<PathBuf> = matches.get_many::<String>("files").unwrap().map(|s| PathBuf::from(s)).collect();

    let file_list = parse_filelist(&initial_file_list, recursive).unwrap(); 
 
    let file_list = if wipefree {
        let mut fl = file_list;
        fl.push(PathBuf::from("dick.tmp"));
        fl
    } else {
        file_list
    };

    let mut handles = Vec::new();

    for f in file_list {
        let handle = thread::spawn(move || {
            let mut rng = thread_rng(); // create new threads uwu 
            if let Err(e) = wipe(&f, numrounds, &mut rng) {
                eprintln!("ERROR: {:?}: {:?}", f, e);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

