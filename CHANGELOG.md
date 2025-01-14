# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.2.10] - 2022-12-30

- Add region config parameter for Amazon S3 object store
  (<https://github.com/splitgraph/seafowl/pull/255>)
- Enable querying external Delta tables in Seafowl
  (<https://github.com/splitgraph/seafowl/pull/252>)
- Implement remote table factory (<https://github.com/splitgraph/seafowl/pull/250>)

## [0.2.9] - 2022-12-23

- Add support for pushdown in remote tables:
  - `LIMIT` (<https://github.com/splitgraph/seafowl/pull/221>)
  - `WHERE` (<https://github.com/splitgraph/seafowl/pull/226>,
    <https://github.com/splitgraph/seafowl/pull/235>)
- Factor out remote tables into a separate crate (<https://github.com/splitgraph/seafowl/pull/238>)
- Upgrade to DataFusion 15 (<https://github.com/splitgraph/seafowl/pull/248>)

## [0.2.8] - 2022-11-21

- Implement `table_partitions` system table (<https://github.com/splitgraph/seafowl/pull/214>)
- Add WASI + MessagePack UDF language variant (<https://github.com/splitgraph/seafowl/pull/149>)

## [0.2.7] - 2022-11-17

- Import JSON values as strings in `CREATE EXTERNAL TABLE`
  (<https://github.com/splitgraph/seafowl/pull/208>)
- Add support for SQLite in `CREATE EXTERNAL TABLE`
  (<https://github.com/splitgraph/seafowl/pull/200>)

## [0.2.6] - 2022-11-08

- Update to DataFusion 14 / Arrow 26 (<https://github.com/splitgraph/seafowl/pull/198>)
- Bugfix: `VACUUM` with shared partitions (<https://github.com/splitgraph/seafowl/pull/191>)
- Bugfix: `DELETE` with certain filters that cover a whole partition
  (<https://github.com/splitgraph/seafowl/pull/192>)
- Initial support for other databases in `CREATE EXTERNAL TABLE`
  (<https://github.com/splitgraph/seafowl/pull/190>)
  - More documentation pending. Example:
    `CREATE EXTERNAL TABLE t STORED AS TABLE 'public.t' LOCATION 'postgresql://uname:pw@host:port/dbname'`

## [0.2.5] - 2022-11-02

- Upgrade to DataFusion 13 (784f10bb) / Arrow 25.0.0
  (<https://github.com/splitgraph/seafowl/pull/176>)
- Use ZSTD compression in Parquet files (<https://github.com/splitgraph/seafowl/pull/182>)
- Fix HTTP external tables using pre-signed S3 URLs
  (<https://github.com/splitgraph/seafowl/pull/183>)
- Fix `INSERT INTO .. SELECT FROM` (<https://github.com/splitgraph/seafowl/pull/184>)
- Fix some `OUTER JOIN` issues by using a minimum of 2 `target_partition`s
  (<https://github.com/splitgraph/seafowl/pull/189>)

## [0.2.4] - 2022-10-25

- Add `system.table_versions` table for inspecting table history
  (<https://github.com/splitgraph/seafowl/pull/168>)
- Add SQLite `catalog.read_only` option for compatibility with LiteFS replicas
  (<https://github.com/splitgraph/seafowl/pull/171>)

## [0.2.3] - 2022-10-21

- Add support for time travel queries (`SELECT * FROM table('2022-01-01T00:00:00')`)
  (<https://github.com/splitgraph/seafowl/pull/154>)
- Allow overriding SQLite journal mode (<https://github.com/splitgraph/seafowl/pull/158>)

## [0.2.2] - 2022-10-12

- Allow using SQL types in WASM UDF definitions (<https://github.com/splitgraph/seafowl/pull/147>)

## [0.2.1] - 2022-09-30

- Cached GET API now accepts URL-encoded query text in X-Seafowl-Header
  (<https://github.com/splitgraph/seafowl/pull/122>)
- Add support for `DELETE` statements (<https://github.com/splitgraph/seafowl/pull/121>)
- Add support for `UPDATE` statements (<https://github.com/splitgraph/seafowl/pull/127>)

## [0.2.0] - 2022-09-21

**Breaking**: Previous versions of Seafowl won't be able to read data written by this version.

- Fix storage of nullable / list columns (<https://github.com/splitgraph/seafowl/pull/119>)
- Default columns in `CREATE TABLE` to nullable (<https://github.com/splitgraph/seafowl/pull/119>)

## [0.1.1] - 2022-09-16

- Upgrade to DataFusion 12 (<https://github.com/splitgraph/seafowl/pull/113>)

## [0.1.0] - 2022-09-12

### Fixes

- Use multi-part uploads to fix the memory usage issue when uploading data to S3
  (<https://github.com/splitgraph/seafowl/pull/99>)

<!-- next-url -->

[unreleased]: https://github.com/splitgraph/seafowl/compare/v0.2.10...HEAD
[0.2.10]: https://github.com/splitgraph/seafowl/compare/v0.2.10...v0.2.10
[0.2.10]: https://github.com/splitgraph/seafowl/compare/v0.2.9...v0.2.10
[0.2.9]: https://github.com/splitgraph/seafowl/compare/v0.2.8...v0.2.9
[0.2.8]: https://github.com/splitgraph/seafowl/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/splitgraph/seafowl/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/splitgraph/seafowl/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/splitgraph/seafowl/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/splitgraph/seafowl/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/splitgraph/seafowl/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/splitgraph/seafowl/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/splitgraph/seafowl/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/splitgraph/seafowl/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/splitgraph/seafowl/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/splitgraph/seafowl/compare/v0.1.0-dev.4...v0.1.0
