use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use crate::models::Person;

pub fn read_vector_of_persons<T: AsRef<Path>>(path: T) -> Result<Vec<Person>, csv::Error> {
    let file_reader = File::open(path)?;
    read_vector_of_persons_from_reader(file_reader)
}

// having a function that takes a reader instead of a path
// allows for easier testing
pub fn read_vector_of_persons_from_reader<T: Read>(reader: T) -> Result<Vec<Person>, csv::Error> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut persons = vec![];
    for result in rdr.deserialize::<Person>() {
        let record = result?;
        persons.push(record);
    }
    Ok(persons)
}

pub fn is_event_move_into_or_write(event_kind: notify::event::EventKind) -> bool {
    match event_kind {
        notify::EventKind::Modify(notify::event::ModifyKind::Name(
            notify::event::RenameMode::To,
        )) => true,
        notify::EventKind::Access(notify::event::AccessKind::Close(
            notify::event::AccessMode::Write,
        )) => true,
        _ => false,
    }

    // alternatively the following could have been done,
    // however this is less readable imo

    // matches!(
    //     event_kind,
    //     notify::EventKind::Modify(notify::event::ModifyKind::Name(
    //         notify::event::RenameMode::To
    //     ))
    // ) || matches!(
    //     event_kind,
    //     notify::EventKind::Access(notify::event::AccessKind::Close(
    //         notify::event::AccessMode::Write
    //     ))
    // )
}

pub fn get_csv_paths_from_notify_event(event: &notify::Event) -> impl Iterator<Item = &PathBuf> {
    event
        .paths
        .iter()
        .filter(|item| item.extension().is_some_and(|ext| ext == "csv"))
        .into_iter()
}
