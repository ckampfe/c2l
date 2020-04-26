use conch_parser::ast::Command::*;
use conch_parser::ast::ListableCommand::*;
use conch_parser::ast::PipeableCommand::*;
use conch_parser::ast::RedirectOrEnvVar::EnvVar;
use conch_parser::ast::*;
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use std::error::Error;
use std::io::Write;
use structopt::StructOpt;
use xml::escape::escape_str_pcdata;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

#[derive(StructOpt, Debug)]
#[structopt(name = "c2l")]
struct Options {
    #[structopt()]
    label: String,
    #[structopt()]
    cmd: String,
    #[structopt(short, long)]
    run_at_load: bool,
    #[structopt(short, long)]
    keepalive: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::from_args();

    let out = std::io::stdout();
    let out = out.lock();
    let mut config = EmitterConfig::new()
        .line_separator("\n")
        .indent_string("\t")
        .perform_indent(true)
        .write_document_declaration(false)
        .normalize_empty_elements(true)
        .cdata_to_characters(true);

    config.perform_escaping = false;

    let mut writer = EventWriter::new_with_config(out, config);

    let prologue = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
"#;
    writer.write(XmlEvent::Characters(&prologue))?;

    write_element(&mut writer, "plist", Some(vec![["version", "1.0"]]), |w| {
        write_element(w, "dict", None, |w| {
            write_element(w, "key", None, |w| write_chars(w, "Label"))?;
            write_element(w, "string", None, |w| write_chars(w, &options.label))?;

            let lex = Lexer::new(options.cmd.chars());
            let parser = DefaultParser::new(lex);
            for term in parser {
                match term {
                    Ok(TopLevelCommand(List(AndOrList {
                        first: Single(Simple(x)),
                        rest: _rest,
                    }))) => {
                        match *x {
                            SimpleCommand {
                                redirects_or_env_vars,
                                redirects_or_cmd_words,
                            } => {
                                if !redirects_or_cmd_words.is_empty() {
                                    write_element(w, "key", None, |w| {
                                        write_chars(w, "ProgramArguments")
                                    })?;

                                    write_element(w, "array", None, |w| {
                                        for el in &redirects_or_cmd_words {
                                            match el {
                                                RedirectOrCmdWord::CmdWord(TopLevelWord(
                                                    conch_parser::ast::ComplexWord::Single(
                                                        conch_parser::ast::Word::Simple(
                                                            conch_parser::ast::SimpleWord::Literal(
                                                                l,
                                                            ),
                                                        ),
                                                    ),
                                                )) => {
                                                    write_element(w, "string", None, |w| {
                                                        write_chars(w, l)
                                                    })?;
                                                }
                                                e => panic!(
                                                    "Redirect or command not supported: {:?}",
                                                    e
                                                ),
                                            }
                                        }
                                        Ok(())
                                    })?;
                                }

                                if !redirects_or_env_vars.is_empty() {
                                    write_element(w, "key", None, |w| {
                                        write_chars(w, "EnvironmentVariables")
                                    })?;

                                    write_element(w, "dict", None, |w| {
                                        for el in &redirects_or_env_vars {
                                            match el {
                                        EnvVar(
                                            key,
                                            Some(TopLevelWord(
                                                conch_parser::ast::ComplexWord::Single(
                                                    conch_parser::ast::Word::Simple(
                                                        conch_parser::ast::SimpleWord::Literal(l),
                                                    ),
                                                ),
                                            )),
                                        ) => {
                                            write_element(w, "key", None, |w| write_chars(w, key))?;

                                            write_element(w, "string", None, |w| {
                                                write_chars(w, l)
                                            })?;
                                        }
                                        e => panic!("Redirect or envvar not supported: {:?}", e),
                                    }
                                        }
                                        Ok(())
                                    })?;
                                }
                            }
                        }
                    }
                    _ => panic!(),
                }
            }

            if options.run_at_load {
                write_element(w, "key", None, |w| write_chars(w, "RunAtLoad"))?;
                write_element(w, "true", None, |_w| Ok(()))?;
            }
            if options.keepalive {
                write_element(w, "key", None, |w| write_chars(w, "KeepAlive"))?;
                write_element(w, "true", None, |_w| Ok(()))?;
            }

            Ok(())
        })?;
        Ok(())
    })?;

    Ok(())
}

fn write_chars<W: Write, I: AsRef<str>>(
    w: &mut EventWriter<W>,
    i: I,
) -> Result<(), Box<dyn Error>> {
    let as_ref = i.as_ref();
    let escaped = escape_str_pcdata(as_ref);
    let c = XmlEvent::characters(&escaped);
    w.write(c)?;
    Ok(())
}

fn write_element<W: Write>(
    writer: &mut EventWriter<W>,
    element: &str,
    attributes: Option<Vec<[&str; 2]>>,
    job: impl Fn(&mut EventWriter<W>) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    if let Some(attrs) = attributes {
        let mut start = XmlEvent::start_element(element);
        for [k, v] in attrs {
            start = start.attr(k, v);
        }
        writer.write(start)?;
    } else {
        let start = XmlEvent::start_element(element);
        writer.write(start)?;
    }

    job(writer)?;

    let end = XmlEvent::end_element();
    writer.write(end)?;

    Ok(())
}
