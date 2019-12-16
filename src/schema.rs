table! {
    entries (id) {
        id -> Integer,
        start -> Text,
        stop -> Text,
        week_day -> Text,
        code -> Text,
        memo -> Text,
    }
}

table! {
    projects (id) {
        id -> Integer,
        name -> Text,
        code -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    entries,
    projects,
);
