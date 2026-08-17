use std::time::{Duration, Instant};

use linkprobe_core::Error;
use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use url::Url;

pub fn publish_json(
    broker_url: &str,
    topic: &str,
    username: Option<&str>,
    password: Option<&str>,
    payload: &str,
) -> Result<(), Error> {
    let parsed = Url::parse(broker_url).map_err(|e| Error::Message(format!("url: {e}")))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| Error::Message("url missing host".into()))?;
    let port = parsed.port().unwrap_or(1883);

    let mut opts = MqttOptions::new(format!("linkprobe-{}", std::process::id()), host, port);
    opts.set_keep_alive(Duration::from_secs(5));
    if let Some(user) = username {
        opts.set_credentials(user, password.unwrap_or(""));
    }

    let (client, mut connection) = Client::new(opts, 10);
    client
        .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
        .map_err(|e| Error::Mqtt(format!("publish: {e}")))?;

    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if Instant::now() > deadline {
            return Err(Error::Mqtt("publish timed out".into()));
        }
        match connection.recv() {
            Ok(Ok(Event::Incoming(Incoming::PubAck(_)))) => return Ok(()),
            Ok(Ok(_)) => continue,
            Ok(Err(e)) => {
                return Err(Error::Mqtt(format!(
                    "cannot connect to {host}:{port} ({e})"
                )));
            }
            Err(e) => {
                return Err(Error::Mqtt(format!(
                    "cannot connect to {host}:{port} ({e:?})"
                )));
            }
        }
    }
}
