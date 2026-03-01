use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use mass_spectrometry::prelude::*;
use serde::{Deserialize, Serialize};

/// A list of (mz, intensity) pairs stored as JSON text in SQLite.
///
/// Serialized format: `[[mz, intensity], ...]`
#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct Peaks(pub Vec<(f32, f32)>);

impl ToSql<Text, Sqlite> for Peaks {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(serde_json::to_string(&self.0)?);
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for Peaks {
    fn from_sql(
        bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        let vec: Vec<(f32, f32)> = serde_json::from_str(&s)?;
        Ok(Peaks(vec))
    }
}

impl Peaks {
    pub fn to_generic_spectrum(&self, precursor_mz: f32) -> GenericSpectrum<f32, f32> {
        let mut sorted = self.0.clone();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        // Merge duplicate m/z values by summing their intensities
        sorted.dedup_by(|b, a| {
            if a.0 == b.0 {
                a.1 += b.1;
                true
            } else {
                false
            }
        });

        let mut spectrum = GenericSpectrum::<f32, f32>::with_capacity(precursor_mz, sorted.len())
            .expect("failed to create GenericSpectrum with reserved peak capacity");
        for &(mz, intensity) in &sorted {
            spectrum
                .add_peak(mz, intensity)
                .expect("peaks should be strictly sorted after sort+dedup");
        }
        spectrum
    }
}

#[cfg(test)]
mod tests {
    use super::Peaks;
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Text;
    use mass_spectrometry::prelude::Spectrum;

    #[derive(diesel::QueryableByName)]
    struct PeakRow {
        #[diesel(sql_type = Text)]
        peaks: Peaks,
    }

    #[test]
    fn to_generic_spectrum_sorts_and_merges_duplicate_mz() {
        let peaks = Peaks(vec![(200.0, 2.0), (100.0, 1.0), (200.0, 3.0), (150.0, 4.0)]);
        let spectrum = peaks.to_generic_spectrum(400.0);

        assert_eq!(spectrum.len(), 3);
        assert_eq!(
            spectrum.peaks().collect::<Vec<_>>(),
            vec![(100.0, 1.0), (150.0, 4.0), (200.0, 5.0)]
        );
    }

    #[test]
    fn sqlite_roundtrips_peaks_json() {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("failed to open in-memory sqlite db");

        sql_query(
            "CREATE TABLE peak_rows (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                peaks TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .expect("failed to create test table");

        let input = Peaks(vec![(101.1, 5.0), (102.2, 6.0)]);

        sql_query("INSERT INTO peak_rows (peaks) VALUES (?)")
            .bind::<Text, _>(input.clone())
            .execute(&mut conn)
            .expect("failed to insert test peaks");

        let loaded = sql_query("SELECT peaks FROM peak_rows")
            .load::<PeakRow>(&mut conn)
            .expect("failed to load inserted peaks");

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].peaks.0, input.0);
    }
}
