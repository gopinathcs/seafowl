use crate::statements::*;

#[tokio::test]
async fn test_information_schema() {
    let context = make_context_with_pg().await;

    let plan = context
        .plan_query(
            "SELECT * FROM information_schema.tables ORDER BY table_catalog, table_name",
        )
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+---------------+--------------------+------------------+------------+",
        "| table_catalog | table_schema       | table_name       | table_type |",
        "+---------------+--------------------+------------------+------------+",
        "| default       | information_schema | columns          | VIEW       |",
        "| default       | information_schema | df_settings      | VIEW       |",
        "| default       | system             | table_partitions | VIEW       |",
        "| default       | system             | table_versions   | VIEW       |",
        "| default       | information_schema | tables           | VIEW       |",
        "| default       | information_schema | views            | VIEW       |",
        "+---------------+--------------------+------------------+------------+",
    ];

    assert_batches_eq!(expected, &results);

    let plan = context
        .plan_query(
            format!(
                "SELECT table_schema, table_name, column_name, data_type, is_nullable
        FROM information_schema.columns
        WHERE table_schema = '{SYSTEM_SCHEMA}'
        ORDER BY table_name, ordinal_position",
            )
            .as_str(),
        )
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+--------------+------------------+--------------------+-------------------------+-------------+",
        "| table_schema | table_name       | column_name        | data_type               | is_nullable |",
        "+--------------+------------------+--------------------+-------------------------+-------------+",
        "| system       | table_partitions | table_schema       | Utf8                    | NO          |",
        "| system       | table_partitions | table_name         | Utf8                    | NO          |",
        "| system       | table_partitions | table_version_id   | Int64                   | NO          |",
        "| system       | table_partitions | table_partition_id | Int64                   | YES         |",
        "| system       | table_partitions | object_storage_id  | Utf8                    | YES         |",
        "| system       | table_partitions | row_count          | Int32                   | YES         |",
        "| system       | table_versions   | table_schema       | Utf8                    | NO          |",
        "| system       | table_versions   | table_name         | Utf8                    | NO          |",
        "| system       | table_versions   | table_version_id   | Int64                   | NO          |",
        "| system       | table_versions   | creation_time      | Timestamp(Second, None) | NO          |",
        "+--------------+------------------+--------------------+-------------------------+-------------+",
    ];
    assert_batches_eq!(expected, &results);
}

#[tokio::test]
async fn test_create_table_and_insert() {
    let context = make_context_with_pg().await;

    // TODO: insert into nonexistent table outputs a wrong error (schema "public" does not exist)
    create_table_and_insert(&context, "test_table").await;

    // Check table columns: make sure scanning through our file pads the rest with NULLs
    let plan = context
        .plan_query("SELECT * FROM test_table")
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+-----------------+----------------+------------------+---------------------+------------+",
        "| some_bool_value | some_int_value | some_other_value | some_time           | some_value |",
        "+-----------------+----------------+------------------+---------------------+------------+",
        "|                 | 1111           |                  | 2022-01-01T20:01:01 | 42         |",
        "|                 | 2222           |                  | 2022-01-01T20:02:02 | 43         |",
        "|                 | 3333           |                  | 2022-01-01T20:03:03 | 44         |",
        "+-----------------+----------------+------------------+---------------------+------------+",
    ];

    assert_batches_eq!(expected, &results);

    // Test some projections and aggregations
    let plan = context
        .plan_query("SELECT MAX(some_time) FROM test_table")
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+---------------------------+",
        "| MAX(test_table.some_time) |",
        "+---------------------------+",
        "| 2022-01-01T20:03:03       |",
        "+---------------------------+",
    ];

    assert_batches_eq!(expected, &results);

    let plan = context
        .plan_query("SELECT MAX(some_int_value), COUNT(DISTINCT some_bool_value), MAX(some_value) FROM test_table")
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+--------------------------------+--------------------------------------------+----------------------------+",
        "| MAX(test_table.some_int_value) | COUNT(DISTINCT test_table.some_bool_value) | MAX(test_table.some_value) |",
        "+--------------------------------+--------------------------------------------+----------------------------+",
        "| 3333                           | 0                                          | 44                         |",
        "+--------------------------------+--------------------------------------------+----------------------------+",
    ];

    assert_batches_eq!(expected, &results);
}

#[tokio::test]
async fn test_table_time_travel() {
    let context = make_context_with_pg().await;
    let (version_results, version_timestamps) = create_table_and_some_partitions(
        &context,
        "test_table",
        Some(Duration::from_secs(1)),
    )
    .await;

    let timestamp_to_rfc3339 = |timestamp: Timestamp| -> String {
        Utc.timestamp_opt(timestamp, 0).unwrap().to_rfc3339()
    };

    //
    // Verify that the new table versions are shown in the corresponding system table
    //

    let plan = context
        .plan_query("SELECT table_schema, table_name, table_version_id FROM system.table_versions")
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+--------------+------------+------------------+",
        "| table_schema | table_name | table_version_id |",
        "+--------------+------------+------------------+",
        "| public       | test_table | 1                |",
        "| public       | test_table | 2                |",
        "| public       | test_table | 3                |",
        "| public       | test_table | 4                |",
        "| public       | test_table | 5                |",
        "+--------------+------------+------------------+",
    ];
    assert_batches_eq!(expected, &results);

    //
    // Test that filtering the system table works, given that we provide all rows to DF and expect
    // it to do it.
    //
    let plan = context
        .plan_query(
            format!(
                "
            SELECT table_version_id FROM system.table_versions \
            WHERE table_version_id < 5 AND creation_time > to_timestamp('{}')
        ",
                timestamp_to_rfc3339(version_timestamps[&2])
            )
            .as_str(),
        )
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+------------------+",
        "| table_version_id |",
        "+------------------+",
        "| 3                |",
        "| 4                |",
        "+------------------+",
    ];
    assert_batches_eq!(expected, &results);

    //
    // Verify that the new table partitions for all versions are shown in the corresponding system table
    //

    let plan = context
        .plan_query("SELECT table_schema, table_name, table_version_id, table_partition_id, row_count FROM system.table_partitions")
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+--------------+------------+------------------+--------------------+-----------+",
        "| table_schema | table_name | table_version_id | table_partition_id | row_count |",
        "+--------------+------------+------------------+--------------------+-----------+",
        "| public       | test_table | 1                |                    |           |",
        "| public       | test_table | 2                | 1                  | 3         |",
        "| public       | test_table | 3                | 1                  | 3         |",
        "| public       | test_table | 3                | 2                  | 3         |",
        "| public       | test_table | 4                | 1                  | 3         |",
        "| public       | test_table | 4                | 2                  | 3         |",
        "| public       | test_table | 4                | 3                  | 3         |",
        "| public       | test_table | 5                | 1                  | 3         |",
        "| public       | test_table | 5                | 2                  | 3         |",
        "| public       | test_table | 5                | 3                  | 3         |",
        "| public       | test_table | 5                | 4                  | 3         |",
        "+--------------+------------+------------------+--------------------+-----------+",
    ];
    assert_batches_eq!(expected, &results);

    //
    // Now use the recorded timestamps to query specific earlier table versions and compare them to
    // the recorded results for that version.
    //

    async fn query_table_version(
        context: &DefaultSeafowlContext,
        version_id: TableVersionId,
        version_results: &HashMap<TableVersionId, Vec<RecordBatch>>,
        version_timestamps: &HashMap<TableVersionId, Timestamp>,
        timestamp_converter: fn(Timestamp) -> String,
    ) {
        let plan = context
            .plan_query(
                format!(
                    "SELECT * FROM test_table('{}')",
                    timestamp_converter(version_timestamps[&version_id])
                )
                .as_str(),
            )
            .await
            .unwrap();
        let results = context.collect(plan).await.unwrap();

        assert_eq!(version_results[&version_id], results);
    }

    for version_id in [2, 3, 4, 5] {
        query_table_version(
            &context,
            version_id as TableVersionId,
            &version_results,
            &version_timestamps,
            timestamp_to_rfc3339,
        )
        .await;
    }

    //
    // Try to query a non-existent version (timestamp older than the oldest version)
    //

    let err = context
        .plan_query("SELECT * FROM test_table('2012-12-21 20:12:21 +00:00')")
        .await
        .unwrap_err();

    assert!(err
        .to_string()
        .contains("No recorded table versions for the provided timestamp"));

    //
    // Use multiple different version specifiers in the same complex query (including the latest
    // version both explicitly and in the default notation).
    // Ensures row differences between different versions are consistent:
    // 5 - ((5 - 4) + (4 - 3) + (3 - 2)) = 2
    //

    let plan = context
        .plan_query(
            format!(
                r#"
                WITH diff_3_2 AS (
                    SELECT * FROM test_table('{}')
                    EXCEPT
                    SELECT * FROM test_table('{}')
                ), diff_4_3 AS (
                    SELECT * FROM test_table('{}')
                    EXCEPT
                    SELECT * FROM test_table('{}')
                ), diff_5_4 AS (
                    SELECT * FROM test_table('{}')
                    EXCEPT
                    SELECT * FROM test_table('{}')
                )
                SELECT * FROM test_table
                EXCEPT (
                    SELECT * FROM diff_5_4
                    UNION
                    SELECT * FROM diff_4_3
                    UNION
                    SELECT * FROM diff_3_2
                )
                ORDER BY some_int_value
            "#,
                timestamp_to_rfc3339(version_timestamps[&3]),
                timestamp_to_rfc3339(version_timestamps[&2]),
                timestamp_to_rfc3339(version_timestamps[&4]),
                timestamp_to_rfc3339(version_timestamps[&3]),
                timestamp_to_rfc3339(version_timestamps[&5]),
                timestamp_to_rfc3339(version_timestamps[&4]),
            )
            .as_str(),
        )
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();
    assert_eq!(version_results[&2], results);

    // Ensure the context table map contains the versioned + the latest table entries
    assert_eq!(
        sorted(
            context
                .inner()
                .state
                .read()
                .catalog_list
                .catalog(DEFAULT_DB)
                .unwrap()
                .schema(DEFAULT_SCHEMA)
                .unwrap()
                .table_names()
        )
        .collect::<Vec<String>>(),
        vec![
            "test_table".to_string(),
            "test_table:2".to_string(),
            "test_table:3".to_string(),
            "test_table:4".to_string(),
        ],
    );

    //
    // Verify that information schema is not polluted with versioned tables/columns
    //

    let results = list_tables_query(&context).await;

    let expected = vec![
        "+--------------------+-------------+",
        "| table_schema       | table_name  |",
        "+--------------------+-------------+",
        "| information_schema | columns     |",
        "| information_schema | df_settings |",
        "| information_schema | tables      |",
        "| information_schema | views       |",
        "| public             | test_table  |",
        "+--------------------+-------------+",
    ];
    assert_batches_eq!(expected, &results);

    let results = list_columns_query(&context).await;

    let expected = vec![
        "+--------------+------------+------------------+-----------------------------+",
        "| table_schema | table_name | column_name      | data_type                   |",
        "+--------------+------------+------------------+-----------------------------+",
        "| public       | test_table | some_bool_value  | Boolean                     |",
        "| public       | test_table | some_int_value   | Int64                       |",
        "| public       | test_table | some_other_value | Decimal128(38, 10)          |",
        "| public       | test_table | some_time        | Timestamp(Nanosecond, None) |",
        "| public       | test_table | some_value       | Float32                     |",
        "+--------------+------------+------------------+-----------------------------+",
    ];
    assert_batches_eq!(expected, &results);
}

#[cfg(feature = "remote-tables")]
#[rstest]
#[case::postgres_schema_introspected("Postgres", true)]
#[case::postgres_schema_declared("Postgres", false)]
#[case::sqlite_schema_introspected("SQLite", true)]
#[case::sqlite_schema_declared("SQLite", false)]
#[tokio::test]
async fn test_remote_table_querying(
    #[case] db_type: &str,
    #[case] introspect_schema: bool,
) {
    let context = make_context_with_pg().await;

    let schema = get_random_schema();
    let _temp_path: TempPath;
    let (dsn, table_name) = if db_type == "Postgres" {
        (
            env::var("DATABASE_URL").unwrap(),
            format!("{schema}.\"source table\""),
        )
    } else {
        // SQLite
        let temp_file = NamedTempFile::new().unwrap();
        let dsn = temp_file.path().to_string_lossy().to_string();
        // We need the temp file to outlive this scope, so we must open a path ref to it
        _temp_path = temp_file.into_temp_path();
        (format!("sqlite://{dsn}"), "\"source table\"".to_string())
    };
    let pool = AnyPool::connect(dsn.as_str()).await.unwrap();

    if db_type == "Postgres" {
        pool.execute(format!("CREATE SCHEMA {schema}").as_str())
            .await
            .unwrap();
    }

    //
    // Create a table in our metadata store, and insert some dummy data
    //
    pool.execute(
            format!(
                "CREATE TABLE {table_name} (a INT, b FLOAT, c VARCHAR, \"date field\" DATE, e TIMESTAMP, f JSON)"
            )
            .as_str(),
        )
        .await
        .unwrap();
    pool.execute(
        format!(
            "INSERT INTO {table_name} VALUES \
            (1, 1.1, 'one', '2022-11-01', '2022-11-01 22:11:01', '{{\"rows\":[1]}}'),\
            (2, 2.22, 'two', '2022-11-02', '2022-11-02 22:11:02', '{{\"rows\":[1,2]}}'),\
            (3, 3.333, 'three', '2022-11-03', '2022-11-03 22:11:03', '{{\"rows\":[1,2,3]}}'),\
            (4, 4.4444, 'four', '2022-11-04', '2022-11-04 22:11:04', '{{\"rows\":[1,2,3,4]}}')"
        )
        .as_str(),
    )
    .await
    .unwrap();

    let table_column_schema = if introspect_schema {
        ""
    } else {
        "(a INT, b FLOAT, c VARCHAR, \"date field\" DATE, e TIMESTAMP, f TEXT)"
    };

    //
    // Create a remote table (pointed at our metadata store table)
    //
    let plan = context
        .plan_query(
            format!(
                "CREATE EXTERNAL TABLE remote_table {table_column_schema}
                STORED AS TABLE
                OPTIONS ('name' '{table_name}')
                LOCATION '{dsn}'"
            )
            .as_str(),
        )
        .await
        .unwrap();
    context.collect(plan).await.unwrap();

    //
    // Verify column types in information schema
    //
    let results = list_columns_query(&context).await;

    let expected = if introspect_schema {
        vec![
            "+--------------+--------------+-------------+-----------+",
            "| table_schema | table_name   | column_name | data_type |",
            "+--------------+--------------+-------------+-----------+",
            "| staging      | remote_table | a           | Int64     |",
            "| staging      | remote_table | b           | Float64   |",
            "| staging      | remote_table | c           | Utf8      |",
            "| staging      | remote_table | date field  | Date32    |",
            "| staging      | remote_table | e           | Date64    |",
            "| staging      | remote_table | f           | Utf8      |",
            "+--------------+--------------+-------------+-----------+",
        ]
    } else {
        vec![
            "+--------------+--------------+-------------+-----------------------------+",
            "| table_schema | table_name   | column_name | data_type                   |",
            "+--------------+--------------+-------------+-----------------------------+",
            "| staging      | remote_table | a           | Int32                       |",
            "| staging      | remote_table | b           | Float32                     |",
            "| staging      | remote_table | c           | Utf8                        |",
            "| staging      | remote_table | date field  | Date32                      |",
            "| staging      | remote_table | e           | Timestamp(Nanosecond, None) |",
            "| staging      | remote_table | f           | Utf8                        |",
            "+--------------+--------------+-------------+-----------------------------+",
        ]
    };
    assert_batches_eq!(expected, &results);

    //
    // Query remote table
    //
    let plan = context
        .plan_query("SELECT * FROM staging.remote_table")
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = if introspect_schema {
        // Connector-X coerces the TIMESTAMP field to Date64, but that data type still has
        // millisecond resolution so why isn't it being shown?
        vec![
            "+---+--------+-------+------------+------------+--------------------+",
            "| a | b      | c     | date field | e          | f                  |",
            "+---+--------+-------+------------+------------+--------------------+",
            "| 1 | 1.1    | one   | 2022-11-01 | 2022-11-01 | {\"rows\":[1]}       |",
            "| 2 | 2.22   | two   | 2022-11-02 | 2022-11-02 | {\"rows\":[1,2]}     |",
            "| 3 | 3.333  | three | 2022-11-03 | 2022-11-03 | {\"rows\":[1,2,3]}   |",
            "| 4 | 4.4444 | four  | 2022-11-04 | 2022-11-04 | {\"rows\":[1,2,3,4]} |",
            "+---+--------+-------+------------+------------+--------------------+",
        ]
    } else {
        vec![
            "+---+--------+-------+------------+---------------------+--------------------+",
            "| a | b      | c     | date field | e                   | f                  |",
            "+---+--------+-------+------------+---------------------+--------------------+",
            "| 1 | 1.1    | one   | 2022-11-01 | 2022-11-01T22:11:01 | {\"rows\":[1]}       |",
            "| 2 | 2.22   | two   | 2022-11-02 | 2022-11-02T22:11:02 | {\"rows\":[1,2]}     |",
            "| 3 | 3.333  | three | 2022-11-03 | 2022-11-03T22:11:03 | {\"rows\":[1,2,3]}   |",
            "| 4 | 4.4444 | four  | 2022-11-04 | 2022-11-04T22:11:04 | {\"rows\":[1,2,3,4]} |",
            "+---+--------+-------+------------+---------------------+--------------------+",
        ]
    };
    assert_batches_eq!(expected, &results);

    // Test that projection and filtering work
    let plan = context
        .plan_query(
            "SELECT \"date field\", c FROM staging.remote_table \
            WHERE (\"date field\" > '2022-11-01' OR c = 'two') \
            AND (a > 2 OR e < to_timestamp('2022-11-04 22:11:05')) LIMIT 2",
        )
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+------------+-------+",
        "| date field | c     |",
        "+------------+-------+",
        "| 2022-11-02 | two   |",
        "| 2022-11-03 | three |",
        "+------------+-------+",
    ];
    assert_batches_eq!(expected, &results);

    // Ensure pushdown of WHERE and LIMIT clause shows up in the plan
    let plan = context
        .plan_query(
            "EXPLAIN SELECT \"date field\", c FROM staging.remote_table \
            WHERE (\"date field\" > '2022-11-01' OR c = 'two') \
            AND (a > 2 OR e < to_timestamp('2022-11-04 22:11:05')) LIMIT 2",
        )
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = if introspect_schema {
        vec![
            "+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
            "| plan_type     | plan                                                                                                                                                                                                                                                                                               |",
            "+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
            "| logical_plan  | Projection: staging.remote_table.date field, staging.remote_table.c                                                                                                                                                                                                                                |",
            "|               |   Limit: skip=0, fetch=2                                                                                                                                                                                                                                                                           |",
            "|               |     TableScan: staging.remote_table projection=[c, date field], full_filters=[staging.remote_table.date field > Utf8(\"2022-11-01\") OR staging.remote_table.c = Utf8(\"two\"), staging.remote_table.a > Int64(2) OR staging.remote_table.e < TimestampNanosecond(1667599865000000000, None)], fetch=2 |",
            "| physical_plan | ProjectionExec: expr=[date field@1 as date field, c@0 as c]                                                                                                                                                                                                                                        |",
            "|               |   GlobalLimitExec: skip=0, fetch=2                                                                                                                                                                                                                                                                 |",
            "|               |     MemoryExec: partitions=1, partition_sizes=[1]                                                                                                                                                                                                                                                  |",
            "|               |                                                                                                                                                                                                                                                                                                    |",
            "+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
        ]
    } else {
        vec![
            "+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
            "| plan_type     | plan                                                                                                                                                                                                                                                                                            |",
            "+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
            "| logical_plan  | Projection: staging.remote_table.date field, staging.remote_table.c                                                                                                                                                                                                                             |",
            "|               |   Limit: skip=0, fetch=2                                                                                                                                                                                                                                                                        |",
            "|               |     TableScan: staging.remote_table projection=[c, date field], full_filters=[staging.remote_table.date field > Date32(\"19297\") OR staging.remote_table.c = Utf8(\"two\"), staging.remote_table.a > Int32(2) OR staging.remote_table.e < TimestampNanosecond(1667599865000000000, None)], fetch=2 |",
            "| physical_plan | ProjectionExec: expr=[date field@1 as date field, c@0 as c]                                                                                                                                                                                                                                     |",
            "|               |   GlobalLimitExec: skip=0, fetch=2                                                                                                                                                                                                                                                              |",
            "|               |     ProjectionExec: expr=[c@0 as c, date field@1 as date field]                                                                                                                                                                                                                                 |",
            "|               |       MemoryExec: partitions=1, partition_sizes=[1]                                                                                                                                                                                                                                             |",
            "|               |                                                                                                                                                                                                                                                                                                 |",
            "+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
        ]
    };
    assert_batches_eq!(expected, &results);
}

#[cfg(feature = "delta-tables")]
#[tokio::test]
async fn test_delta_tables() {
    let context = make_context_with_pg().await;

    let plan = context
        .plan_query(
            "CREATE EXTERNAL TABLE test_delta \
            STORED AS DELTATABLE \
            LOCATION 'tests/data/delta-0.8.0-partitioned'",
        )
        .await
        .unwrap();
    context.collect(plan).await.unwrap();

    // The order gets randomized so we need to enforce it
    let plan = context
        .plan_query("SELECT * FROM staging.test_delta ORDER BY value")
        .await
        .unwrap();
    let results = context.collect(plan).await.unwrap();

    let expected = vec![
        "+-------+------+-------+-----+",
        "| value | year | month | day |",
        "+-------+------+-------+-----+",
        "| 1     | 2020 | 1     | 1   |",
        "| 2     | 2020 | 2     | 3   |",
        "| 3     | 2020 | 2     | 5   |",
        "| 4     | 2021 | 4     | 5   |",
        "| 5     | 2021 | 12    | 4   |",
        "| 6     | 2021 | 12    | 20  |",
        "| 7     | 2021 | 12    | 20  |",
        "+-------+------+-------+-----+",
    ];
    assert_batches_eq!(expected, &results);
}
