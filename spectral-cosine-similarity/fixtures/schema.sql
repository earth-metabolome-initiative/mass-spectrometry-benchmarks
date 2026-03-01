-- Pure algorithm concepts (implementation-agnostic)
CREATE TABLE IF NOT EXISTS algorithms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT
) STRICT;

-- Libraries / packages
CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    git_commit TEXT,
    git_url TEXT,
    language TEXT NOT NULL,
    UNIQUE(name, version)
) STRICT;

-- An algorithm implemented in a specific library
CREATE TABLE IF NOT EXISTS implementations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    algorithm_id INTEGER NOT NULL REFERENCES algorithms(id),
    library_id INTEGER NOT NULL REFERENCES libraries(id),
    UNIQUE(algorithm_id, library_id)
) STRICT;

-- Experiment parameter sets (JSON blob for flexibility)
CREATE TABLE IF NOT EXISTS experiments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    params TEXT NOT NULL,
    UNIQUE(params)
) STRICT;

-- Spectra
CREATE TABLE IF NOT EXISTS spectra (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    raw_name TEXT NOT NULL,
    source_file TEXT NOT NULL,
    spectrum_hash TEXT NOT NULL UNIQUE,
    precursor_mz REAL NOT NULL,
    num_peaks INTEGER NOT NULL,
    peaks TEXT NOT NULL
) STRICT;

-- Results: similarity score + timing, one row per (pair, experiment, implementation)
CREATE TABLE IF NOT EXISTS results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    left_id INTEGER NOT NULL REFERENCES spectra(id),
    right_id INTEGER NOT NULL REFERENCES spectra(id),
    experiment_id INTEGER NOT NULL REFERENCES experiments(id),
    implementation_id INTEGER NOT NULL REFERENCES implementations(id),
    score REAL NOT NULL,
    matches INTEGER NOT NULL,
    median_time_us REAL NOT NULL,
    UNIQUE(left_id, right_id, experiment_id, implementation_id)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_results_impl ON results(implementation_id);
CREATE INDEX IF NOT EXISTS idx_results_left ON results(left_id);
CREATE INDEX IF NOT EXISTS idx_results_right ON results(right_id);
