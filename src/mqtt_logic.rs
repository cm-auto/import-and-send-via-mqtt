use std::time::Duration;

use rumqttc::{Client, Connection, MqttOptions, QoS};

use crate::models::Person;

pub fn init_mqtt(id: &str, host: &str, port: u16) -> (Client, Connection) {
    let mut mqttoptions = MqttOptions::new(id, host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    Client::new(mqttoptions, 10)
}

pub fn publish_person(person: &Person, mqtt_client: &mut Client) {
    // I already explained, how errors could be handled in main
    // so I just unwrap here for brevity
    let json_bytes = serde_json::to_vec(&person).unwrap();
    mqtt_client
        .publish("person", QoS::AtLeastOnce, false, json_bytes)
        .unwrap();
}
