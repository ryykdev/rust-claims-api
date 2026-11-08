use crate::{
    error::UserError,
    model::{Decision, DecisionReason, DecisionStatus},
};
use sqlx::PgPool;

// if a decision already exists return HttpStatus::CONFLICT
pub async fn save_decision(pool: &PgPool, decision: Decision) -> Result<(), UserError> {
    //
    sqlx::query_as!(
        Decision,
        r#"
    INSERT INTO decisions (
        id,
        timestamp,
        decision_status,
        payout,
        decision_reason
    )

    VALUES ($1, $2, $3, $4, $5)
            "#,
        decision.id,
        decision.timestamp,
        decision.decision_status as DecisionStatus,
        decision.payout,
        decision.decision_reason as DecisionReason,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        //
        log::warn!("save_decision: failed with: {e}");
        UserError::DecisionConflict
    })?;

    Ok(())
}
