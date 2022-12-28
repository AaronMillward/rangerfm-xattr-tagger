extern crate getopts;

const XATTR_TAG_NAME : &str = "user.tags";
const XATTR_TAG_DELIMITER : &str = ";";

fn print_usage(program: &str, opts: getopts::Options) {
	eprint!("{}", opts.usage(&format!("Usage: {} [options] FILES", program)));
}

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let program = args[0].clone();

	let mut opts = getopts::Options::new();
	opts.optflag( "h", "help",       "show help");
	opts.optflag( "v", "verbose",    "increased vebosity");
	opts.optflag( "c" , "csv",        "print csv of file tags");
	opts.optmulti("a", "add-tag",    "add tag to files",      "NAME");
	opts.optmulti("x", "remove-tag", "remove tag from files", "NAME");
	opts.parsing_style(getopts::ParsingStyle::StopAtFirstFree);

	let matches = match opts.parse(&args[1..]) {
		Ok(m)  => { m }
		Err(f) => { eprintln!("{}", f.to_string()); print_usage(&program, opts); return }
	};
	
	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	}

	let verbose = matches.opt_present("v");
	
	let add_tag = matches.opt_strs("a");
	let remove_tag = matches.opt_strs("x");

	let files = if !matches.free.is_empty() {
		matches.free.clone()
	} else {
		print_usage(&program, opts);
		return;
	};

	{ //Check if all files exist, end program if not
		let mut failed = false;

		for f in &files {
			if std::fs::metadata(f).is_err() {
				eprintln!("File {} does not exist", f);
				failed = true;
			}
		}

		if failed {
			eprintln!("All files must exist before running, aborting...");
			return
		}
	}

	if verbose {
		eprintln!("adding tags |");
		for t in &add_tag {
			eprintln!("=> {}", t);
		}
		eprintln!("removing tags |");
		for t in &remove_tag {
			eprintln!("=> {}", t);
		}
		eprintln!("on files |");
		for f in &files {
			eprintln!("=> {}", f);
		}
	}

	for f in &files {
		let mut tags = std::vec::Vec::<String>::new();
		
		{ //Read existing tags from xattr 
			let existing_tag_str = xattr::get(f,XATTR_TAG_NAME);
			if existing_tag_str.is_ok() {
				let x = existing_tag_str.unwrap();
				if x.is_some() {
					match String::from_utf8(x.unwrap()) {
						Ok(v) => {
							for s in v.split(XATTR_TAG_DELIMITER) {
								tags.push(String::from(s));
							}
						} ,
						Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
					};
				}
			}
		}

		{ // Add tags
			//I don't know if vec has some kind of clone_into but until then I'm using a loop
			for t in &add_tag {
				tags.push(t.clone())
			}
		}

		{ // Remove duplicates
			tags.sort_unstable();
			tags.dedup();
		}

		{ // Remove tags
			for t in &remove_tag {
				let index = tags.iter().position(|x| *x == *t);
				if index.is_some() {
					tags.remove(index.unwrap());
				}
			}
		}

		if verbose {
			eprint!("Tags set on {}: ", f);
			for t in &tags {
				eprint!("{}, ", t);
			}
			eprint!("\n");
		}

		{ // Create csv string and apply it to the xattr (optionally print if flag set)
			let s_opt = tags.into_iter().reduce(|accum,item| { if accum.is_empty() { item } else { accum + XATTR_TAG_DELIMITER + &item} });
			let res = match s_opt {
				Some(s) => {
					if matches.opt_present("c") {
						println!("{}", s)
					}
					xattr::set(f, XATTR_TAG_NAME, s.as_bytes())
				},
				None => xattr::set(f, XATTR_TAG_NAME, &[] as &[u8]),
			};
			if res.is_err() && verbose {
				eprintln!("Unable to set tags on file: {}", f);
			}
		}
	}
}
