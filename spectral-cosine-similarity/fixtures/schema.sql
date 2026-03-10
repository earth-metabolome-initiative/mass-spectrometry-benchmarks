-- Pure algorithm concepts (implementation-agnostic)
CREATE TABLE IF NOT EXISTS algorithms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE CHECK (length(trim(name)) > 0),
    description TEXT,
    approximates_algorithm_id INTEGER REFERENCES algorithms(id),
    CHECK (approximates_algorithm_id IS NULL OR approximates_algorithm_id <> id)
) STRICT;

-- Libraries / packages
CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    version TEXT NOT NULL CHECK (length(trim(version)) > 0),
    git_commit TEXT,
    git_url TEXT,
    language TEXT NOT NULL CHECK (length(trim(language)) > 0)
                      CHECK (language IN ('rust', 'python')),
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
    params TEXT NOT NULL CHECK (length(trim(params)) > 0)
                    CHECK (json_valid(params) = 1),
    UNIQUE(params)
) STRICT;

-- Molecules (linked to spectra via molecule_id FK)
CREATE TABLE IF NOT EXISTS molecules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    smiles TEXT NOT NULL CHECK (length(trim(smiles)) > 0),
    inchikey TEXT NOT NULL UNIQUE CHECK (length(trim(inchikey)) > 0)
) STRICT;

-- Fingerprint algorithm definitions (name + JSON params)
CREATE TABLE IF NOT EXISTS fingerprint_algorithms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE CHECK (length(trim(name)) > 0),
    params TEXT NOT NULL CHECK (length(trim(params)) > 0)
                    CHECK (json_valid(params) = 1)
) STRICT;

-- Per-molecule fingerprint blobs, one per algorithm
CREATE TABLE IF NOT EXISTS fingerprints (
    molecule_id INTEGER NOT NULL REFERENCES molecules(id),
    fingerprint_algorithm_id INTEGER NOT NULL REFERENCES fingerprint_algorithms(id),
    fingerprint BLOB NOT NULL,
    PRIMARY KEY (molecule_id, fingerprint_algorithm_id)
) STRICT, WITHOUT ROWID;

-- Spectra
CREATE TABLE IF NOT EXISTS spectra (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    raw_name TEXT NOT NULL CHECK (length(trim(raw_name)) > 0),
    source_file TEXT NOT NULL CHECK (length(trim(source_file)) > 0),
    spectrum_hash TEXT NOT NULL UNIQUE CHECK (length(trim(spectrum_hash)) > 0),
    precursor_mz REAL NOT NULL CHECK (precursor_mz > 0),
    num_peaks INTEGER NOT NULL CHECK (num_peaks > 0),
    peaks TEXT NOT NULL CHECK (json_valid(peaks) = 1),
    molecule_id INTEGER NOT NULL REFERENCES molecules(id)
) STRICT;

-- Results: similarity score + timing, one row per (pair, experiment, implementation)
CREATE TABLE IF NOT EXISTS results (
    left_id INTEGER NOT NULL REFERENCES spectra(id),
    right_id INTEGER NOT NULL REFERENCES spectra(id),
    experiment_id INTEGER NOT NULL REFERENCES experiments(id),
    implementation_id INTEGER NOT NULL REFERENCES implementations(id),
    score REAL NOT NULL CHECK (score = score)
                    CHECK (score >= 0.0 AND score <= 1.000001),
    matches INTEGER NOT NULL CHECK (matches >= -1),
    median_time_us REAL NOT NULL CHECK (median_time_us >= 0.0),
    CHECK (left_id <= right_id),
    PRIMARY KEY (left_id, right_id, experiment_id, implementation_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_results_impl_pair_exp
ON results(implementation_id, left_id, right_id, experiment_id);

-- Selected spectrum pairs (shared between Rust and Python)
CREATE TABLE IF NOT EXISTS selected_pairs (
    left_id INTEGER NOT NULL REFERENCES spectra(id),
    right_id INTEGER NOT NULL REFERENCES spectra(id),
    CHECK (left_id <= right_id),
    PRIMARY KEY (left_id, right_id)
) STRICT, WITHOUT ROWID;

-- Tanimoto similarity between molecule fingerprints for selected pairs
CREATE TABLE IF NOT EXISTS tanimoto_results (
    left_id INTEGER NOT NULL REFERENCES spectra(id),
    right_id INTEGER NOT NULL REFERENCES spectra(id),
    fingerprint_algorithm_id INTEGER NOT NULL REFERENCES fingerprint_algorithms(id),
    tanimoto_score REAL NOT NULL CHECK (tanimoto_score >= 0.0 AND tanimoto_score <= 1.0),
    CHECK (left_id <= right_id),
    PRIMARY KEY (left_id, right_id, fingerprint_algorithm_id)
) STRICT, WITHOUT ROWID;

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
