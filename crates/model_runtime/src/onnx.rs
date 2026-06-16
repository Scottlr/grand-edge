use crate::{
    artifacts::{ModelArtifactKind, ValidatedArtifactBundle},
    errors::ModelRuntimeError,
    runtime::{InferenceOutput, InferenceRequest},
};

pub fn infer(
    _request: InferenceRequest,
    bundle: &ValidatedArtifactBundle,
) -> Result<InferenceOutput, ModelRuntimeError> {
    let kind = match bundle.bundle.metadata.artifact_kind {
        ModelArtifactKind::GbdtRanker => "gbdt_ranker",
        ModelArtifactKind::ContextualBandit => "contextual_bandit",
        ModelArtifactKind::OnlineEnsemble => "online_ensemble",
        ModelArtifactKind::MetaLabel => "meta_label",
    };
    Err(ModelRuntimeError::UnsupportedArtifactKind(kind))
}
