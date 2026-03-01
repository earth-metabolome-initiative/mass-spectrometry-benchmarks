-- Pure algorithm concepts (implementation-agnostic)
CREATE TABLE IF NOT EXISTS algorithms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    approximates_algorithm_id INTEGER REFERENCES algorithms(id)
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
    is_reference INTEGER NOT NULL DEFAULT 0 CHECK (is_reference IN (0, 1)),
    UNIQUE(algorithm_id, library_id)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_implementations_one_reference_per_algorithm
ON implementations(algorithm_id)
WHERE is_reference = 1;

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
CREATE INDEX IF NOT EXISTS idx_results_impl_pair_exp
ON results(implementation_id, left_id, right_id, experiment_id);
CREATE INDEX IF NOT EXISTS idx_results_pair_exp_impl
ON results(left_id, right_id, experiment_id, implementation_id);

-- Canonical/reference topology derived from schema regularities.
CREATE VIEW IF NOT EXISTS v_implementation_topology AS
SELECT i.id AS implementation_id,
       a.name AS algorithm_name,
       COALESCE(parent.name, a.name) AS canonical_algorithm_name,
       l.name AS library_name,
       l.language AS library_language,
       i.is_reference AS is_reference,
       refi.id AS canonical_reference_implementation_id,
       refl.name AS canonical_reference_library_name
FROM implementations i
JOIN algorithms a ON i.algorithm_id = a.id
LEFT JOIN algorithms parent ON parent.id = a.approximates_algorithm_id
JOIN libraries l ON i.library_id = l.id
LEFT JOIN implementations refi ON refi.algorithm_id = COALESCE(parent.id, a.id)
                             AND refi.is_reference = 1
LEFT JOIN libraries refl ON refl.id = refi.library_id;
