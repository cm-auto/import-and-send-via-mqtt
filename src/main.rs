use std::{fmt::Debug, path::Path, process, sync::mpsc::channel, thread};

use notify::{recommended_watcher, Error, Event, Watcher};
use rumqttc::Client;

mod models;
mod notify_logic;
use notify_logic::*;
mod mqtt_logic;
use mqtt_logic::*;

fn print_value_and_exit<T: Debug>(value: T) -> ! {
    dbg!(value);
    process::exit(1);
}

fn unwrap_or_exit<T, E>(result: Result<T, E>, exit_function: fn(E) -> !) -> T {
    match result {
        Err(error) => exit_function(error),
        Ok(value) => value,
    }
}

fn import_file_and_send_over_mqtt(event: notify::Event, mqtt_client: &mut Client) {
    let csv_paths = get_csv_paths_from_notify_event(&event);

    for path in csv_paths {
        // I already explained, how errors could be handled in main
        // so I just unwrap here for brevity
        let persons = read_vector_of_persons(path).unwrap();
        persons
            .iter()
            .for_each(|person| publish_person(person, mqtt_client))
    }
}

fn handle_notify_event_result(
    event_result: Result<notify::Event, notify::Error>,
    client: &mut Client,
) {
    // let event = match result {
    //     Ok(event) => event,
    //     Err(error) => {
    //         eprintln!("{}", error);
    //         continue;
    //     }
    // };

    // in my opinion this is more readable than
    // the previous code using match.
    // and since we checked for an error
    // we can safely unwrap
    if let Err(error) = event_result {
        dbg!(error);
        return;
    };
    let event = event_result.unwrap();

    // we only care about a file being moved into or
    // written to in the watched directory
    if !is_event_move_into_or_write(event.kind) {
        return;
    }

    import_file_and_send_over_mqtt(event, client);
}

fn main() {
    let (sender, receiver) = channel::<Result<Event, Error>>();

    let mut watcher = unwrap_or_exit(
        recommended_watcher(move |event_result| {
            // on error log and continue
            // if this wasn't a demo, then the
            // error would be written into a log file
            // or otherwise logged
            if let Err(error) = sender.send(event_result) {
                dbg!(error);
            };
        }),
        print_value_and_exit,
    );

    // since this is a demo, this value is hardcoded
    // otherwise we would take it from .env, or program
    // arguments or something like that
    let path = Path::new("./watch-dir");
    unwrap_or_exit(
        watcher.watch(path, notify::RecursiveMode::Recursive),
        print_value_and_exit,
    );

    let (mut client, mut connection) = init_mqtt("rust-csv-importer", "localhost", 1883);

    // spawn a thread to poll the connection
    thread::spawn(move || {
        connection.iter().for_each(|result| {
            if let Err(error) = result {
                dbg!(error);
            }
        })
    });

    // if handle_notify_event_result only took one argument
    // I would prefer the following:
    // receiver.iter().for_each(handle_notify_event_result);

    // However, it also needs the mqtt client, which forces us to
    // use a closure as the argument to for_each:
    // receiver.iter()
    //     .for_each(|result| handle_notify_event_result(result, &mut client));

    // so I decided to do it this way,
    // which is more readable in my opinion
    for event_result in receiver {
        handle_notify_event_result(event_result, &mut client);
    }
}

#[cfg(test)]
mod tests {
    use stringreader::StringReader;

    use crate::{models::Person, read_vector_of_persons_from_reader};

    #[test]
    fn read_one_person() {
        let string_reader = StringReader::new(
            r#"Id,Name,Age
1,Peter,45
"#,
        );
        let persons = read_vector_of_persons_from_reader(string_reader).unwrap();
        let expected = vec![Person {
            id: 1,
            first_name: "Peter".to_string(),
            age: 45,
        }];
        assert_eq!(persons, expected);
    }

    #[test]
    fn read_two_persons() {
        let string_reader = StringReader::new(
            r#"Id,Name,Age
1,Peter,45
2,Mary,22
"#,
        );
        let persons = read_vector_of_persons_from_reader(string_reader).unwrap();
        let expected = vec![
            Person {
                id: 1,
                first_name: "Peter".to_string(),
                age: 45,
            },
            Person {
                id: 2,
                first_name: "Mary".to_string(),
                age: 22,
            },
        ];
        assert_eq!(persons, expected);
    }

    #[test]
    fn read_zero_persons() {
        let string_reader = StringReader::new(
            r#"Id,Name,Age
"#,
        );
        let persons = read_vector_of_persons_from_reader(string_reader).unwrap();
        let expected = vec![];
        assert_eq!(persons, expected);
    }
}
