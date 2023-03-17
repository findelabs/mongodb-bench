# mongo-bench

A simple app to run simple queries against MongoDB.

```
Usage: mongo-bench [OPTIONS] --url <URL> --query <QUERY> --database <DATABASE> --collection <COLLECTION>

Options:
  -i, --iterations <ITERATIONS>  Iterations of test [default: 10]
  -t, --threads <THREADS>        Default number of threads [default: 5]
  -u, --url <URL>                MongoDB connection string
  -q, --query <QUERY>            Query to execute
  -d, --database <DATABASE>      Database to execute queries against
  -c, --collection <COLLECTION>  Collection to execute queries against
  -h, --help                     Print help
  -V, --version                  Print version
  ````
