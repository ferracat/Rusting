use std::fs::File;
use std::io::{Read, Write};
use protobuf::Message;
use person::*;


include!(concat!(env!("CARGO_MANIFEST_DIR"), "/person.rs"));

pub mod person {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/person.rs"));
}

fn write_to_file(people_list: &person::PeopleList, filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    people_list.write_to_writer(&mut file)?;
    Ok(())
}

fn read_from_file(filename: &str) -> std::io::Result<person::PeopleList> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let decoded_people_list = person::PeopleList::parse_from_bytes(&buffer)?;
    Ok(decoded_people_list)
}

fn main() {
    let mut people_list = person::PeopleList::new();

    let mut p1 = person::Person::new();
    p1.set_name("John".to_string());
    p1.set_id(123);
    p1.set_email("john@example.com".to_string());

    let mut p2 = person::Person::new();
    p2.set_name("Jane".to_string());
    p2.set_id(456);
    p2.set_email("jane@example.com".to_string());

    people_list.set_people(vec![p1, p2].into());

    // Write to a file
    write_to_file(&people_list, "people_list.pb").expect("Failed to write to file");

    // Read from a file
    let decoded_people_list = read_from_file("people_list.pb").expect("Failed to read from file");

    // Print the deserialized PeopleList
    println!("{:?}", decoded_people_list);
}
