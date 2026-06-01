use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use regex::bytes::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

struct Options {
    filter_in_groups: Vec<Regex>,
    filter_out_groups: Vec<Regex>,
    filter_in_cells: Vec<Regex>,
    filter_out_cells: Vec<Regex>,
    input_file: String,
    output_file: String,
    remove_comments: bool,
}

fn usage() {
    println!("USAGE:");
    println!("  liberty_filter --in-file <file_path> --out-file <file_path> [OPTIONS]");
    println!("OPTIONS:");
    println!("  --filter-in-groups <regex>   Regex pattern of groups to keep. Can be repeated.");
    println!("  --filter-out-groups <regex>  Regex pattern of groups to delete. Can be repeated.");
    println!("  --filter-in-cells <regex>    Regex pattern of cells to keep. Can be repeated.");
    println!("  --filter-out-cells <regex>   Regex pattern of cells to delete. Can be repeated.");
    println!("  --in-file <file_path>        Liberty file to process");
    println!("  --out-file <file_path>       Write processed Liberty data to this file");
    println!("  --remove-comments            Remove comments");
    println!("  --help                       produce help message");
}

fn compile_regex(pattern: String) -> Result<Regex, String> {
    Regex::new(&pattern).map_err(|err| format!("invalid regex {pattern:?}: {err}"))
}

fn parse_args() -> Result<Option<Options>, String> {
    let mut filter_in_groups = Vec::new();
    let mut filter_out_groups = Vec::new();
    let mut filter_in_cells = Vec::new();
    let mut filter_out_cells = Vec::new();
    let mut input_file = None;
    let mut output_file = None;
    let mut remove_comments = false;
    let mut positional = Vec::new();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => return Ok(None),
            "--remove-comments" => remove_comments = true,
            "--filter-in-groups" => {
                let value = args.next().ok_or("--filter-in-groups requires a value")?;
                filter_in_groups.push(compile_regex(value)?);
            }
            "--filter-out-groups" => {
                let value = args.next().ok_or("--filter-out-groups requires a value")?;
                filter_out_groups.push(compile_regex(value)?);
            }
            "--filter-in-cells" => {
                let value = args.next().ok_or("--filter-in-cells requires a value")?;
                filter_in_cells.push(compile_regex(value)?);
            }
            "--filter-out-cells" => {
                let value = args.next().ok_or("--filter-out-cells requires a value")?;
                filter_out_cells.push(compile_regex(value)?);
            }
            "--in-file" => {
                input_file = Some(args.next().ok_or("--in-file requires a value")?);
            }
            "--out-file" => {
                output_file = Some(args.next().ok_or("--out-file requires a value")?);
            }
            _ if arg.starts_with("--") => return Err(format!("unrecognized option {arg}")),
            _ => positional.push(arg),
        }
    }

    if input_file.is_none() {
        input_file = positional.into_iter().next();
    }

    let input_file = input_file
        .ok_or("Error: You must specify an input file, --in-file <file_path>, or <file_path>.")?;
    let output_file =
        output_file.ok_or("Error: You must specify an output file, --out-file <file_path>")?;

    Ok(Some(Options {
        filter_in_groups,
        filter_out_groups,
        filter_in_cells,
        filter_out_cells,
        input_file,
        output_file,
        remove_comments,
    }))
}

fn open_input(path: &str, buf_size: usize) -> io::Result<Box<dyn Read>> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(buf_size, file);
    if path.contains(".gz") {
        Ok(Box::new(GzDecoder::new(reader)))
    } else {
        Ok(Box::new(reader))
    }
}

fn open_output(path: &str, buf_size: usize) -> io::Result<Box<dyn Write>> {
    let file = File::create(path)?;
    let writer = BufWriter::with_capacity(buf_size, file);
    if path.contains(".gz") {
        Ok(Box::new(GzEncoder::new(writer, Compression::default())))
    } else {
        Ok(Box::new(writer))
    }
}

fn flush_vec<W: Write + ?Sized>(writer: &mut W, buf: &mut Vec<u8>) -> io::Result<()> {
    writer.write_all(buf)?;
    buf.clear();
    Ok(())
}

fn should_filter(
    opts: &Options,
    tmp: &[u8],
    compact: &mut Vec<u8>,
    group_name: &mut Vec<u8>,
) -> bool {
    compact.clear();
    group_name.clear();

    let mut in_quotes = false;
    let mut in_group_name = false;
    for &c in tmp {
        if in_quotes {
            if c == b'"' {
                in_quotes = false;
            }
        } else if c == b'"' {
            in_quotes = true;
        } else if c == b'\n' || c == b' ' || c == b'\t' {
            continue;
        }

        compact.push(c);

        if in_group_name {
            if c == b')' && !in_quotes {
                in_group_name = false;
            } else if c != b'"' {
                group_name.push(c);
            }
        } else if c == b'(' && !in_quotes {
            in_group_name = true;
        }
    }

    let mut match_filter_out_cell = false;
    let mut match_filter_in_cell = false;
    if compact.starts_with(b"cell(") {
        match_filter_out_cell = opts
            .filter_out_cells
            .iter()
            .any(|re| re.is_match(group_name));
        match_filter_in_cell = opts
            .filter_in_cells
            .iter()
            .any(|re| re.is_match(group_name));
    }

    let match_filter_out_group = opts.filter_out_groups.iter().any(|re| re.is_match(compact));
    let match_filter_in_group = opts.filter_in_groups.iter().any(|re| re.is_match(compact));

    (match_filter_out_group && !match_filter_in_group)
        || (match_filter_out_cell && !match_filter_in_cell)
}

fn run(opts: &Options) -> io::Result<()> {
    let buf_size = env::var("LIBERTY_FILTER_BUF_SIZE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(4096 * 2);

    let mut input = open_input(&opts.input_file, buf_size)?;
    let mut output = open_output(&opts.output_file, buf_size)?;

    let mut in_buf = vec![0_u8; buf_size];
    let mut out_buf = Vec::with_capacity(buf_size);
    let mut tmp_buf = Vec::with_capacity(buf_size);
    let mut compact = Vec::with_capacity(buf_size);
    let mut group_name = Vec::with_capacity(buf_size);

    let mut c = b'\n';
    let mut removing_group = false;
    let mut in_quotes = false;
    let mut in_comment = false;
    let mut brace_count = 0_i32;

    loop {
        let n = input.read(&mut in_buf)?;
        if n == 0 {
            break;
        }

        for &byte in &in_buf[..n] {
            if removing_group {
                if byte == b'{' {
                    brace_count += 1;
                } else if byte == b'}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        removing_group = false;
                    }
                }
                continue;
            }

            if in_quotes {
                tmp_buf.push(byte);
                if byte == b'"' {
                    in_quotes = false;
                }
                if tmp_buf.len() == buf_size {
                    out_buf.extend_from_slice(&tmp_buf);
                    tmp_buf.clear();
                    if out_buf.len() >= buf_size {
                        output.write_all(&out_buf)?;
                        out_buf.clear();
                    }
                }
                continue;
            }

            let prev_c = c;
            c = byte;

            if in_comment {
                if prev_c == b'*' && c == b'/' {
                    in_comment = false;
                    if !opts.remove_comments {
                        tmp_buf.push(b'/');
                        out_buf.extend_from_slice(&tmp_buf);
                        tmp_buf.clear();
                        if out_buf.len() >= buf_size {
                            output.write_all(&out_buf)?;
                            out_buf.clear();
                        }
                    }
                    continue;
                }
                if !opts.remove_comments {
                    tmp_buf.push(c);
                    if tmp_buf.len() == buf_size {
                        out_buf.extend_from_slice(&tmp_buf);
                        tmp_buf.clear();
                        if out_buf.len() >= buf_size {
                            output.write_all(&out_buf)?;
                            out_buf.clear();
                        }
                    }
                    continue;
                }
            } else if prev_c == b'/' && c == b'*' {
                in_comment = true;
                if opts.remove_comments {
                    tmp_buf.pop();
                    continue;
                }
            }

            match c {
                b'"' => {
                    in_quotes = true;
                    tmp_buf.push(c);
                    if tmp_buf.len() == buf_size {
                        out_buf.extend_from_slice(&tmp_buf);
                        tmp_buf.clear();
                        if out_buf.len() >= buf_size {
                            output.write_all(&out_buf)?;
                            out_buf.clear();
                        }
                    }
                }
                b'{' => {
                    if should_filter(opts, &tmp_buf, &mut compact, &mut group_name) {
                        brace_count = 1;
                        removing_group = true;
                    } else {
                        tmp_buf.push(b'{');
                        out_buf.extend_from_slice(&tmp_buf);
                        if out_buf.len() >= buf_size {
                            output.write_all(&out_buf)?;
                            out_buf.clear();
                        }
                    }
                    tmp_buf.clear();
                }
                b'}' | b';' => {
                    tmp_buf.push(c);
                    out_buf.extend_from_slice(&tmp_buf);
                    tmp_buf.clear();
                    if out_buf.len() >= buf_size {
                        output.write_all(&out_buf)?;
                        out_buf.clear();
                    }
                }
                _ => {
                    if c != b'\n' || prev_c != b'\n' {
                        tmp_buf.push(c);
                        if tmp_buf.len() == buf_size {
                            out_buf.extend_from_slice(&tmp_buf);
                            tmp_buf.clear();
                            if out_buf.len() >= buf_size {
                                output.write_all(&out_buf)?;
                                out_buf.clear();
                            }
                        }
                    }
                }
            }
        }
    }

    flush_vec(&mut output, &mut out_buf)?;
    output.write_all(&tmp_buf)?;
    output.flush()
}

fn main() {
    match parse_args() {
        Ok(Some(opts)) => {
            if let Err(err) = run(&opts) {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
        Ok(None) => {
            usage();
            std::process::exit(1);
        }
        Err(err) => {
            println!("{err}");
            std::process::exit(1);
        }
    }
}
