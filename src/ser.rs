use std::io::Write;
use std::fs::File;
use std::path::Path;
use crate::{KeyValuePair, Value};

/// todo
pub fn ser_str(output: &[KeyValuePair]) -> String {
    let mut tabs = String::from("\t");
    let mut buf = String::new();
    write!(buf, "<?php\nreturn [\n");
    ser_str_ex(output, &mut tabs, &mut buf);
    write!(buf, "];\n");
    buf
}

fn ser_str_ex(output: &[KeyValuePair], tabs: &mut String, buf: &mut String) {
    for kvp in output {
        write!(buf, "{}{}", tabs, kvp.key_prefix());

        match &kvp.value {
            Value::Set(put) => {
                if put.is_empty() {
                    write!(buf, "[]");
                } else {
                    write!(buf, "[");
                    tabs.push('\t');
                    ser_str_ex(&put, tabs, buf);
                    tabs.pop();
                    write!(buf, "]");
                }
            }
            x => {
                write!(buf, "{}", x);
            }
        }
        write!(buf, ",\n")
    }
}

/// todo
pub fn ser_file<P: AsRef<Path>>(output: &[KeyValuePair], path: P) {
    let mut buf = File::create(path).unwrap();
    let mut tabs = String::from("\t");
    write!(buf, "<?php\nreturn [\n");
    ser_buf_ex(output, &mut tabs, &mut buf);
    write!(buf, "];\n");
}

/// todo
pub fn ser_write<W: Write>(output: &[KeyValuePair], mut buf: W) {
    let mut tabs = String::from("\t");
    write!(buf, "<?php\nreturn [\n");
    ser_buf_ex(output, &mut tabs, &mut buf);
    write!(buf, "];\n");
}


fn ser_buf_ex<W: Write>(output: &[KeyValuePair], tabs: &mut String, mut buf: W) {
    for kvp in output {
        write!(buf, "{}{}", tabs, kvp.key_prefix());

        match &kvp.value {
            Value::Set(put) => {
                if put.is_empty() {
                    write!(buf, "[]");
                } else {
                    write!(buf, "[");
                    tabs.push('\t');
                    ser_buf_ex(&put, tabs, buf);
                    tabs.pop();
                    write!(buf, "]");
                }
            }
            x => {
                write!(buf, "{}", x);
            }
        }
        write!(buf, ",\n")
    }
}
