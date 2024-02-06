use kafka::client::KafkaClient;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::error::Error as KafkaError;
use kafka::Result;
use log::info;
use prost::Message;
use prost::DecodeError;
use std::io;

// use rdkafka::client::Client;
// use rdkafka::consumer::{BaseConsumer, Consumer as RdConsumer};
// use rdkafka::error::KafkaResult;
// use rdkafka::ClientConfig;
// pub mod over18 {
    pub mod proto {
        include!(concat!(env!("OUT_DIR"), "/over18.proto.rs"));
    }
// }


fn main() {
    tracing_subscriber::fmt::init();
    let topics = dump_metadata().unwrap();
    let turned18 = topics.clone().into_iter().find(|it| it.contains("over_18"));

    match turned18 {
        Some(turned18) => {
            log::info!("My Topic: {}", turned18);
            let broker = "localhost:9784".to_owned();

            if let Err(e) = consume_messages(turned18.clone(), turned18, vec![broker]) {
                println!("Failed consuming messages: {}", e);
            }
        },
        None => {
            log::error!("Topic turned18 does not exist");
            topics.into_iter().for_each(|t| log::info!("   {}", t));
        }
    }


}

// fn dump_metadata2()  -> KafkaResult<Vec<String>> {
//     let consumer: BaseConsumer = ClientConfig::new()
//     .set("bootstrap.servers", "localhost:9784")
//     .create()?;

//     trace!("Consumer created");

//     let metadata = consumer.fetch_metadata(None, time::Duration::from_secs(10))?;
//     let topics = metadata.topics();
//     let topic_names = topics.iter().map(|t| {
//         t.name().to_owned()
//     }).collect();

//     Ok(topic_names)
// }

fn consume_messages(group: String, topic: String, brokers: Vec<String>) -> Result<()> {
    let mut con = Consumer::from_hosts(brokers)
        .with_topic(topic)
        .with_group(group)
        .with_fallback_offset(FetchOffset::Earliest)
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()?;

    loop {
        let mss = con.poll()?;
        if mss.is_empty() {
            println!("No messages available right now.");
            return Ok(());
        }

        // Over18MessagePayloadPbT
        for ms in mss.iter() {
            for m in ms.messages() {
                info!("Offset {}", m.offset);
                let over18: proto::Over18MessagePbT = proto::Over18MessagePbT::decode(m.value).unwrap();
                info!("  Entity ID = {}", over18.message_header.clone().unwrap().entity_id);
                info!("  Customer Name = {}", over18.subset.clone().unwrap().generic.unwrap().name);
                info!("  Customer Age = {}", over18.subset.clone().unwrap().generic.unwrap().customer_age);
                info!("  Amount Spent = {}", over18.message_payload.clone().unwrap().amount_spent);
                // info!("----------------------------------------------------------------------");
                // info!("{:?}", over18);
                // info!("----------------------------------------------------------------------");
            
                // println!(
                //     "{}:{}@{}: {:?}",
                //     ms.topic(),
                //     ms.partition(),
                //     m.offset,
                //     m.value
                // );
            }
            let _ = con.consume_messageset(ms);
        }
        con.commit_consumed()?;
    }
}

fn dump_metadata() -> Result<Vec<String>> { 
    let broker = vec!["localhost:9784".into()];
    let mut client = KafkaClient::new(broker);
    client.load_metadata_all()?;
    let topics = client.topics();
    let topic_names = topics.names();
    let names = topic_names.into_iter().map(|name| name.to_owned()).collect();

    Ok(names)
}