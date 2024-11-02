use crossbeam_channel::{unbounded, Sender};
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, Consumer, message::Delivery};
use std::thread;
use std::time::Duration;
use tokio_amqp::LapinTokioExt;
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct InventoryItem {
    name: String,
    quantity: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Customer {
    name: String,
    email: String,
}

pub async fn send_inventory_to_rabbitmq(item: InventoryItem) {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://localhost:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_default_executor())
        .await
        .expect("Connection error");
    let channel = conn.create_channel().await.expect("Channel error");
    let payload = serde_json::to_string(&item).expect("Serialization error");

    channel
        .basic_publish(
            "",
            "inventory_queue",
            BasicPublishOptions::default(),
            payload.as_bytes(),
            BasicProperties::default(),
        )
        .await
        .expect("Publish error");
}

pub async fn send_customer_to_rabbitmq(customer: Customer) {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://localhost:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_default_executor())
        .await
        .expect("Connection error");
    let channel = conn.create_channel().await.expect("Channel error");
    let payload = serde_json::to_string(&customer).expect("Serialization error");

    channel
        .basic_publish(
            "",
            "customer_queue",
            BasicPublishOptions::default(),
            payload.as_bytes(),
            BasicProperties::default(),
        )
        .await
        .expect("Publish error");
}

pub async fn send_notification_to_rabbitmq(notification: &str) {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://localhost:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_default_executor())
        .await
        .expect("Connection error");
    let channel = conn.create_channel().await.expect("Channel error");

    channel
        .basic_publish(
            "",
            "notification_queue",
            BasicPublishOptions::default(),
            notification.as_bytes(),
            BasicProperties::default(),
        )
        .await
        .expect("Publish error");
}

pub fn simulate_producer(sender: Sender<InventoryItem>) {
    let mut product_id = 0;
    loop {
        product_id += 1;
        let product = InventoryItem {
            name: format!("Product {}", product_id),
            quantity: 100,
        };
        println!("Producing: {:?}", product);
        sender.send(product).unwrap();
        thread::sleep(Duration::from_secs(1));
    }
}

pub async fn simulate_consumer() {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://localhost:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_default_executor())
        .await
        .expect("Connection error");
    let channel = conn.create_channel().await.expect("Channel error");

    let consumer = channel
        .basic_consume(
            "inventory_queue",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Consume error");

    println!("Consumer ready to receive messages");

    consumer.set_delegate(move |delivery: Result<Option<Delivery>, lapin::Error>| {
        async move {
            if let Ok(Some(delivery)) = delivery {
                let data: InventoryItem = serde_json::from_slice(&delivery.data).expect("Deserialization error");
                println!("Consumed: {:?}", data);
                delivery.ack(BasicAckOptions::default()).await.expect("Ack error");
            }
        }
    });
}

#[tokio::main]
async fn main() {
    let (sender, _receiver) = unbounded();
    for _ in 0..3 {
        let sender_clone = sender.clone();
        thread::spawn(move || {
            simulate_producer(sender_clone);
        });
    }

    simulate_consumer().await;
}
