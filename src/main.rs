use clap::Parser;
use env_logger::{Builder, Target};
use chrono::Local;
use std::io::Write;
use log::LevelFilter;
use mongodb::{Client, options::ClientOptions, options::FindOptions};
use mongodb::bson::{Document, to_document};
use serde_json::Value;
use metrics_runtime::Receiver;
use metrics_runtime::observers::JsonBuilder;
use metrics_core::{Builder as MetricsBuilder, Observe, Drain};
use tokio::time::{sleep, Duration};


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

   /// Collation for query
   #[arg(short, long)]
   collation: Option<String>,

   /// Sort for query
   #[arg(short, long)]
   sort: Option<String>,

   /// Database to execute queries against
   #[arg(short, long)]
   database: String,

   /// Collection to execute queries against
   #[arg(short, long)]
   collection: String,

   /// Time to pause between query loops in ms
   #[arg(short, long, default_value_t = 0)]
   pause: u64,

   /// Number of documents to limit response to
   #[arg(short, long, default_value_t = 10)]
   limit: i64
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
//    let sink = receiver.sink();

    // Generate query Document
    let query: Vec<Value> = serde_json::from_str(&args.query)?;
//    let query = to_document(&value)?;

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

    let collation = match args.collation {
        Some(c) => serde_json::from_str(&c).expect("Failed serializing collation value"),
        None => None
    };

    let sort = match args.sort {
        Some(c) => serde_json::from_str(&c).expect("Failed serializing sort value"),
        None => None
    };

    let find_options = FindOptions::builder()
        .limit(args.limit)
        .sort(sort)
        .collation(collation)
        .build();

    for t in 0..args.threads {
        log::info!("\"Creating thread {}\"", t);

        // Create clones of all necessary vars
        let coll = collection.clone();
        let query = query.clone();
        let mut sink = receiver.sink();
        let pause = args.pause;
        let find_options = find_options.clone();

        // Create thread
        handles.push(tokio::spawn(async move {

            // Create query loop
            for i in 0..args.iterations {

                // Loop over query array
                for (c, q) in query.clone().into_iter().enumerate() {
                    let doc = match to_document(&q) {
                        Ok(convert) => convert,
                        Err(e) => {
                            log::error!("Unable to convert query array item into Document: {}", e);
                            continue
                        }
                    };

                    let start = sink.now();

                    // Run find
                    if let Err(e) = coll.find(doc, Some(find_options.clone())).await {
                        log::error!("\"Error running query: {}\"", e);
                    };

                    let end = sink.now();
                    log::info!("\"Completed query attempt {}, within loop {}, from thread {}, in {}ms\"", c + 1, i, t, (end - start) / 1000000);
                    sink.record_timing("query histogram", start, end);
                    sink.increment_counter("query count", 1);
    
                    // Pause between loops
                    sleep(Duration::from_millis(pause)).await;
                }
            }
        }));
    }

    // Wait for all threads to finish
    log::debug!("Waiting for all threads to complete");
    futures::future::join_all(handles).await;
    log::debug!("All threads have completed");

    // Now create our exporter/observer configuration, and print out results
    let mut observer = JsonBuilder::new().build();
    receiver.controller().observe(&mut observer);
    let output = observer.drain();
    log::info!("{}", output);

    Ok(())
}
