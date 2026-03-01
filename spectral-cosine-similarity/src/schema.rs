diesel::table! {
    algorithms (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        approximates_algorithm_id -> Nullable<Integer>,
    }
}

diesel::table! {
    libraries (id) {
        id -> Integer,
        name -> Text,
        version -> Text,
        git_commit -> Nullable<Text>,
        git_url -> Nullable<Text>,
        language -> Text,
    }
}

diesel::table! {
    implementations (id) {
        id -> Integer,
        algorithm_id -> Integer,
        library_id -> Integer,
        is_reference -> Bool,
    }
}

diesel::table! {
    experiments (id) {
        id -> Integer,
        params -> Text,
    }
}

diesel::table! {
    spectra (id) {
        id -> Integer,
        name -> Text,
        raw_name -> Text,
        source_file -> Text,
        spectrum_hash -> Text,
        precursor_mz -> Double,
        num_peaks -> Integer,
        peaks -> Text,
    }
}

diesel::table! {
    results (id) {
        id -> Integer,
        left_id -> Integer,
        right_id -> Integer,
        experiment_id -> Integer,
        implementation_id -> Integer,
        score -> Double,
        matches -> Integer,
        median_time_us -> Double,
    }
}

diesel::joinable!(implementations -> algorithms (algorithm_id));
diesel::joinable!(implementations -> libraries (library_id));
diesel::joinable!(results -> experiments (experiment_id));
diesel::joinable!(results -> implementations (implementation_id));

diesel::allow_tables_to_appear_in_same_query!(
    algorithms,
    libraries,
    implementations,
    experiments,
    spectra,
    results,
);
