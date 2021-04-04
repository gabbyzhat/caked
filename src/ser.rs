use std::io::Write;
use std::fs::File;
use std::path::Path;
use crate::{KeyValuePair, Value};

/// todo
pub fn ser_str(output: &[KeyValuePair]) -> String {
    let mut tabs = String::from("\t");
    let mut buf = String::new();
    buf.push_str("<?php\nreturn [\n");
    ser_str_ex(output, &mut tabs, &mut buf);
    buf.push_str("];\n");
    buf
}

fn ser_str_ex(output: &[KeyValuePair], tabs: &mut String, buf: &mut String) {
    for kvp in output {
        buf.push_str(&tabs);
        buf.push_str(&kvp.key_prefix());

        match &kvp.value {
            Value::Set(put) => {
                if put.is_empty() {
                    buf.push_str("[]");
                } else {
                    buf.push('[');
                    tabs.push('\t');
                    ser_str_ex(&put, tabs, buf);
                    tabs.pop();
                    buf.push(']');
                }
            }
            x => {
                buf.push_str(&format!("{}", x));
            }
        }
        buf.push_str(",\n")
    }
}

/// todo
pub fn ser_file<P: AsRef<Path>>(output: &[KeyValuePair], path: P) -> std::io::Result<()> {
    let mut buf = File::create(path)?;
    let mut tabs = String::from("\t");
    write!(&mut buf, "<?php\nreturn [\n")?;
    ser_buf_ex(output, &mut tabs, &mut buf)?;
    write!(&mut buf, "];\n")?;
    Ok(())
}

/// todo
pub fn ser_write<W: Write>(output: &[KeyValuePair], mut buf: W) -> std::io::Result<()> {
    let mut tabs = String::from("\t");
    write!(&mut buf, "<?php\nreturn [\n")?;
    ser_buf_ex(output, &mut tabs, &mut buf)?;
    write!(&mut buf, "];\n")?;
    Ok(())
}


fn ser_buf_ex<W: Write>(output: &[KeyValuePair], tabs: &mut String, mut buf: W) -> std::io::Result<()> {
    for kvp in output {
        write!(&mut buf, "{}{}", tabs, kvp.key_prefix())?;

        match &kvp.value {
            Value::Set(put) => {
                if put.is_empty() {
                    write!(&mut buf, "[]")?;
                } else {
                    write!(&mut buf, "[")?;
                    tabs.push('\t');
                    ser_buf_ex(&put, tabs, &mut buf)?;
                    tabs.pop();
                    write!(&mut buf, "]")?;
                }
            }
            x => {
                write!(&mut buf, "{}", x)?;
            }
        }
        write!(&mut buf, ",\n")?;
    }
    Ok(())
}
