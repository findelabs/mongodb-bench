# mongo-bench

A simple app to run simple queries against MongoDB.


## Usage
```
Usage: mongodb-bench [OPTIONS] --url <URL> --query <QUERY> --database <DATABASE> --collection <COLLECTION>

Options:
  -i, --iterations <ITERATIONS>  Iterations of test [default: 10]
  -t, --threads <THREADS>        Default number of threads [default: 5]
  -u, --url <URL>                MongoDB connection string
  -q, --query <QUERY>            Query to execute
  -d, --database <DATABASE>      Database to execute queries against
  -c, --collection <COLLECTION>  Collection to execute queries against
  -p, --pause <PAUSE>            Time to pause between query loops in ms [default: 0]
  -l, --limit <LIMIT>            Number of documents to limit response to [default: 10]
  -h, --help                     Print help
  -V, --version                  Print version
  ````

### Example: 

This example benchmark will run 100 threads, with 100 iterations, over the query `{"module_setup":true}`:

```
cargo run --release -- --url 'mongodb+srv://<username>:<password>@cluster1-prd-rs.cd1c4x.mongodb.net/?retryWrites=true&w=majority' --query '[{"module_setup":true}]' --database my_database --collection my_collection --iterations 100 --threads 100 --limit 1
```

## Output

The final log output will display a histogram for all queries hitting mongo, in ns:
```
{
  "date": "2023-03-17T15:19:18:095755290",
  "level": "INFO",
  "log": {
    "query count": 1000,
    "query histogram count": 683,
    "query histogram max": 223739903,
    "query histogram min": 48037888,
    "query histogram p50": 154533887,
    "query histogram p90": 192544767,
    "query histogram p95": 199360511,
    "query histogram p99": 213909503,
    "query histogram p999": 223739903
  }
}
```
