extern crate clap;

mod os;

use os::*;

use std::io::Write;
use std::path::Path;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;

use clap::{App,Arg,ArgMatches};

type FileHashSet = ::std::collections::HashSet<(u64, u64)>;


macro_rules! stderr(
    ($($arg:tt)*) => { {
        let result = write!(&mut ::std::io::stderr(), $($arg)*);
        result.expect("error printing to stderr");
    } }
);


macro_rules! stderrln(
    ($($arg:tt)*) => { {
        let result = writeln!(&mut ::std::io::stderr(), $($arg)*);
        result.expect("error printing to stderr");
    } }
);


struct Counter {
	count: u64,
	total: u64,
}


impl Counter {
	fn inc(&mut self, path: &Path, size: u64) {
	    self.count += 1;
	    self.total += size;
	    if self.count % 10000 == 0 {
	        let s = path.to_string_lossy().chars().take(78).collect::<String>();
    	    stderr!("\r{} files, {}: {:<78}..", self.count, format_size(self.total), s);
	    }
	}

	fn finalize(&self) {
        stderrln!("\r{} files, {}  {:<80}", self.count, format_size(self.total), "");
	}
}


struct User {
    n_files: u64,
    n_links: u64,
    n_bytes: u64,
    files: FileHashSet,
}


type UserMap = ::std::collections::HashMap<u32, User>;


fn format_size(n_bytes: u64) -> String {
    let (div, desc) = match n_bytes {
        0 ..= 1023 => return format!("{}", n_bytes),
        1024 ..= 1048575 => (2u64.pow(10), " KB"),
        1048576 ..= 1073741823 => (2u64.pow(20), " MB"),
        1073741824 ..= 1099511627775 => (2u64.pow(30), " GB"),
        _ => (2u64.pow(40), " TB"),
    };

    format!("{:.1}{}", n_bytes as f64 / div as f64, desc)
}


fn walk(path: &Path, func: &mut dyn FnMut(&Path, &Metadata)) {
    let metadata = match path.symlink_metadata() {
        Ok(v) => v,
        Err(e) => {
            stderrln!("\nError retrieving metadata for {:?}: {:?}", path, e);
            return;
        },
    };

    func(path, &metadata);

    let path = ::std::path::PathBuf::from(path);
    let ftype = metadata.file_type();
    if !ftype.is_symlink() && ftype.is_dir() {
        let records = match ::std::fs::read_dir(&path) {
            Ok(v) => v,
            Err(e) => {
                stderrln!("\nError reading directory {:?}: {:?}", &path, e);
                return;
            },
        };

        for record in records {
            match record {
                Ok(v) => {
                    walk(&v.path(), func);
                },
                Err(e) => {
                    stderrln!("\nError reading file record in {:?}: {:?}", &path, e);
                    return;
                },
            };
        }
    }
}


fn collect_stats(path: &str, apparent_size: bool, users: &mut UserMap, counter: &mut Counter) {
    stderrln!("Collecting statistics for {:?}", path);

    walk(path.as_ref(), &mut |path, metadata| {
        let mut user = users.entry(metadata.uid()).or_insert_with(|| {
            User {n_bytes: 0,
                  n_files: 0,
                  n_links: 0,
                  files: FileHashSet::new()}
        });

        let key = (metadata.dev(), metadata.ino());
        if user.files.insert(key) {
            user.n_files += 1;
            let len = if apparent_size {
                metadata.len()
            } else {
                metadata.blocks() * 512
            };

            user.n_bytes += len;
            counter.inc(path, len);
        } else {
            user.n_links += 1;
            counter.inc(path, 0);
        }
    });

    counter.finalize();
}


fn calculate_totals(users: &UserMap) -> User {
    let mut total = User {
        n_bytes: 0,
        n_files: 0,
        n_links: 0,
        files: FileHashSet::new(),
    };

    for stats in users.values() {
        total.n_files += stats.n_files;
        total.n_links += stats.n_links;
        total.n_bytes += stats.n_bytes;
    }

    total
}


fn print_user(name: &str, user: &User, total: f64) {
    println!("{}\t{}\t{}\t{}\t{}\t{:.3}",
             name, user.n_files, user.n_links,
             format_size(user.n_bytes), user.n_bytes,
             user.n_bytes as f64 / total);
}


fn print_stats(users: &UserMap, total: &User) {
    println!("\nUser\tNFiles\tNLinks\tSize\tBytes\tFrac");
    let mut users: Vec<_> = users.iter().collect();
    users.sort_by_key(|v| v.1.n_bytes);

    for (uid, stats) in users {
        let username = match get_username(*uid) {
            Ok(v) => v,
            Err(_) => format!("{}", uid),
        };

        print_user(&username, stats, total.n_bytes as f64);
    }
    print_user("*", total, total.n_bytes as f64);
}


fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("disk_usage")
        .version("0.0.1")
        .author("Mikkel Schubert")

        .arg(Arg::with_name("apparent-size")
             .long("apparent-size")
             .help("Calculate apparent size rather than block size."))

        .arg(Arg::with_name("root")
             .multiple(true)
             .help("Root folder or file."))

        .get_matches()
}


fn parse_strings(args: &ArgMatches, key: &str) -> Vec<String> {
    if let Some(values) = args.values_of(key) {
        values.map(|v| v.into()).collect()
    } else {
        vec![".".into()]
    }
}


fn main() {
    let args = parse_args();
    let apparent_size = args.is_present("apparent-size");

    let mut users = UserMap::new();
    let mut counter = Counter { count: 0, total: 0 };
    for path in parse_strings(&args, "root") {
        collect_stats(&path, apparent_size, &mut users, &mut counter);
    }

    let total = calculate_totals(&users);
    print_stats(&users, &total);
}
