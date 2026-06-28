use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state_persistence::{StateBackend, StateMachine};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Preparing,
    Prepared,
    Committing,
    Committed,
    RollingBack,
    RolledBack,
    Unknown,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Preparing => "preparing",
            Self::Prepared => "prepared",
            Self::Committing => "committing",
            Self::Committed => "committed",
            Self::RollingBack => "rolling_back",
            Self::RolledBack => "rolled_back",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "preparing" => Self::Preparing,
            "prepared" => Self::Prepared,
            "committing" => Self::Committing,
            "committed" => Self::Committed,
            "rolling_back" => Self::RollingBack,
            "rolled_back" => Self::RolledBack,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResult {
    pub participant_id: String,
    pub vote: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
    pub transaction_id: String,
    pub status: String,
    pub participants: Vec<ParticipantInfo>,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: serde_json::Value,
}

impl TransactionLog {
    fn new(transaction_id: String, participants: &[ParticipantInfo], metadata: serde_json::Value) -> Self {
        Self {
            transaction_id,
            status: TransactionStatus::Preparing.as_str().to_string(),
            participants: participants.to_vec(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata,
        }
    }
}

#[async_trait]
pub trait Participant: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn role(&self) -> &str;

    async fn prepare(&self, transaction_id: &str) -> Result<(), String>;
    async fn commit(&self, transaction_id: &str) -> Result<(), String>;
    async fn rollback(&self, transaction_id: &str) -> Result<(), String>;
}

pub struct TwoPhaseCommit {
    backend: Arc<dyn StateBackend>,
    state_machine: StateMachine,
}

impl TwoPhaseCommit {
    pub fn new(backend: Arc<dyn StateBackend>) -> Self {
        let sm = StateMachine::new(backend.clone());
        Self { backend, state_machine: sm }
    }

    fn log_key(transaction_id: &str) -> String {
        format!("2pc:tx:{transaction_id}")
    }

    async fn save_log(&self, log: &TransactionLog) -> Result<(), String> {
        let data = serde_json::to_vec(log).map_err(|e| e.to_string())?;
        self.backend.save(&Self::log_key(&log.transaction_id), &data).await
    }

    async fn load_log(&self, transaction_id: &str) -> Result<Option<TransactionLog>, String> {
        let raw = self.backend.load(&Self::log_key(transaction_id)).await?;
        match raw {
            Some(data) => {
                let log: TransactionLog = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
                Ok(Some(log))
            }
            None => Ok(None),
        }
    }

    async fn update_status(&self, transaction_id: &str, status: TransactionStatus) -> Result<TransactionLog, String> {
        let mut log =
            self.load_log(transaction_id).await?.ok_or_else(|| format!("Transaction not found: {transaction_id}"))?;
        log.status = status.as_str().to_string();
        log.updated_at = chrono::Utc::now().to_rfc3339();
        self.save_log(&log).await?;
        Ok(log)
    }

    pub async fn execute(
        &self,
        transaction_id: &str,
        participants: &[Arc<dyn Participant>],
        metadata: serde_json::Value,
    ) -> Result<TransactionLog, String> {
        let infos: Vec<ParticipantInfo> = participants
            .iter()
            .map(|p| ParticipantInfo { id: p.id().to_string(), name: p.name().to_string(), role: p.role().to_string() })
            .collect();

        let log = TransactionLog::new(transaction_id.to_string(), &infos, metadata);
        self.save_log(&log).await?;

        let _state = self
            .state_machine
            .create_state(&format!("2pc_{transaction_id}"), serde_json::json!({"phase": "begin"}))
            .await?;

        let vote_results = self.prepare_phase(transaction_id, participants).await?;
        let all_agreed = vote_results.iter().all(|v| v.vote);

        if all_agreed {
            self.update_status(transaction_id, TransactionStatus::Committing).await?;
            match self.commit_phase(transaction_id, participants).await {
                Ok(()) => {
                    let log = self.update_status(transaction_id, TransactionStatus::Committed).await?;
                    let _ = self
                        .state_machine
                        .transition(
                            &format!("2pc_{transaction_id}"),
                            crate::state_persistence::StateTransition::Completed,
                        )
                        .await;
                    return Ok(log);
                }
                Err(e) => {
                    let _ = self.update_status(transaction_id, TransactionStatus::RollingBack).await?;
                    let _ = self.rollback_phase(transaction_id, participants).await;
                    let log = self.update_status(transaction_id, TransactionStatus::RolledBack).await?;
                    let _ = self
                        .state_machine
                        .transition(&format!("2pc_{transaction_id}"), crate::state_persistence::StateTransition::Failed)
                        .await;
                    return Err(format!("Commit phase failed: {e}. Transaction rolled back. Status: {}", log.status));
                }
            }
        } else {
            let _ = self.update_status(transaction_id, TransactionStatus::RollingBack).await?;
            let _ = self.rollback_phase(transaction_id, participants).await;
            let log = self.update_status(transaction_id, TransactionStatus::RolledBack).await?;
            let _ = self
                .state_machine
                .transition(&format!("2pc_{transaction_id}"), crate::state_persistence::StateTransition::Failed)
                .await;

            let rejections: Vec<String> = vote_results
                .iter()
                .filter_map(|v| {
                    if !v.vote {
                        Some(format!(
                            "{}: {}",
                            v.participant_id,
                            v.reason.clone().unwrap_or_else(|| "no reason".to_string())
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            Err(format!(
                "Prepare phase failed: participants did not agree. Rejections: [{}]. Status: {}",
                rejections.join(", "),
                log.status
            ))
        }
    }

    async fn prepare_phase(
        &self,
        transaction_id: &str,
        participants: &[Arc<dyn Participant>],
    ) -> Result<Vec<VoteResult>, String> {
        self.update_status(transaction_id, TransactionStatus::Preparing).await?;

        let mut results = Vec::new();
        for p in participants {
            match p.prepare(transaction_id).await {
                Ok(()) => {
                    results.push(VoteResult { participant_id: p.id().to_string(), vote: true, reason: None });
                }
                Err(e) => {
                    results.push(VoteResult {
                        participant_id: p.id().to_string(),
                        vote: false,
                        reason: Some(e.clone()),
                    });
                    return Ok(results);
                }
            }
        }

        self.update_status(transaction_id, TransactionStatus::Prepared).await?;
        Ok(results)
    }

    async fn commit_phase(&self, transaction_id: &str, participants: &[Arc<dyn Participant>]) -> Result<(), String> {
        let mut errors = Vec::new();
        for p in participants {
            if let Err(e) = p.commit(transaction_id).await {
                errors.push(format!("{}: {}", p.id(), e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    async fn rollback_phase(&self, transaction_id: &str, participants: &[Arc<dyn Participant>]) -> Result<(), String> {
        let mut errors = Vec::new();
        for p in participants {
            if let Err(e) = p.rollback(transaction_id).await {
                errors.push(format!("{}: {}", p.id(), e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    pub async fn recover(
        &self,
        transaction_id: &str,
        participants: &[Arc<dyn Participant>],
    ) -> Result<TransactionLog, String> {
        let log = self
            .load_log(transaction_id)
            .await?
            .ok_or_else(|| format!("No transaction log found for: {transaction_id}"))?;

        let status = TransactionStatus::from_str(&log.status);
        match status {
            TransactionStatus::Preparing | TransactionStatus::Prepared => {
                self.rollback_phase(transaction_id, participants).await?;
                self.update_status(transaction_id, TransactionStatus::RolledBack).await
            }
            TransactionStatus::Committing => match self.commit_phase(transaction_id, participants).await {
                Ok(()) => self.update_status(transaction_id, TransactionStatus::Committed).await,
                Err(e) => {
                    self.rollback_phase(transaction_id, participants).await?;
                    self.update_status(transaction_id, TransactionStatus::RolledBack).await.map_err(|_| e.clone())?;
                    Err(format!("Recovery failed during commit: {e}"))
                }
            },
            TransactionStatus::RollingBack => {
                self.rollback_phase(transaction_id, participants).await?;
                self.update_status(transaction_id, TransactionStatus::RolledBack).await
            }
            TransactionStatus::Committed | TransactionStatus::RolledBack => Ok(log),
            TransactionStatus::Unknown => {
                Err(format!("Unknown transaction status for {transaction_id}: cannot recover"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockParticipant {
        id: String,
        name: String,
        role: String,
        should_fail_prepare: bool,
        should_fail_commit: bool,
        prepare_called: Mutex<bool>,
        commit_called: Mutex<bool>,
        rollback_called: Mutex<bool>,
    }

    impl MockParticipant {
        fn new(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                role: "worker".to_string(),
                should_fail_prepare: false,
                should_fail_commit: false,
                prepare_called: Mutex::new(false),
                commit_called: Mutex::new(false),
                rollback_called: Mutex::new(false),
            }
        }

        fn with_prepare_failure(mut self) -> Self {
            self.should_fail_prepare = true;
            self
        }

        fn with_commit_failure(mut self) -> Self {
            self.should_fail_commit = true;
            self
        }

        fn prepare_was_called(&self) -> bool {
            *self.prepare_called.lock().unwrap()
        }

        fn commit_was_called(&self) -> bool {
            *self.commit_called.lock().unwrap()
        }

        fn rollback_was_called(&self) -> bool {
            *self.rollback_called.lock().unwrap()
        }
    }

    #[async_trait]
    impl Participant for MockParticipant {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn role(&self) -> &str {
            &self.role
        }

        async fn prepare(&self, _transaction_id: &str) -> Result<(), String> {
            *self.prepare_called.lock().unwrap() = true;
            if self.should_fail_prepare {
                Err(format!("{}: prepare failed", self.id))
            } else {
                Ok(())
            }
        }

        async fn commit(&self, _transaction_id: &str) -> Result<(), String> {
            *self.commit_called.lock().unwrap() = true;
            if self.should_fail_commit {
                Err(format!("{}: commit failed", self.id))
            } else {
                Ok(())
            }
        }

        async fn rollback(&self, _transaction_id: &str) -> Result<(), String> {
            *self.rollback_called.lock().unwrap() = true;
            Ok(())
        }
    }

    async fn make_2pc() -> (TwoPhaseCommit, std::path::PathBuf) {
        use crate::storage::Storage;
        let path = std::env::temp_dir().join(format!("test_2pc_{}.db", uuid::Uuid::new_v4()));
        let storage = Arc::new(Storage::open(&path).await.unwrap());
        let backend: Arc<dyn StateBackend> = Arc::new(crate::state_persistence::LocalBackend::new(storage));
        (TwoPhaseCommit::new(backend), path)
    }

    #[tokio::test]
    async fn two_phase_commit_success() {
        let (coordinator, _tmp) = make_2pc().await;
        let p1: Arc<dyn Participant> = Arc::new(MockParticipant::new("p1", "worker1"));
        let p2: Arc<dyn Participant> = Arc::new(MockParticipant::new("p2", "worker2"));

        let result =
            coordinator.execute("tx_success", &[p1.clone(), p2.clone()], serde_json::json!({"op": "test"})).await;

        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.status, TransactionStatus::Committed.as_str());
    }

    #[tokio::test]
    async fn two_phase_commit_prepare_failure_triggers_rollback() {
        let (coordinator, _tmp) = make_2pc().await;
        let p1: Arc<dyn Participant> = Arc::new(MockParticipant::new("p1", "good").with_commit_failure());
        let p2: Arc<dyn Participant> = Arc::new(MockParticipant::new("p2", "bad").with_prepare_failure());

        let result = coordinator.execute("tx_prepare_fail", &[p1.clone(), p2.clone()], serde_json::json!({})).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Prepare phase failed"));
        assert!(err.contains("p2"));
    }

    #[tokio::test]
    async fn two_phase_commit_commit_failure_triggers_rollback() {
        let (coordinator, _tmp) = make_2pc().await;
        let failing: Arc<dyn Participant> = Arc::new(MockParticipant::new("p1", "fail_commit").with_commit_failure());
        let passing: Arc<dyn Participant> = Arc::new(MockParticipant::new("p2", "ok"));

        let result =
            coordinator.execute("tx_commit_fail", &[failing.clone(), passing.clone()], serde_json::json!({})).await;

        assert!(result.is_err());
        let log = coordinator.load_log("tx_commit_fail").await.unwrap().unwrap();
        assert_eq!(log.status, TransactionStatus::RolledBack.as_str());
    }

    #[tokio::test]
    async fn two_phase_commit_empty_participants() {
        let (coordinator, _tmp) = make_2pc().await;
        let participants: Vec<Arc<dyn Participant>> = vec![];

        let result = coordinator.execute("tx_empty", &participants, serde_json::json!({})).await;

        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.status, TransactionStatus::Committed.as_str());
    }

    #[tokio::test]
    async fn recovery_from_preparing_state() {
        let (coordinator, _tmp) = make_2pc().await;
        let p1: Arc<dyn Participant> = Arc::new(MockParticipant::new("p1", "w1"));

        let log = TransactionLog {
            transaction_id: "tx_recover_preparing".to_string(),
            status: TransactionStatus::Preparing.as_str().to_string(),
            participants: vec![ParticipantInfo {
                id: "p1".to_string(),
                name: "w1".to_string(),
                role: "worker".to_string(),
            }],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata: serde_json::json!({}),
        };
        coordinator.save_log(&log).await.unwrap();

        let result = coordinator.recover("tx_recover_preparing", &[p1.clone()]).await;

        assert!(result.is_ok());
        let recovered = result.unwrap();
        assert_eq!(recovered.status, TransactionStatus::RolledBack.as_str());
    }

    #[tokio::test]
    async fn recovery_from_committing_state() {
        let (coordinator, _tmp) = make_2pc().await;
        let p1: Arc<dyn Participant> = Arc::new(MockParticipant::new("p1", "w1"));

        let log = TransactionLog {
            transaction_id: "tx_recover_committing".to_string(),
            status: TransactionStatus::Committing.as_str().to_string(),
            participants: vec![ParticipantInfo {
                id: "p1".to_string(),
                name: "w1".to_string(),
                role: "worker".to_string(),
            }],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata: serde_json::json!({}),
        };
        coordinator.save_log(&log).await.unwrap();

        let result = coordinator.recover("tx_recover_committing", &[p1.clone()]).await;

        assert!(result.is_ok());
        let recovered = result.unwrap();
        assert_eq!(recovered.status, TransactionStatus::Committed.as_str());
    }

    #[tokio::test]
    async fn recovery_already_completed_noop() {
        let (coordinator, _tmp) = make_2pc().await;
        let p1: Arc<dyn Participant> = Arc::new(MockParticipant::new("p1", "w1"));

        let log = TransactionLog {
            transaction_id: "tx_done".to_string(),
            status: TransactionStatus::Committed.as_str().to_string(),
            participants: vec![ParticipantInfo {
                id: "p1".to_string(),
                name: "w1".to_string(),
                role: "worker".to_string(),
            }],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata: serde_json::json!({}),
        };
        coordinator.save_log(&log).await.unwrap();

        let result = coordinator.recover("tx_done", &[p1]).await;
        assert!(result.is_ok());
        let recovered = result.unwrap();
        assert_eq!(recovered.status, TransactionStatus::Committed.as_str());
    }

    #[tokio::test]
    async fn recovery_unknown_transaction_errors() {
        let (coordinator, _tmp) = make_2pc().await;
        let p1: Arc<dyn Participant> = Arc::new(MockParticipant::new("p1", "w1"));

        let result = coordinator.recover("tx_nonexistent", &[p1]).await;
        assert!(result.is_err());
    }
}
