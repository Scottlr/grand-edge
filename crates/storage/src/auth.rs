use chrono::{DateTime, Utc};
use grand_edge_domain::{
    AuthSession, AuthenticatedUser, EmailAddress, SessionId, UpdateRiskProfile, UserId,
    UserRiskProfile,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::StorageError;

#[derive(Clone)]
pub struct AuthRepository {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct NewUserIdentity {
    pub user_id: UserId,
    pub email: EmailAddress,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuthRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user_identity(
        &self,
        identity: &NewUserIdentity,
    ) -> Result<AuthenticatedUser, StorageError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO users (user_id, display_name, created_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(identity.user_id.0)
        .bind(&identity.display_name)
        .bind(identity.created_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_auth_identities (user_id, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(identity.user_id.0)
        .bind(identity.email.as_str())
        .bind(&identity.password_hash)
        .bind(identity.created_at)
        .bind(identity.created_at)
        .execute(&mut *tx)
        .await?;

        let profile = UserRiskProfile::default_for_user(identity.user_id, identity.created_at);
        upsert_risk_profile_with_executor(&mut *tx, &profile).await?;
        tx.commit().await?;

        Ok(AuthenticatedUser {
            user_id: identity.user_id,
            email: identity.email.clone(),
            display_name: identity.display_name.clone(),
        })
    }

    pub async fn get_user_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<(AuthenticatedUser, String)>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT u.user_id, u.display_name, a.email, a.password_hash
            FROM user_auth_identities a
            INNER JOIN users u ON u.user_id = a.user_id
            WHERE a.email = $1
            LIMIT 1
            "#,
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some((
                AuthenticatedUser {
                    user_id: UserId(row.try_get::<Uuid, _>("user_id")?),
                    email: EmailAddress::new(row.try_get::<String, _>("email")?)?,
                    display_name: row.try_get("display_name")?,
                },
                row.try_get("password_hash")?,
            ))),
            None => Ok(None),
        }
    }

    pub async fn get_user_by_id(
        &self,
        user_id: UserId,
    ) -> Result<Option<AuthenticatedUser>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT u.user_id, u.display_name, a.email
            FROM users u
            LEFT JOIN user_auth_identities a ON a.user_id = u.user_id
            WHERE u.user_id = $1
            LIMIT 1
            "#,
        )
        .bind(user_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_authenticated_user).transpose()
    }

    pub async fn create_session(&self, session: &AuthSession) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO user_sessions (session_id, user_id, expires_at, revoked_at, created_at)
            VALUES ($1, $2, $3, NULL, $4)
            "#,
        )
        .bind(session.session_id.0)
        .bind(session.user_id.0)
        .bind(session.expires_at)
        .bind(session.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn revoke_session(&self, session_id: SessionId) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET revoked_at = $2
            WHERE session_id = $1
            "#,
        )
        .bind(session_id.0)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_active_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<AuthSession>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT session_id, user_id, created_at, expires_at
            FROM user_sessions
            WHERE session_id = $1
              AND revoked_at IS NULL
              AND expires_at > NOW()
            LIMIT 1
            "#,
        )
        .bind(session_id.0)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(AuthSession {
                session_id: SessionId(row.try_get::<Uuid, _>("session_id")?),
                user_id: UserId(row.try_get::<Uuid, _>("user_id")?),
                created_at: row.try_get("created_at")?,
                expires_at: row.try_get("expires_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_risk_profile(
        &self,
        user_id: UserId,
    ) -> Result<Option<UserRiskProfile>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT user_id, max_gp_per_item, max_portfolio_drawdown, min_expected_roi,
                   min_confidence, participation_rate, preferred_execution_mode, updated_at
            FROM user_risk_profiles
            WHERE user_id = $1
            LIMIT 1
            "#,
        )
        .bind(user_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_risk_profile).transpose()
    }

    pub async fn upsert_risk_profile(
        &self,
        profile: &UserRiskProfile,
    ) -> Result<UserRiskProfile, StorageError> {
        upsert_risk_profile_with_executor(&self.pool, profile).await?;
        Ok(profile.clone())
    }

    pub async fn update_risk_profile(
        &self,
        user_id: UserId,
        update: UpdateRiskProfile,
    ) -> Result<UserRiskProfile, StorageError> {
        let profile = UserRiskProfile::from_update(user_id, update, Utc::now())?;
        self.upsert_risk_profile(&profile).await
    }
}

async fn upsert_risk_profile_with_executor<'e, E>(
    executor: E,
    profile: &UserRiskProfile,
) -> Result<(), StorageError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
        INSERT INTO user_risk_profiles (
            user_id, max_gp_per_item, max_portfolio_drawdown, min_expected_roi,
            min_confidence, participation_rate, preferred_execution_mode, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8
        )
        ON CONFLICT (user_id) DO UPDATE SET
            max_gp_per_item = EXCLUDED.max_gp_per_item,
            max_portfolio_drawdown = EXCLUDED.max_portfolio_drawdown,
            min_expected_roi = EXCLUDED.min_expected_roi,
            min_confidence = EXCLUDED.min_confidence,
            participation_rate = EXCLUDED.participation_rate,
            preferred_execution_mode = EXCLUDED.preferred_execution_mode,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(profile.user_id.0)
    .bind(profile.max_gp_per_item.0)
    .bind(profile.max_portfolio_drawdown.get())
    .bind(profile.min_expected_roi.get())
    .bind(profile.min_confidence.get())
    .bind(profile.participation_rate.get())
    .bind(serde_json::to_string(&profile.preferred_execution_mode)?)
    .bind(profile.updated_at)
    .execute(executor)
    .await?;
    Ok(())
}

fn row_to_authenticated_user(
    row: sqlx::postgres::PgRow,
) -> Result<AuthenticatedUser, StorageError> {
    let email = row
        .try_get::<Option<String>, _>("email")?
        .unwrap_or_else(|| "local-default@grandedge.invalid".to_string());
    Ok(AuthenticatedUser {
        user_id: UserId(row.try_get::<Uuid, _>("user_id")?),
        email: EmailAddress::new(email)?,
        display_name: row.try_get("display_name")?,
    })
}

fn row_to_risk_profile(row: sqlx::postgres::PgRow) -> Result<UserRiskProfile, StorageError> {
    let mode = serde_json::from_str(&row.try_get::<String, _>("preferred_execution_mode")?)?;
    Ok(UserRiskProfile::from_update(
        UserId(row.try_get::<Uuid, _>("user_id")?),
        UpdateRiskProfile {
            max_gp_per_item: row.try_get("max_gp_per_item")?,
            max_portfolio_drawdown: row.try_get("max_portfolio_drawdown")?,
            min_expected_roi: row.try_get("min_expected_roi")?,
            min_confidence: row.try_get("min_confidence")?,
            participation_rate: row.try_get("participation_rate")?,
            preferred_execution_mode: mode,
        },
        row.try_get("updated_at")?,
    )?)
}
