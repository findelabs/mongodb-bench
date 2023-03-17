use clap::Parser;
use env_logger::{Builder, Target};
use chrono::Local;
use std::io::Write;
use log::LevelFilter;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{Document, to_document};
use serde_json::Value;
use metrics_runtime::Receiver;
use metrics_runtime::observers::JsonBuilder;
use metrics_runtime::exporters::LogExporter;
use std::{time::Duration};
use log::Level;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Iterations of test
   #[arg(short, long, default_value_t = 10)]
   iterations: u16,

   /// Default number of threads
   #[arg(short, long, default_value_t = 5)]
   threads: u16,

   /// MongoDB connection string
   #[arg(short, long)]
   url: String,

   /// Query to execute
   #[arg(short, long)]
   query: String,

   /// Database to execute queries against
   #[arg(short, long)]
   database: String,

   /// Collection to execute queries against
   #[arg(short, long)]
   collection: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let args = Args::parse();

    // Initialize log Builder
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{{\"date\": \"{}\", \"level\": \"{}\", \"log\": {}}}",
                Local::now().format("%Y-%m-%dT%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    // Create monitoring
    let receiver = Receiver::builder().build().expect("failed to create receiver");
    let sink = receiver.sink();

    // Generate query Document
    let value: Value = serde_json::from_str(&args.query)?;
    let query = to_document(&value)?;

    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(args.url).await?;
    
    // Manually set an option.
    client_options.app_name = Some("mongo-bench".to_string());
    
    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // Get a handle to a database.
    let db = client.database(&args.database);

    // Execute query
    let collection = db.collection::<Document>(&args.collection);

    // Create vector for task handles
    let mut handles = vec![];

    for t in 0..args.threads {
        log::info!("\"Creating thread {}\"", t);

        // Create clones of all necessary vars
        let coll = collection.clone();
        let query = query.clone();
        let mut sink = sink.clone();

        // Create thread
        handles.push(tokio::spawn(async move {

            // Create query loop
            for i in 0..args.iterations {
                let start = sink.now();
                if let Err(e) = coll.find(query.clone(), None).await {
                    log::error!("\"Error running query: {}\"", e);
                };
                let end = sink.now();
                log::info!("\"Completed query attempt {}, from thread {}, in {}ms\"", i, t, (end - start) / 1000000);
                sink.record_timing("query", start, end);
            }
        }));
    }

    // Wait for all threads to finish
    futures::future::join_all(handles).await;

    // Now create our exporter/observer configuration, and print out results
    LogExporter::new(
        receiver.controller(),
        JsonBuilder::new(),
        Level::Info,
        Duration::from_secs(5),
    ).turn();

    Ok(())
}
