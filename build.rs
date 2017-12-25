use std::fs::File;
use std::io::prelude::*;

const INDEX: &'static str = "./src/templates/chapter/index.html";
const INDEX_OUT: &'static str = "./src/template_fragments/chapter/index.html";
const GEN: &'static str = "./src/templates/chapter/gen.css";
const GEN_OUT: &'static str = "./src/template_fragments/chapter/gen.css";

fn process_file(fname: &'static str, oname: &'static str) {
	let mut f = File::open(fname).expect(
		"Precompilation error: template file not found"
	);
	let mut temp_cont = String::new();
	f.read_to_string(&mut temp_cont)
		.expect("Precompilation error: Could not read template file");
	let sub_files: Vec<&str> = temp_cont.split("{{ BREAK }}").collect();
	let mut i = 0;
	for sub in sub_files {
		let mut out_buf = File::create(oname.to_string()+&i.to_string()).expect(
			"Precompilation error: Could not create sub file"
		);
		out_buf.write(sub.as_bytes()).expect(
			"Precompilation error: Could not write to sub file"
		);
		i += 1;
	}
}

fn main() {
	process_file(INDEX, INDEX_OUT);
	process_file(GEN, GEN_OUT);
}
