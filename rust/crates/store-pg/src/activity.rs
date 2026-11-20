//! The days a student was active, shared by the home streak and Progres so both count the same
//! things: a question asked, a practice set submitted, a lesson started or completed. Each day is
//! the Jakarta calendar date of the stored instant (`AT TIME ZONE 'Asia/Jakarta'`, fixed UTC+7).

use std::collections::BTreeSet;

use belajar_ports::store::StoreError;
use chrono::NaiveDate;
use sqlx::{PgConnection, Row};
use uuid::Uuid;

/// Every Jakarta date on which `student_id` did anything.
pub(crate) async fn active_days(
    conn: &mut PgConnection,
    student_id: Uuid,
) -> Result<BTreeSet<NaiveDate>, StoreError> {
    let rows = sqlx::query(
        "SELECT DISTINCT d FROM ( \
             SELECT (created_at AT TIME ZONE 'Asia/Jakarta')::date AS d FROM messages \
             WHERE student_id = $1 AND role = 'user' \
             UNION \
             SELECT (submitted_at AT TIME ZONE 'Asia/Jakarta')::date FROM practice_sets \
             WHERE student_id = $1 AND submitted_at IS NOT NULL \
             UNION \
             SELECT (created_at AT TIME ZONE 'Asia/Jakarta')::date FROM lesson_progress \
             WHERE student_id = $1 \
             UNION \
             SELECT (completed_at AT TIME ZONE 'Asia/Jakarta')::date FROM lesson_progress \
             WHERE student_id = $1 AND completed_at IS NOT NULL \
         ) days",
    )
    .bind(student_id)
    .fetch_all(conn)
    .await
    .map_err(|err| StoreError::Internal(err.to_string()))?;
    Ok(rows
        .iter()
        .map(|row| row.get::<NaiveDate, _>("d"))
        .collect())
}
