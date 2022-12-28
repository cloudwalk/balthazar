# Balthazar

Balthazar is a meta-crate for many of the [CloudWalk](https://cloudwalk.io)
internal projects, providing a unified way to set up services and dependencies
such as telemetry and database access.

## Cargo Features

* `postgres`: Enables PostgreSQL support with pooling, using `sqlx`.
* `redis`: Enables Redis support with pooling, using `bb8`.

## Environment Variables Reference

| Variable Name                    | Default Value                       | Description                                                                                |
|----------------------------------|-------------------------------------|--------------------------------------------------------------------------------------------|
| `NO_COLOR`                       | `false`                             | Set to `true` to disable all terminal colors.                                              |
| `TRACING_DISABLE_OPENTELEMETRY`  | `false`                             | Set to `true` to disable exporting OpenTelemetry metrics and traces to a collector.        |
| `TRACING_OPENTELEMETRY_ENDPOINT` | `http://localhost:14268/api/traces` | The endpoint to the OpenTelemetry collector.                                               |
| `TRACING_LOG_LEVEL`              | `debug`                             | Log level filter, do not show messages with lower priority than this.                      |
| `TRACING_FORMAT`                 | `text-pretty`                       | `json` or `json-pretty` for JSON. `text` or `text-pretty` for formatted text. |
| `POSTGRES_URL`                   | -                                   | Connection string for the PostgreSQL instance.                                             |
| `POSTGRES_MAX_CONNECTIONS`       | `8`                                 | Maximum amount of concurrent connections to the SQL instance.                              |
| `REDIS_URL`                      | -                                   | Connection string to the Redis instance.                                                   |
