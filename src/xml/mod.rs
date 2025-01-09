use quick_xml::events::{BytesStart, BytesEnd, BytesText, Event};
use quick_xml::Writer;
use std::io::Cursor;
use std::sync::Arc;

use crate::backend::ResourceMetadata;

pub fn create_multistatus_response(resources: &[ResourceMetadata]) -> Result<String, quick_xml::Error> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    
    // Write XML declaration
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("utf-8"), None)))?;
    
    // Start D:multistatus
    let mut multistatus = BytesStart::new("D:multistatus");
    multistatus.push_attribute(("xmlns:D", "DAV:"));
    writer.write_event(Event::Start(multistatus))?;

    for resource in resources {
        write_response(&mut writer, resource)?;
    }

    // End D:multistatus
    writer.write_event(Event::End(BytesEnd::new("D:multistatus")))?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| quick_xml::Error::Io(Arc::new(std::io::Error::new(std::io::ErrorKind::Other, e))))
}

fn write_response(writer: &mut Writer<Cursor<Vec<u8>>>, resource: &ResourceMetadata) -> Result<(), quick_xml::Error> {
    // Start D:response
    writer.write_event(Event::Start(BytesStart::new("D:response")))?;

    // Write D:href
    writer.write_event(Event::Start(BytesStart::new("D:href")))?;
    writer.write_event(Event::Text(BytesText::new(&resource.path.to_string_lossy())))?;
    writer.write_event(Event::End(BytesEnd::new("D:href")))?;

    // Write D:propstat
    writer.write_event(Event::Start(BytesStart::new("D:propstat")))?;

    // Write D:prop
    writer.write_event(Event::Start(BytesStart::new("D:prop")))?;

    // Write resource type
    writer.write_event(Event::Start(BytesStart::new("D:resourcetype")))?;
    if resource.is_dir {
        writer.write_event(Event::Empty(BytesStart::new("D:collection")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("D:resourcetype")))?;

    // Write getcontentlength
    if !resource.is_dir {
        writer.write_event(Event::Start(BytesStart::new("D:getcontentlength")))?;
        writer.write_event(Event::Text(BytesText::new(&resource.len.to_string())))?;
        writer.write_event(Event::End(BytesEnd::new("D:getcontentlength")))?;
    }

    // Write getlastmodified
    writer.write_event(Event::Start(BytesStart::new("D:getlastmodified")))?;
    writer.write_event(Event::Text(BytesText::new(&resource.modified.to_rfc2822())))?;
    writer.write_event(Event::End(BytesEnd::new("D:getlastmodified")))?;

    // Write getetag
    writer.write_event(Event::Start(BytesStart::new("D:getetag")))?;
    writer.write_event(Event::Text(BytesText::new(&resource.etag)))?;
    writer.write_event(Event::End(BytesEnd::new("D:getetag")))?;

    // End D:prop
    writer.write_event(Event::End(BytesEnd::new("D:prop")))?;

    // Write D:status
    writer.write_event(Event::Start(BytesStart::new("D:status")))?;
    writer.write_event(Event::Text(BytesText::new("HTTP/1.1 200 OK")))?;
    writer.write_event(Event::End(BytesEnd::new("D:status")))?;

    // End D:propstat
    writer.write_event(Event::End(BytesEnd::new("D:propstat")))?;

    // End D:response
    writer.write_event(Event::End(BytesEnd::new("D:response")))?;

    Ok(())
}

pub fn parse_propfind_request(xml: &[u8]) -> Result<Vec<String>, quick_xml::Error> {
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut properties = Vec::new();
    let mut in_prop = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"prop" => in_prop = true,
                    name if in_prop => {
                        properties.push(String::from_utf8_lossy(name).into_owned());
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"prop" {
                    in_prop = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e),
            _ => {}
        }
        buf.clear();
    }

    Ok(properties)
} 