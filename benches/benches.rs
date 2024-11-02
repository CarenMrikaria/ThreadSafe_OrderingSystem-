use criterion::{criterion_group, criterion_main, Criterion};
use factory_simulation::{send_inventory_to_rabbitmq, send_customer_to_rabbitmq, send_notification_to_rabbitmq, simulate_producer, simulate_consumer, InventoryItem, Customer};
use crossbeam_channel::unbounded;
use std::thread;
use tokio::runtime::Runtime;

fn benchmark_send_inventory_to_rabbitmq(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let item = InventoryItem {
        name: "Product 1".to_string(),
        quantity: 100,
    };

    c.bench_function("send_inventory_to_rabbitmq", |b| {
        b.to_async(&rt).iter(|| send_inventory_to_rabbitmq(item.clone()))
    });
}

fn benchmark_send_customer_to_rabbitmq(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let customer = Customer {
        name: "Customer 1".to_string(),
        email: "customer1@example.com".to_string(),
    };

    c.bench_function("send_customer_to_rabbitmq", |b| {
        b.to_async(&rt).iter(|| send_customer_to_rabbitmq(customer.clone()))
    });
}

fn benchmark_send_notification_to_rabbitmq(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let notification = "Stock is low!";

    c.bench_function("send_notification_to_rabbitmq", |b| {
        b.to_async(&rt).iter(|| send_notification_to_rabbitmq(notification))
    });
}

fn benchmark_simulate_producer(c: &mut Criterion) {
    let (sender, _receiver) = unbounded();

    c.bench_function("simulate_producer", |b| {
        b.iter(|| {
            let sender_clone = sender.clone();
            thread::spawn(move || {
                simulate_producer(sender_clone);
            }).join().unwrap();
        });
    });
}

fn benchmark_simulate_consumer(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("simulate_consumer", |b| {
        b.to_async(&rt).iter(|| simulate_consumer())
    });
}

fn benchmark_end_to_end_simulation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("end_to_end_simulation", |b| {
        b.to_async(&rt).iter(|| async {
            let (sender, _receiver) = unbounded();
            for _ in 0..3 {
                let sender_clone = sender.clone();
                thread::spawn(move || {
                    simulate_producer(sender_clone);
                });
            }
            simulate_consumer().await;
        })
    });
}

criterion_group!(
    benches,
    benchmark_send_inventory_to_rabbitmq,
    benchmark_send_customer_to_rabbitmq,
    benchmark_send_notification_to_rabbitmq,
    benchmark_simulate_producer,
    benchmark_simulate_consumer,
    benchmark_end_to_end_simulation,
);
criterion_main!(benches);
