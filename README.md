# supabase-realtime-rs

Supabase Realtime client for Rust.

Filter semantics are similar to [`postgrest-rs`](https://github.com/supabase-community/postgrest-rs).

## Using

```rust
// Build a client.
let client = Client::builder()
    .params()
    .build();

// Create a connection.
let conn = client.connect().await?;

// Subscribe to listen for an event.
let channel = conn.
    .on()
    .subscribe().await?;
```

## Setting up development

Add the subdomain to your `/etc/hosts`.

```
127.0.0.1       realtime-dev.localhost
```

Spin up the realtime server container and database

```shell
docker-compose up -d
```

Add the development tenant via the realtime API

```shell
curl -X POST \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiIiLCJpYXQiOjE2NzEyMzc4NzMsImV4cCI6MTcwMjc3Mzk5MywiYXVkIjoiIiwic3ViIjoiIn0._ARixa2KFUVsKBf3UGR90qKLCpGjxhKcXY4akVbmeNQ' \
  -d $'{
    "tenant" : {
      "name": "realtime-dev",
      "external_id": "realtime-dev",
      "jwt_secret": "a1d99c8b-91b6-47b2-8f3c-aa7d9a9ad20f",
      "extensions": [
        {
          "type": "postgres_cdc_rls",
          "settings": {
            "db_name": "postgres",
            "db_host": "host.docker.internal",
            "db_user": "postgres",
            "db_password": "postgres",
            "db_port": "5432",
            "region": "us-west-1",
            "poll_interval_ms": 100,
            "poll_max_record_bytes": 1048576,
            "ip_version": 4
          }
        }
      ]
    }
  }' \
  http://localhost:4000/api/tenants
```
