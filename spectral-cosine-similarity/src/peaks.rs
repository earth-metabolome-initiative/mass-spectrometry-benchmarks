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

        let mut spectrum = GenericSpectrum::<f32, f32>::with_capacity(precursor_mz, sorted.len());
        for &(mz, intensity) in &sorted {
            spectrum
                .add_peak(mz, intensity)
                .expect("peaks should be strictly sorted after sort+dedup");
        }
        spectrum
    }
}
