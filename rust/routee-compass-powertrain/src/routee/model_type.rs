use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{EnergyRateUnit, GradeUnit, SpeedUnit},
};
use serde::{Deserialize, Serialize};

use super::{
    prediction_model::SpeedGradePredictionModel,
    smartcore::smartcore_speed_grade_model::SmartcoreSpeedGradeModel,
};

#[cfg(feature = "onnx")]
use super::onnx::onnx_speed_grade_model::OnnxSpeedGradeModel;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Smartcore,
    Onnx,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s)
    }
}

impl ModelType {
    /// builds a speed grade energy prediction model
    pub fn build<P: AsRef<Path>>(
        &self,
        energy_model_path: &P,
        energy_model_speed_unit: SpeedUnit,
        energy_model_grade_unit: GradeUnit,
        energy_model_energy_rate_unit: EnergyRateUnit,
    ) -> Result<Arc<dyn SpeedGradePredictionModel>, TraversalModelError> {
        // Load random forest binary file
        let model: Arc<dyn SpeedGradePredictionModel> = match self {
            ModelType::Smartcore => Arc::new(SmartcoreSpeedGradeModel::new(
                energy_model_path.clone(),
                energy_model_speed_unit.clone(),
                energy_model_grade_unit.clone(),
                energy_model_energy_rate_unit.clone(),
            )?),
            ModelType::Onnx => {
                #[cfg(feature = "onnx")]
                {
                    Arc::new(OnnxSpeedGradeModel::new(
                        energy_model_path.clone(),
                        energy_model_speed_unit.clone(),
                        energy_model_grade_unit.clone(),
                        energy_model_energy_rate_unit.clone(),
                    )?)
                }
                #[cfg(not(feature = "onnx"))]
                {
                    return Err(TraversalModelError::BuildError("Cannot build Onnx model without `onnx` feature enabled for compass-powertrain".to_string()));
                }
            }
        };
        Ok(model)
    }
}
