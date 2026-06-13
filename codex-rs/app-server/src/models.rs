use std::sync::Arc;

use codex_app_server_protocol::Model;
use codex_app_server_protocol::ModelServiceTier;
use codex_app_server_protocol::ModelUpgradeInfo;
use codex_app_server_protocol::ReasoningEffortOption;
use codex_core::ThreadManager;
use codex_models_manager::manager::RefreshStrategy;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use codex_protocol::openai_models::default_input_modalities;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderCatalogModel {
    pub(crate) provider_id: String,
    pub(crate) provider_name: String,
    pub(crate) model_id: String,
}

pub async fn supported_models(
    thread_manager: Arc<ThreadManager>,
    include_hidden: bool,
    provider_models: Vec<ProviderCatalogModel>,
) -> Vec<Model> {
    let presets = thread_manager
        .list_models(RefreshStrategy::OnlineIfUncached)
        .await
        .into_iter()
        .collect::<Vec<_>>();

    let mut models: Vec<Model> = presets
        .iter()
        .filter(|preset| preset.show_in_picker)
        .cloned()
        .map(model_from_preset)
        .collect();

    models.extend(provider_models.into_iter().map(model_from_provider_model));
    if include_hidden {
        models.extend(
            presets
                .iter()
                .filter(|preset| !preset.show_in_picker)
                .cloned()
                .map(model_from_preset),
        );
    }
    models
}

fn model_from_preset(preset: ModelPreset) -> Model {
    Model {
        id: preset.id.to_string(),
        model: preset.model.to_string(),
        upgrade: preset.upgrade.as_ref().map(|upgrade| upgrade.id.clone()),
        upgrade_info: preset.upgrade.as_ref().map(|upgrade| ModelUpgradeInfo {
            model: upgrade.id.clone(),
            upgrade_copy: upgrade.upgrade_copy.clone(),
            model_link: upgrade.model_link.clone(),
            migration_markdown: upgrade.migration_markdown.clone(),
        }),
        availability_nux: preset.availability_nux.map(Into::into),
        display_name: preset.display_name.to_string(),
        description: preset.description.to_string(),
        hidden: !preset.show_in_picker,
        supported_reasoning_efforts: reasoning_efforts_from_preset(
            preset.supported_reasoning_efforts,
        ),
        default_reasoning_effort: preset.default_reasoning_effort,
        input_modalities: preset.input_modalities,
        supports_personality: preset.supports_personality,
        additional_speed_tiers: preset.additional_speed_tiers,
        service_tiers: preset
            .service_tiers
            .into_iter()
            .map(|service_tier| ModelServiceTier {
                id: service_tier.id,
                name: service_tier.name,
                description: service_tier.description,
            })
            .collect(),
        default_service_tier: preset.default_service_tier,
        is_default: preset.is_default,
    }
}

fn model_from_provider_model(provider_model: ProviderCatalogModel) -> Model {
    let ProviderCatalogModel {
        provider_id,
        provider_name,
        model_id,
    } = provider_model;
    let provider_display = if provider_name.trim().is_empty() {
        provider_id.as_str()
    } else {
        provider_name.as_str()
    };
    let namespaced_id = format!("{provider_id}/{model_id}");

    Model {
        id: namespaced_id.clone(),
        model: namespaced_id,
        upgrade: None,
        upgrade_info: None,
        availability_nux: None,
        display_name: model_id,
        description: format!("{provider_display} model"),
        hidden: false,
        supported_reasoning_efforts: vec![ReasoningEffortOption {
            reasoning_effort: ReasoningEffort::None,
            description: "No reasoning controls".to_string(),
        }],
        default_reasoning_effort: ReasoningEffort::None,
        input_modalities: default_input_modalities(),
        supports_personality: false,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        is_default: false,
    }
}

fn reasoning_efforts_from_preset(
    efforts: Vec<ReasoningEffortPreset>,
) -> Vec<ReasoningEffortOption> {
    efforts
        .into_iter()
        .map(|preset| ReasoningEffortOption {
            reasoning_effort: preset.effort,
            description: preset.description,
        })
        .collect()
}
